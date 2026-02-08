use anyhow::{anyhow, Result};
use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket},
        Json, State, WebSocketUpgrade,
    },
    http::{header, Method},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
    env, fs,
    hash::{Hash, Hasher},
    io::ErrorKind,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

mod room;

use rand::distributions::{Distribution, Uniform};
use room::{get_time_s, Room, ServerMsg};

const GARBAGE_COLLECT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 20); // 20 minutes
const GC_ROOM_TIMEOUT_S: u64 = 60 * 60; // 1 hour
const BASE_DECK_DIR: &str = "../static/assets/cards/";
const EXTRA_IMAGE_DIRS_ENV: &str = "TALESPIN_EXTRA_IMAGE_DIRS";
const DISABLE_BUILTIN_IMAGES_ENV: &str = "TALESPIN_DISABLE_BUILTIN_IMAGES_P";
const EXTRA_CARD_PREFIX: &str = "extra_dir__";

fn expand_home(path: &str) -> PathBuf {
    if path == "~" {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }

    PathBuf::from(path)
}

fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png"))
        .unwrap_or(false)
}

fn env_is_y(key: &str) -> bool {
    env::var(key)
        .map(|v| v.trim().eq_ignore_ascii_case("y"))
        .unwrap_or(false)
}

fn get_extra_image_dirs() -> Vec<PathBuf> {
    env::var(EXTRA_IMAGE_DIRS_ENV)
        .map(|raw| {
            raw.split('\n')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(expand_home)
                .collect()
        })
        .unwrap_or_else(|_| Vec::new())
}

fn clear_old_extra_links(base_deck_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(base_deck_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name.to_string_lossy().starts_with(EXTRA_CARD_PREFIX) {
            if let Err(err) = fs::remove_file(entry.path()) {
                if err.kind() != ErrorKind::NotFound {
                    println!(
                        "Warning: failed to remove stale extra card link {}: {}",
                        entry.path().display(),
                        err
                    );
                }
            }
        }
    }
    Ok(())
}

fn link_or_copy_image(source: &Path, target: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        if let Err(link_err) = symlink(source, target) {
            fs::copy(source, target).map_err(|copy_err| {
                anyhow!(
                    "failed to link {} -> {} ({}) and copy fallback also failed ({})",
                    source.display(),
                    target.display(),
                    link_err,
                    copy_err
                )
            })?;
        }
    }

    #[cfg(not(unix))]
    {
        fs::copy(source, target).map_err(|copy_err| {
            anyhow!(
                "failed to copy {} -> {} ({})",
                source.display(),
                target.display(),
                copy_err
            )
        })?;
    }

    Ok(())
}

fn load_extra_cards(base_deck_dir: &Path, extra_image_dirs: &[PathBuf]) -> Result<usize> {
    clear_old_extra_links(base_deck_dir)?;

    let mut loaded = 0usize;
    let mut seen_sources = HashSet::new();

    for dir_path in extra_image_dirs {
        let mut dirs_to_scan = VecDeque::from([dir_path.clone()]);

        while let Some(scan_dir) = dirs_to_scan.pop_front() {
            let entries = match fs::read_dir(&scan_dir) {
                Ok(entries) => entries,
                Err(err) => {
                    println!(
                        "Warning: unable to read extra image dir {}: {}",
                        scan_dir.display(),
                        err
                    );
                    continue;
                }
            };

            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        println!(
                            "Warning: failed reading entry in {}: {}",
                            scan_dir.display(),
                            err
                        );
                        continue;
                    }
                };

                let file_type = match entry.file_type() {
                    Ok(file_type) => file_type,
                    Err(err) => {
                        println!(
                            "Warning: failed to read entry type {}: {}",
                            entry.path().display(),
                            err
                        );
                        continue;
                    }
                };

                if file_type.is_dir() {
                    dirs_to_scan.push_back(entry.path());
                    continue;
                }

                if !file_type.is_file() {
                    continue;
                }

                let source = entry.path();
                if !is_supported_image(&source) {
                    continue;
                }
                if !seen_sources.insert(source.clone()) {
                    continue;
                }

                let mut hasher = DefaultHasher::new();
                source.to_string_lossy().hash(&mut hasher);
                let hash = hasher.finish();
                let ext = source
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("jpg")
                    .to_ascii_lowercase();
                let target_name = format!("{EXTRA_CARD_PREFIX}{hash:016x}.{ext}");
                let target = base_deck_dir.join(target_name);

                if target.exists() {
                    let _ = fs::remove_file(&target);
                }

                match link_or_copy_image(&source, &target) {
                    Ok(_) => loaded += 1,
                    Err(err) => println!(
                        "Warning: failed to register extra image {}: {}",
                        source.display(),
                        err
                    ),
                }
            }
        }
    }

    Ok(loaded)
}

