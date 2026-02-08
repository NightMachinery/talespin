use anyhow::{anyhow, Context, Result};
use axum::{
    body::Bytes,
    extract::{
        ws::{Message as WsMessage, WebSocket},
        Json, Path as AxumPath, State, WebSocketUpgrade,
    },
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use image::{
    codecs::{avif::AvifEncoder, jpeg::JpegEncoder},
    imageops::FilterType,
    DynamicImage, ExtendedColorType, GenericImageView, ImageEncoder,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, fs,
    io::BufWriter,
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
use room::{get_time_s, Room, ServerMsg, WinCondition};

const GARBAGE_COLLECT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 20); // 20 minutes
const ROOM_MAINTENANCE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
const GC_ROOM_TIMEOUT_S: u64 = 60 * 60; // 1 hour

const BUILTIN_IMAGE_DIR: &str = "../static/assets/cards/";
const LEGACY_EXTRA_CARD_PREFIX: &str = "extra_dir__";

const EXTRA_IMAGE_DIRS_ENV: &str = "TALESPIN_EXTRA_IMAGE_DIRS";
const DISABLE_BUILTIN_IMAGES_ENV: &str = "TALESPIN_DISABLE_BUILTIN_IMAGES_P";
const SNIFF_EXTENSIONLESS_IMAGES_ENV: &str = "TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P";
const CACHE_DIR_ENV: &str = "TALESPIN_CACHE_DIR";
const CARD_ASPECT_RATIO_ENV: &str = "TALESPIN_CARD_ASPECT_RATIO";
const CARD_LONG_SIDE_ENV: &str = "TALESPIN_CARD_LONG_SIDE";
const CARD_CACHE_FORMAT_ENV: &str = "TALESPIN_CARD_CACHE_FORMAT";
const DEFAULT_WIN_POINTS_ENV: &str = "TALESPIN_DEFAULT_WIN_POINTS";

const DEFAULT_CARD_ASPECT_RATIO: &str = "2:3";
const DEFAULT_CARD_LONG_SIDE: u32 = 1536;
const DEFAULT_WIN_POINTS: u16 = 10;
const DEFAULT_CACHE_DIR: &str = "~/.cache/talespin";
const CACHE_SUBDIR_CARDS: &str = "cards";

const CARD_AVIF_QUALITY: u8 = 80;
const CARD_AVIF_SPEED: u8 = 4;
const CARD_JPEG_QUALITY: u8 = 90;
const NORMALIZATION_PIPELINE_VERSION: &str = "v1";

#[derive(Debug, Clone, Copy)]
enum CacheImageFormat {
    Avif,
    Jpeg,
}

impl CacheImageFormat {
    fn from_env_value(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "avif" => Some(Self::Avif),
            "jpeg" | "jpg" => Some(Self::Jpeg),
            _ => None,
        }
    }

    fn env_value(self) -> &'static str {
        match self {
            Self::Avif => "avif",
            Self::Jpeg => "jpeg",
        }
    }

    fn file_extension(self) -> &'static str {
        match self {
            Self::Avif => "avif",
            Self::Jpeg => "jpg",
        }
    }

    fn mime_type(self) -> &'static str {
        match self {
            Self::Avif => "image/avif",
            Self::Jpeg => "image/jpeg",
        }
    }
}

const DEFAULT_CARD_CACHE_FORMAT: CacheImageFormat = CacheImageFormat::Avif;

#[derive(Debug, Clone)]
struct NormalizationConfig {
    ratio_width: u32,
    ratio_height: u32,
    long_side: u32,
    cache_format: CacheImageFormat,
    cards_cache_dir: PathBuf,
}

impl NormalizationConfig {
    fn from_env() -> Result<Self> {
        let (ratio_width, ratio_height) = parse_ratio_from_env();
        let long_side = parse_long_side_from_env();
        let cache_format = parse_cache_image_format_from_env();

        let cache_root = env::var(CACHE_DIR_ENV)
            .map(|v| expand_home(v.trim()))
            .unwrap_or_else(|_| expand_home(DEFAULT_CACHE_DIR));
        let cards_cache_dir = cache_root.join(CACHE_SUBDIR_CARDS);
        fs::create_dir_all(&cards_cache_dir).with_context(|| {
            format!(
                "Failed to create cards cache directory {}",
                cards_cache_dir.display()
            )
        })?;

        Ok(Self {
            ratio_width,
            ratio_height,
            long_side,
            cache_format,
            cards_cache_dir,
        })
    }