fn load_base_deck(base_deck_dir: &Path) -> Result<Vec<String>> {
    let mut base_deck: Vec<String> = fs::read_dir(base_deck_dir)?
        .filter_map(|res| match res {
            Ok(entry) => Some(entry),
            Err(err) => {
                println!(
                    "Warning: failed reading deck entry in {}: {}",
                    base_deck_dir.display(),
                    err
                );
                None
            }
        })
        .filter_map(|entry| {
            if !is_supported_image(&entry.path()) {
                return None;
            }

            entry.file_name().into_string().ok()
        })
        .collect();

    base_deck.sort();
    Ok(base_deck)
}

// main object for server
#[derive(Debug, Clone)]
struct ServerState {
    rooms: DashMap<String, Arc<Room>>,
    base_deck: Arc<Vec<String>>,
}

impl ServerState {
    fn new() -> Result<Self> {
        let base_deck_dir = Path::new(BASE_DECK_DIR);
        let extra_image_dirs = get_extra_image_dirs();
        let loaded_extra_cards = load_extra_cards(base_deck_dir, &extra_image_dirs)?;
        let disable_builtin_images = env_is_y(DISABLE_BUILTIN_IMAGES_ENV);

        if !extra_image_dirs.is_empty() && loaded_extra_cards == 0 {
            return Err(anyhow!(
                "No supported images (.jpg/.jpeg/.png) were found in {}. Checked {} director{}.",
                EXTRA_IMAGE_DIRS_ENV,
                extra_image_dirs.len(),
                if extra_image_dirs.len() == 1 {
                    "y"
                } else {
                    "ies"
                }
            ));
        }

        if disable_builtin_images && loaded_extra_cards == 0 {
            return Err(anyhow!(
                "{}=y requires at least one image from {}, but none were loaded.",
                DISABLE_BUILTIN_IMAGES_ENV,
                EXTRA_IMAGE_DIRS_ENV
            ));
        }

        let mut base_deck = load_base_deck(base_deck_dir)?;
        if disable_builtin_images {
            base_deck.retain(|f| f.starts_with(EXTRA_CARD_PREFIX));
        }

        if base_deck.is_empty() {
            return Err(anyhow!(
                "No cards available after loading images. Check built-in cards and {}.",
                EXTRA_IMAGE_DIRS_ENV
            ));
        }

        println!(
            "Loaded {} cards ({} extra cards from custom dirs; builtins {})",
            base_deck.len(),
            loaded_extra_cards,
            if disable_builtin_images {
                "disabled"
            } else {
                "enabled"
            }
        );

        Ok(ServerState {
            rooms: DashMap::new(),
            base_deck: Arc::new(base_deck),
        })
    }

    async fn create_room(&self) -> Result<ServerMsg> {
        let mut room_id = generate_room_id(4);

        // println!("create room: 0");
        while (self.get_room(&room_id)).is_some() {
            room_id = generate_room_id(4);
        }

        let room = Room::new(&room_id, self.base_deck.clone());
        let msg = room.get_room_state().await;
        self.rooms.insert(room_id.clone(), Arc::new(room));
        Ok(msg)
    }

    async fn join_room(&self, room_id: &str, socket: &mut WebSocket, name: &str) -> Result<()> {
        // hold no reference to inside the dashmap to prevent deadlock
        if let Some(room) = self.get_room(room_id) {
            room.on_connection(socket, name).await;
        } else {
            socket.send(ServerMsg::InvalidRoomId {}.into()).await?;
            return Ok(());
        }

        Ok(())
    }

    fn get_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.get(room_id).map(|r| r.value().clone())
    }

    fn stats(&self) -> HashMap<String, (usize, u64)> {
        self.rooms
            .iter()
            .map(|r| {
                (
                    r.key().clone(),
                    (r.value().num_active(), r.value().last_access()),
                )
            })
            .collect()
    }

    fn garbage_collect(&self) {
        let mut to_remove = Vec::new();
        for entry in &self.rooms {
            // hasn't been accessed in an hour
            if entry.value().num_active() == 0
                && get_time_s() - entry.value().last_access() > GC_ROOM_TIMEOUT_S
            {
                to_remove.push(entry.key().clone());
            }
        }

        println!("(gc) rooms to delete {:?}", to_remove);
        for room_id in to_remove {
            self.rooms.remove(&room_id);
        }
    }
}

async fn garbage_collect(state: Arc<ServerState>) {
    loop {
        tokio::time::sleep(GARBAGE_COLLECT_INTERVAL).await;
        state.garbage_collect();
    }
}

fn generate_room_id(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let letters = Uniform::new_inclusive(b'a', b'z'); // Range of lowercase letters
    (0..length)
        .map(|_| letters.sample(&mut rng) as char)
        .collect()
}

#[tokio::main]
async fn main() {
    let state = Arc::new(ServerState::new().unwrap());

    tokio::spawn(garbage_collect(state.clone()));

    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/create", post(create_room_handler))
        .route("/exists", post(exists_handler))
        .route("/stats", get(stats_handler))
        .route("/", get(root))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn create_room_handler(State(state): State<Arc<ServerState>>) -> String {
    let room = state.create_room().await;
    // json response with room id

    if let Ok(room_state) = room {
        serde_json::to_string(&room_state).unwrap()
    } else {
        serde_json::to_string(&room::ServerMsg::ErrorMsg(
            "Failed to create room".to_string(),
        ))
        .unwrap()
    }
}

async fn exists_handler(
    State(state): State<Arc<ServerState>>,
    Json(room_id): Json<String>,
) -> &'static str {
    if state.get_room(&room_id).is_some() {
        "true"
    } else {
        "false"
    }
}

async fn stats_handler(State(state): State<Arc<ServerState>>) -> String {
    serde_json::to_string(&state.stats()).unwrap()
}

async fn root() -> &'static str {
    "Hello, world!"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<ServerState>) {
    let res = initialize_socket(&mut socket, state).await;

    if let Err(e) = res {
        println!("Error in initialize_socket: {}", e);
    }
}

async fn initialize_socket(socket: &mut WebSocket, state: Arc<ServerState>) -> Result<()> {
    let msg = socket
        .recv()
        .await
        .ok_or_else(|| anyhow!("Expected initial message from client"))??;

    if let WsMessage::Text(s) = msg {
        if let Ok(msg) = serde_json::from_str(&s) {
            if let room::ClientMsg::JoinRoom { room_id, name } = msg {
                if name.len() > 30 {
                    socket
                        .send(room::ServerMsg::ErrorMsg("Name too long".to_string()).into())
                        .await?;
                    return Err(anyhow!("Name too long"));
                }
                state
                    .join_room(&room_id.to_lowercase(), socket, &name)
                    .await?
            }
        }
    }

    Ok(())
}