    fn output_dimensions(&self) -> (u32, u32) {
        if self.ratio_width <= self.ratio_height {
            let height = self.long_side.max(1);
            let width = (((height as f64) * (self.ratio_width as f64) / (self.ratio_height as f64))
                .round() as u32)
                .max(1);
            (width, height)
        } else {
            let width = self.long_side.max(1);
            let height = (((width as f64) * (self.ratio_height as f64) / (self.ratio_width as f64))
                .round() as u32)
                .max(1);
            (width, height)
        }
    }
}

#[derive(Debug)]
struct LoadedCards {
    deck: Vec<String>,
    cards: HashMap<String, PathBuf>,
    loaded_builtin: usize,
    loaded_extra: usize,
    failed_sources: usize,
}

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

fn hash_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn parse_ratio(raw: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = raw.trim().split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let width = parts[0].trim().parse::<u32>().ok()?;
    let height = parts[1].trim().parse::<u32>().ok()?;

    if width == 0 || height == 0 {
        return None;
    }

    Some((width, height))
}

fn parse_ratio_from_env() -> (u32, u32) {
    if let Ok(raw) = env::var(CARD_ASPECT_RATIO_ENV) {
        if let Some((w, h)) = parse_ratio(&raw) {
            return (w, h);
        }

        println!(
            "Warning: invalid {}='{}'; using default {}",
            CARD_ASPECT_RATIO_ENV, raw, DEFAULT_CARD_ASPECT_RATIO
        );
    }

    parse_ratio(DEFAULT_CARD_ASPECT_RATIO).expect("DEFAULT_CARD_ASPECT_RATIO must be a valid ratio")
}

fn parse_long_side_from_env() -> u32 {
    if let Ok(raw) = env::var(CARD_LONG_SIDE_ENV) {
        if let Ok(long_side) = raw.trim().parse::<u32>() {
            if long_side > 0 {
                return long_side;
            }
        }

        println!(
            "Warning: invalid {}='{}'; using default {}",
            CARD_LONG_SIDE_ENV, raw, DEFAULT_CARD_LONG_SIDE
        );
    }

    DEFAULT_CARD_LONG_SIDE
}

fn parse_cache_image_format_from_env() -> CacheImageFormat {
    if let Ok(raw) = env::var(CARD_CACHE_FORMAT_ENV) {
        if let Some(format) = CacheImageFormat::from_env_value(&raw) {
            return format;
        }

        println!(
            "Warning: invalid {}='{}'; using default {}",
            CARD_CACHE_FORMAT_ENV,
            raw,
            DEFAULT_CARD_CACHE_FORMAT.env_value()
        );
    }

    DEFAULT_CARD_CACHE_FORMAT
}

fn parse_default_win_points_from_env() -> u16 {
    if let Ok(raw) = env::var(DEFAULT_WIN_POINTS_ENV) {
        if let Ok(value) = raw.trim().parse::<u16>() {
            if value > 0 {
                return value;
            }
        }

        println!(
            "Warning: invalid {}='{}'; using default {}",
            DEFAULT_WIN_POINTS_ENV, raw, DEFAULT_WIN_POINTS
        );
    }

    DEFAULT_WIN_POINTS
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

fn has_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "webp"
            )
        })
        .unwrap_or(false)
}

fn sniff_supported_extensionless_image(path: &Path) -> bool {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => {
            println!(
                "Warning: failed to read extensionless image candidate {}: {}",
                path.display(),
                err
            );
            return false;
        }
    };

    match infer::get(&bytes) {
        Some(kind) => matches!(kind.mime_type(), "image/jpeg" | "image/png" | "image/webp"),
        None => false,
    }
}

fn is_supported_image(path: &Path, sniff_extensionless_images: bool) -> bool {
    if has_supported_extension(path) {
        return true;
    }

    sniff_extensionless_images
        && path.extension().is_none()
        && sniff_supported_extensionless_image(path)
}

fn cleanup_legacy_generated_cards() -> Result<()> {
    let builtin_dir = Path::new(BUILTIN_IMAGE_DIR);
    if !builtin_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(builtin_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name
            .to_string_lossy()
            .starts_with(LEGACY_EXTRA_CARD_PREFIX)
        {
            if let Err(err) = fs::remove_file(entry.path()) {
                println!(
                    "Warning: failed to remove legacy generated card {}: {}",
                    entry.path().display(),
                    err
                );
            }
        }
    }

    Ok(())
}

fn collect_image_files_recursive(
    root: &Path,
    strict_root: bool,
    sniff_extensionless_images: bool,
) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut dirs_to_scan = VecDeque::from([root.to_path_buf()]);
    let mut visited_dirs = HashSet::new();

    while let Some(scan_dir) = dirs_to_scan.pop_front() {
        let resolved_scan_dir = match fs::canonicalize(&scan_dir) {
            Ok(path) => path,
            Err(err) => {
                if strict_root && scan_dir == root {
                    return Err(anyhow!(
                        "Unable to resolve image directory {}: {}",
                        scan_dir.display(),
                        err
                    ));
                }

                println!(
                    "Warning: unable to resolve image directory {}: {}",
                    scan_dir.display(),
                    err
                );
                continue;
            }
        };

        if !visited_dirs.insert(resolved_scan_dir.clone()) {
            continue;
        }

        let entries = match fs::read_dir(&resolved_scan_dir) {
            Ok(entries) => entries,
            Err(err) => {
                if strict_root && scan_dir == root {
                    return Err(anyhow!(
                        "Unable to read image directory {}: {}",
                        scan_dir.display(),
                        err
                    ));
                }

                println!(
                    "Warning: unable to read image directory {}: {}",
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
                        resolved_scan_dir.display(),
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

            let resolved_entry = match fs::canonicalize(entry.path()) {
                Ok(path) => path,
                Err(err) => {
                    println!(
                        "Warning: unable to resolve entry {}: {}",
                        entry.path().display(),
                        err
                    );
                    continue;
                }
            };

            if file_type.is_dir() || resolved_entry.is_dir() {
                dirs_to_scan.push_back(resolved_entry);
                continue;
            }

            if (file_type.is_file() || resolved_entry.is_file())
                && is_supported_image(&resolved_entry, sniff_extensionless_images)
            {
                found.push(resolved_entry);
            }
        }
    }

    found.sort();
    Ok(found)
}

fn center_crop_rect(
    src_width: u32,
    src_height: u32,
    ratio_width: u32,
    ratio_height: u32,
) -> (u32, u32, u32, u32) {
    let src_width_u64 = src_width as u64;
    let src_height_u64 = src_height as u64;
    let ratio_width_u64 = ratio_width as u64;
    let ratio_height_u64 = ratio_height as u64;

    if src_width_u64 * ratio_height_u64 > src_height_u64 * ratio_width_u64 {
        let crop_width = ((src_height_u64 * ratio_width_u64) / ratio_height_u64).max(1) as u32;
        let offset_x = (src_width.saturating_sub(crop_width)) / 2;
        (offset_x, 0, crop_width, src_height)
    } else {
        let crop_height = ((src_width_u64 * ratio_height_u64) / ratio_width_u64).max(1) as u32;
        let offset_y = (src_height.saturating_sub(crop_height)) / 2;
        (0, offset_y, src_width, crop_height)
    }
}

fn normalize_source_to_cache(
    source: &Path,
    config: &NormalizationConfig,
) -> Result<(String, PathBuf)> {
    let bytes = fs::read(source)
        .with_context(|| format!("Failed to read source image {}", source.display()))?;

    let source_hash = hash_hex(&bytes);
    let (output_width, output_height) = config.output_dimensions();

    let encoding_descriptor = match config.cache_format {
        CacheImageFormat::Avif => format!(
            "fmt=avif|quality={}|speed={}",
            CARD_AVIF_QUALITY, CARD_AVIF_SPEED
        ),
        CacheImageFormat::Jpeg => format!("fmt=jpeg|quality={}", CARD_JPEG_QUALITY),
    };
    let transform_descriptor = format!(
        "source={source_hash}|ratio={}:{}|long_side={}|output={}x{}|{}|pipeline={}",
        config.ratio_width,
        config.ratio_height,
        config.long_side,
        output_width,
        output_height,
        encoding_descriptor,
        NORMALIZATION_PIPELINE_VERSION
    );
    let final_hash = hash_hex(transform_descriptor.as_bytes());
    let card_id = final_hash.clone();
    let cache_path = config.cards_cache_dir.join(format!(
        "{final_hash}.{}",
        config.cache_format.file_extension()
    ));

    if !cache_path.exists() {
        let source_image = image::load_from_memory(&bytes)
            .with_context(|| format!("Failed to decode image {}", source.display()))?;

        let (src_width, src_height) = source_image.dimensions();
        if src_width == 0 || src_height == 0 {
            return Err(anyhow!(
                "Image {} has invalid dimensions {}x{}",
                source.display(),
                src_width,
                src_height
            ));
        }

        let (crop_x, crop_y, crop_width, crop_height) = center_crop_rect(
            src_width,
            src_height,
            config.ratio_width,
            config.ratio_height,
        );

        let cropped =
            image::imageops::crop_imm(&source_image, crop_x, crop_y, crop_width, crop_height)
                .to_image();

        let resized = DynamicImage::ImageRgba8(cropped).resize_exact(
            output_width,
            output_height,
            FilterType::Lanczos3,
        );

        let file = fs::File::create(&cache_path)
            .with_context(|| format!("Failed to create cache file {}", cache_path.display()))?;
        let mut writer = BufWriter::new(file);
        match config.cache_format {
            CacheImageFormat::Avif => {
                let rgba = resized.to_rgba8();
                let (width, height) = rgba.dimensions();
                let encoder = AvifEncoder::new_with_speed_quality(
                    &mut writer,
                    CARD_AVIF_SPEED,
                    CARD_AVIF_QUALITY,
                );
                encoder
                    .write_image(rgba.as_raw(), width, height, ExtendedColorType::Rgba8)
                    .with_context(|| {
                        format!("Failed to encode cached image {}", cache_path.display())
                    })?;
            }
            CacheImageFormat::Jpeg => {
                let mut encoder = JpegEncoder::new_with_quality(&mut writer, CARD_JPEG_QUALITY);
                encoder.encode_image(&resized).with_context(|| {
                    format!("Failed to encode cached image {}", cache_path.display())
                })?;
            }
        }
    }

    Ok((card_id, cache_path))
}

fn load_cards(
    config: &NormalizationConfig,
    extra_image_dirs: &[PathBuf],
    disable_builtin_images: bool,
    sniff_extensionless_images: bool,
) -> Result<LoadedCards> {
    let builtin_sources = if disable_builtin_images {
        Vec::new()
    } else {
        collect_image_files_recursive(
            Path::new(BUILTIN_IMAGE_DIR),
            true,
            sniff_extensionless_images,
        )?
    };

    let mut extra_sources = Vec::new();
    for dir in extra_image_dirs {
        extra_sources.extend(collect_image_files_recursive(
            dir,
            false,
            sniff_extensionless_images,
        )?);
    }

    if !extra_image_dirs.is_empty() && extra_sources.is_empty() {
        return Err(anyhow!(
            "No supported images (.jpg/.jpeg/.png/.webp) were found in {}. Checked {} director{}.",
            EXTRA_IMAGE_DIRS_ENV,
            extra_image_dirs.len(),
            if extra_image_dirs.len() == 1 {
                "y"
            } else {
                "ies"
            }
        ));
    }

    if disable_builtin_images && extra_sources.is_empty() {
        return Err(anyhow!(
            "{}=y requires at least one image from {}, but none were loaded.",
            DISABLE_BUILTIN_IMAGES_ENV,
            EXTRA_IMAGE_DIRS_ENV
        ));
    }

    let mut seen_sources = HashSet::new();
    let mut seen_card_ids = HashSet::new();
    let mut deck = Vec::new();
    let mut cards = HashMap::new();
    let mut loaded_builtin = 0usize;
    let mut loaded_extra = 0usize;
    let mut failed_sources = 0usize;

    for source in builtin_sources {
        if !seen_sources.insert(source.clone()) {
            continue;
        }

        match normalize_source_to_cache(&source, config) {
            Ok((card_id, cache_path)) => {
                if seen_card_ids.insert(card_id.clone()) {
                    deck.push(card_id.clone());
                    cards.insert(card_id, cache_path);
                    loaded_builtin += 1;
                }
            }
            Err(err) => {
                failed_sources += 1;
                println!(
                    "Warning: failed to normalize built-in image {}: {}",
                    source.display(),
                    err
                );
            }
        }
    }

    for source in extra_sources {
        if !seen_sources.insert(source.clone()) {
            continue;
        }

        match normalize_source_to_cache(&source, config) {
            Ok((card_id, cache_path)) => {
                if seen_card_ids.insert(card_id.clone()) {
                    deck.push(card_id.clone());
                    cards.insert(card_id, cache_path);
                    loaded_extra += 1;
                }
            }
            Err(err) => {
                failed_sources += 1;
                println!(
                    "Warning: failed to normalize extra image {}: {}",
                    source.display(),
                    err
                );
            }
        }
    }

    if deck.is_empty() {
        return Err(anyhow!(
            "No cards available after loading images. Check {} and {}.",
            BUILTIN_IMAGE_DIR,
            EXTRA_IMAGE_DIRS_ENV
        ));
    }

    deck.sort();

    Ok(LoadedCards {
        deck,
        cards,
        loaded_builtin,
        loaded_extra,
        failed_sources,
    })
}

#[derive(Debug, Deserialize)]
struct CreateRoomRequest {
    win_condition: Option<WinCondition>,
    creator_name: Option<String>,
}

#[derive(Debug)]
struct CreateRoomConfig {
    win_condition: WinCondition,
    creator_name: Option<String>,
}

fn validate_win_condition(win_condition: WinCondition) -> Result<WinCondition> {
    match win_condition {
        WinCondition::Points { target_points } => {
            if target_points == 0 {
                return Err(anyhow!("target_points must be >= 1"));
            }
            Ok(WinCondition::Points { target_points })
        }
        WinCondition::Cycles { target_cycles } => {
            if target_cycles == 0 {
                return Err(anyhow!("target_cycles must be >= 1"));
            }
            Ok(WinCondition::Cycles { target_cycles })
        }
        WinCondition::CardsFinish => Ok(WinCondition::CardsFinish),
    }
}

fn parse_create_room_win_condition(
    body: &[u8],
    _default_points_target: u16,
) -> Result<CreateRoomConfig> {
    if body.is_empty() || body.iter().all(|b| b.is_ascii_whitespace()) {
        return Ok(CreateRoomConfig {
            win_condition: WinCondition::CardsFinish,
            creator_name: None,
        });
    }

    let request: CreateRoomRequest =
        serde_json::from_slice(body).context("Failed to parse create-room request payload")?;
    let requested = request.win_condition.unwrap_or(WinCondition::CardsFinish);
    let creator_name = request.creator_name.map(|name| name.trim().to_string());
    Ok(CreateRoomConfig {
        win_condition: validate_win_condition(requested)?,
        creator_name: creator_name.filter(|name| !name.is_empty()),
    })
}

// main object for server
#[derive(Debug, Clone)]
struct ServerState {
    rooms: DashMap<String, Arc<Room>>,
    base_deck: Arc<Vec<String>>,
    cards: Arc<HashMap<String, PathBuf>>,
    card_content_type: &'static str,
    default_win_points_target: u16,
}

impl ServerState {
    fn new() -> Result<Self> {
        cleanup_legacy_generated_cards()?;

        let config = NormalizationConfig::from_env()?;
        let default_win_points_target = parse_default_win_points_from_env();
        let extra_image_dirs = get_extra_image_dirs();
        let disable_builtin_images = env_is_y(DISABLE_BUILTIN_IMAGES_ENV);
        let sniff_extensionless_images = env_is_y(SNIFF_EXTENSIONLESS_IMAGES_ENV);

        let loaded_cards = load_cards(
            &config,
            &extra_image_dirs,
            disable_builtin_images,
            sniff_extensionless_images,
        )?;

        println!(
            "Loaded {} cards ({} built-in, {} extra, {} failed; builtins {}; extensionless sniff {}; ratio {}:{}, long side {}; cache format {}; cache {}; default points target {})",
            loaded_cards.deck.len(),
            loaded_cards.loaded_builtin,
            loaded_cards.loaded_extra,
            loaded_cards.failed_sources,
            if disable_builtin_images { "disabled" } else { "enabled" },
            if sniff_extensionless_images {
                "enabled"
            } else {
                "disabled"
            },
            config.ratio_width,
            config.ratio_height,
            config.long_side,
            config.cache_format.env_value(),
            config.cards_cache_dir.display(),
            default_win_points_target
        );

        Ok(ServerState {
            rooms: DashMap::new(),
            base_deck: Arc::new(loaded_cards.deck),
            cards: Arc::new(loaded_cards.cards),
            card_content_type: config.cache_format.mime_type(),
            default_win_points_target,
        })
    }

    async fn create_room(
        &self,
        win_condition: WinCondition,
        creator_name: Option<String>,
    ) -> Result<ServerMsg> {
        let mut room_id = generate_room_id(4);

        while (self.get_room(&room_id)).is_some() {
            room_id = generate_room_id(4);
        }

        let room = Room::new(
            &room_id,
            self.base_deck.clone(),
            win_condition,
            creator_name,
        );
        let msg = room.get_room_state().await;
        self.rooms.insert(room_id.clone(), Arc::new(room));
        Ok(msg)
    }

    async fn join_room(&self, room_id: &str, socket: &mut WebSocket, name: &str) -> Result<()> {
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

    async fn run_room_maintenance(&self) {
        let rooms: Vec<Arc<Room>> = self
            .rooms
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        for room in rooms {
            room.run_maintenance().await;
        }
    }
}

async fn garbage_collect(state: Arc<ServerState>) {
    loop {
        tokio::time::sleep(GARBAGE_COLLECT_INTERVAL).await;
        state.garbage_collect();
    }
}

async fn room_maintenance(state: Arc<ServerState>) {
    loop {
        tokio::time::sleep(ROOM_MAINTENANCE_INTERVAL).await;
        state.run_room_maintenance().await;
    }
}

fn generate_room_id(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let letters = Uniform::new_inclusive(b'a', b'z');
    (0..length)
        .map(|_| letters.sample(&mut rng) as char)
        .collect()
}

#[tokio::main]
async fn main() {
    let state = Arc::new(ServerState::new().unwrap());

    tokio::spawn(garbage_collect(state.clone()));
    tokio::spawn(room_maintenance(state.clone()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/cards/:card_id", get(card_handler))
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

async fn card_handler(
    AxumPath(card_id): AxumPath<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    let Some(cache_path) = state.cards.get(&card_id).cloned() else {
        return (StatusCode::NOT_FOUND, "Card not found").into_response();
    };

    match tokio::fs::read(&cache_path).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, state.card_content_type),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            bytes,
        )
            .into_response(),
        Err(err) => {
            println!(
                "Warning: failed to read cached card image {}: {}",
                cache_path.display(),
                err
            );
            (StatusCode::NOT_FOUND, "Card image unavailable").into_response()
        }
    }
}

async fn create_room_handler(State(state): State<Arc<ServerState>>, body: Bytes) -> String {
    let room_config = match parse_create_room_win_condition(&body, state.default_win_points_target)
    {
        Ok(config) => config,
        Err(err) => {
            println!("Failed to parse create-room payload: {}", err);
            return serde_json::to_string(&room::ServerMsg::ErrorMsg(
                "Failed to create room".to_string(),
            ))
            .unwrap();
        }
    };

    let room = state
        .create_room(room_config.win_condition, room_config.creator_name)
        .await;

    match room {
        Ok(room_state) => serde_json::to_string(&room_state).unwrap(),
        Err(err) => {
            println!("Failed to create room: {}", err);
            serde_json::to_string(&room::ServerMsg::ErrorMsg(
                "Failed to create room".to_string(),
            ))
            .unwrap()
        }
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
