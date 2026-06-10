use anyhow::{anyhow, Context, Result};
use axum::{
    body::Bytes,
    extract::{
        ws::{Message as WsMessage, WebSocket},
        Json, Path as AxumPath, Query, State, WebSocketUpgrade,
    },
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType, GenericImageView};
use indicatif::{ProgressBar, ProgressStyle};
use notify::{EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, fs,
    io::BufWriter,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

mod avif;
mod most_beautiful_stats;
mod room;

use most_beautiful_stats::{MostBeautifulStatsResponse, MostBeautifulStatsStore};
use rand::distributions::{Distribution, Uniform};
use room::{
    canonical_member_name, get_time_s, hash_room_password, Room, ServerMsg, StellaWordPackPreset,
    WinCondition, MAX_MEMBER_NAME_LEN,
};

const GARBAGE_COLLECT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 20); // 20 minutes
const ROOM_MAINTENANCE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
const GC_ROOM_TIMEOUT_S: u64 = 60 * 60; // 1 hour

const BUILTIN_IMAGE_DIR: &str = "../static/assets/cards/";
const WORD_PACKS_DIR: &str = "../wordpacks";
const LEGACY_EXTRA_CARD_PREFIX: &str = "extra_dir__";

const EXTRA_IMAGE_DIRS_ENV: &str = "TALESPIN_EXTRA_IMAGE_DIRS";
const DISABLE_BUILTIN_IMAGES_ENV: &str = "TALESPIN_DISABLE_BUILTIN_IMAGES_P";
const SNIFF_EXTENSIONLESS_IMAGES_ENV: &str = "TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P";
const WATCH_IMAGE_ENV: &str = "TALESPIN_WATCH_IMAGE_P";
const CACHE_DIR_ENV: &str = "TALESPIN_CACHE_DIR";
const CARD_ASPECT_RATIO_ENV: &str = "TALESPIN_CARD_ASPECT_RATIO";
const CARD_LONG_SIDE_ENV: &str = "TALESPIN_CARD_LONG_SIDE";
const CARD_CACHE_FORMAT_ENV: &str = "TALESPIN_CARD_CACHE_FORMAT";
const CARD_AVIF_ENCODER_ENV: &str = "TALESPIN_CARD_AVIF_ENCODER";
const CARD_AVIF_THREADS_ENV: &str = "TALESPIN_CARD_AVIF_THREADS";
const VALIDATE_CACHE_HITS_ENV: &str = "TALESPIN_VALIDATE_CACHE_HITS_P";
const PRODUCTION_ENV: &str = "TALESPIN_PRODUCTION_P";
const SHOW_IMAGE_PATH_ENV: &str = "TALESPIN_IMAGES_SHOW_PATH_P";
const DEFAULT_WIN_POINTS_ENV: &str = "TALESPIN_DEFAULT_WIN_POINTS";
const MAX_MEMBERS_ENV: &str = "TALESPIN_MAX_MEMBERS";
const MB_STATS_DB_PATH_ENV: &str = "TALESPIN_MB_STATS_DB_PATH";

const DEFAULT_CARD_ASPECT_RATIO: &str = "2:3";
const DEFAULT_CARD_LONG_SIDE: u32 = 1536;
const DEFAULT_WIN_POINTS: u16 = 10;
const DEFAULT_MAX_MEMBERS: usize = 64;
const DEFAULT_CACHE_DIR: &str = "~/.cache/talespin";
const CACHE_SUBDIR_CARDS: &str = "cards";
const DEFAULT_MB_STATS_DB_FILENAME: &str = "most_beautiful_stats.sqlite3";

const CARD_JPEG_QUALITY: u8 = 90;
const NORMALIZATION_PIPELINE_VERSION: &str = "v1";
const DEFAULT_VALIDATE_CACHE_HITS: bool = true;
const DEFAULT_CARD_AVIF_ENCODER: avif::EncoderBackend = avif::EncoderBackend::Native;
const DEFAULT_CARD_AVIF_THREADS: avif::ThreadSetting = avif::ThreadSetting::Auto;

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
    avif_encoder_backend: avif::EncoderBackend,
    avif_threads: avif::ThreadSetting,
    validate_cache_hits: bool,
    production_mode: bool,
    cards_cache_dir: PathBuf,
}

impl NormalizationConfig {
    fn from_env() -> Result<Self> {
        let (ratio_width, ratio_height) = parse_ratio_from_env();
        let long_side = parse_long_side_from_env();
        let cache_format = parse_cache_image_format_from_env();
        let avif_encoder_backend = parse_avif_encoder_backend_from_env();
        let avif_threads = parse_avif_threads_from_env();
        let validate_cache_hits = parse_validate_cache_hits_from_env();
        let production_mode = env_is_truthy(PRODUCTION_ENV);

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
            avif_encoder_backend,
            avif_threads,
            validate_cache_hits,
            production_mode,
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

    fn should_validate_cache_hits(&self) -> bool {
        if matches!(self.cache_format, CacheImageFormat::Avif) && !self.production_mode {
            return false;
        }
        self.validate_cache_hits
    }

    fn cache_validation_status_label(&self) -> &'static str {
        if self.should_validate_cache_hits() {
            "enabled"
        } else if matches!(self.cache_format, CacheImageFormat::Avif) && !self.production_mode {
            "disabled (dev mode avif shortcut)"
        } else {
            "disabled"
        }
    }
}

#[derive(Debug)]
struct LoadedCards {
    deck: Vec<String>,
    cards: HashMap<String, PathBuf>,
    original_cards: HashMap<String, OriginalCardInfo>,
    sources: HashMap<PathBuf, SourceCardEntry>,
    loaded_builtin: usize,
    loaded_extra: usize,
    failed_sources: usize,
}

#[derive(Debug, Clone)]
struct SourceCardEntry {
    card_id: String,
    cache_path: PathBuf,
    original_info: OriginalCardInfo,
}

#[derive(Debug, Clone)]
struct OriginalCardInfo {
    path: PathBuf,
    relative_source_path: String,
    content_type: &'static str,
}

#[derive(Debug, Clone, Copy)]
enum SourceKind {
    Builtin,
    Extra,
}

impl SourceKind {
    fn label(self) -> &'static str {
        match self {
            Self::Builtin => "built-in",
            Self::Extra => "extra",
        }
    }
}

fn load_word_pack_presets(dir: &Path) -> Result<Vec<StellaWordPackPreset>> {
    let mut presets = Vec::new();
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read word-pack directory {}", dir.display()))?;

    for entry in entries {
        let entry =
            entry.with_context(|| format!("Failed to read an entry from {}", dir.display()))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
            continue;
        }

        let raw_words = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read word pack {}", path.display()))?;
        let words = Room::parse_stella_word_pack(&raw_words);
        if words.is_empty() {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| anyhow!("Invalid word-pack filename {}", path.display()))?
            .to_string();
        presets.push(StellaWordPackPreset { name, words });
    }

    presets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    if presets.is_empty() {
        return Err(anyhow!(
            "No non-empty .txt word packs were found in {}",
            dir.display()
        ));
    }

    Ok(presets)
}

fn choose_default_word_pack(presets: &[StellaWordPackPreset]) -> Result<&StellaWordPackPreset> {
    presets
        .iter()
        .find(|preset| preset.name == "Resonance_Persian_1")
        .or_else(|| presets.first())
        .ok_or_else(|| anyhow!("No word-pack presets available"))
}

fn create_normalization_progress(total_sources: usize) -> ProgressBar {
    if total_sources == 0 {
        return ProgressBar::hidden();
    }

    let progress = ProgressBar::new(total_sources as u64);
    let style = ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({percent}%) eta {eta_precise} {msg}",
    )
    .expect("progress template must be valid")
    .progress_chars("=> ");
    progress.set_style(style);
    progress.enable_steady_tick(std::time::Duration::from_millis(120));
    progress
}

fn source_progress_message(kind: SourceKind, source: &Path) -> String {
    let short_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|| source.display().to_string());
    format!("{}: {}", kind.label(), short_name)
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

fn parse_avif_encoder_backend_from_env() -> avif::EncoderBackend {
    if let Ok(raw) = env::var(CARD_AVIF_ENCODER_ENV) {
        if let Some(backend) = avif::EncoderBackend::from_env_value(&raw) {
            return backend;
        }

        println!(
            "Warning: invalid {}='{}'; using default {}",
            CARD_AVIF_ENCODER_ENV,
            raw,
            DEFAULT_CARD_AVIF_ENCODER.env_value()
        );
    }

    DEFAULT_CARD_AVIF_ENCODER
}

fn parse_avif_threads_from_env() -> avif::ThreadSetting {
    if let Ok(raw) = env::var(CARD_AVIF_THREADS_ENV) {
        if let Some(threads) = avif::ThreadSetting::from_env_value(&raw) {
            return threads;
        }

        println!(
            "Warning: invalid {}='{}'; using default {}",
            CARD_AVIF_THREADS_ENV,
            raw,
            DEFAULT_CARD_AVIF_THREADS.env_value()
        );
    }

    DEFAULT_CARD_AVIF_THREADS
}

fn parse_validate_cache_hits_from_env() -> bool {
    if let Ok(raw) = env::var(VALIDATE_CACHE_HITS_ENV) {
        match raw.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" | "true" | "1" => return true,
            "n" | "no" | "false" | "0" => return false,
            _ => {
                println!(
                    "Warning: invalid {}='{}'; using default {}",
                    VALIDATE_CACHE_HITS_ENV,
                    raw,
                    if DEFAULT_VALIDATE_CACHE_HITS {
                        "y"
                    } else {
                        "n"
                    }
                );
            }
        }
    }

    DEFAULT_VALIDATE_CACHE_HITS
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

fn parse_max_members_from_env() -> usize {
    if let Ok(raw) = env::var(MAX_MEMBERS_ENV) {
        if let Ok(value) = raw.trim().parse::<usize>() {
            if value >= 3 {
                return value;
            }
        }

        println!(
            "Warning: invalid {}='{}'; using default {}",
            MAX_MEMBERS_ENV, raw, DEFAULT_MAX_MEMBERS
        );
    }

    DEFAULT_MAX_MEMBERS
}

fn cache_root_dir_from_env() -> PathBuf {
    env::var(CACHE_DIR_ENV)
        .map(|v| expand_home(v.trim()))
        .unwrap_or_else(|_| expand_home(DEFAULT_CACHE_DIR))
}

fn parse_mb_stats_db_path_from_env() -> PathBuf {
    env::var(MB_STATS_DB_PATH_ENV)
        .map(|value| expand_home(value.trim()))
        .unwrap_or_else(|_| cache_root_dir_from_env().join(DEFAULT_MB_STATS_DB_FILENAME))
}

fn parse_bool_env_value(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => Some(true),
        "n" | "no" | "false" | "0" => Some(false),
        _ => None,
    }
}

fn env_is_truthy(key: &str) -> bool {
    env::var(key)
        .ok()
        .and_then(|v| parse_bool_env_value(&v))
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

fn source_image_content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
}

fn relative_source_path(source_root: &Path, source: &Path) -> String {
    source
        .strip_prefix(source_root)
        .unwrap_or(source)
        .to_string_lossy()
        .replace('\\', "/")
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
                Err(err)
                    if file_type.is_symlink() && err.kind() == std::io::ErrorKind::NotFound =>
                {
                    continue;
                }
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

fn validate_cached_image(
    cache_path: &Path,
    expected_width: u32,
    expected_height: u32,
) -> Result<()> {
    let cache_bytes = fs::read(cache_path)
        .with_context(|| format!("Failed to read cached image {}", cache_path.display()))?;
    let cached_image = image::load_from_memory(&cache_bytes)
        .with_context(|| format!("Failed to decode cached image {}", cache_path.display()))?;
    let (cached_width, cached_height) = cached_image.dimensions();

    if cached_width != expected_width || cached_height != expected_height {
        return Err(anyhow!(
            "Cached image {} has dimensions {}x{}, expected {}x{}",
            cache_path.display(),
            cached_width,
            cached_height,
            expected_width,
            expected_height
        ));
    }

    Ok(())
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
        CacheImageFormat::Avif => {
            avif::encoding_descriptor(config.avif_encoder_backend, config.avif_threads)
        }
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

    let mut should_rebuild_cache = !cache_path.exists();
    if !should_rebuild_cache && config.should_validate_cache_hits() {
        if let Err(err) = validate_cached_image(&cache_path, output_width, output_height) {
            println!(
                "Warning: cached image {} is invalid/corrupt: {}. Rebuilding.",
                cache_path.display(),
                err
            );
            if let Err(remove_err) = fs::remove_file(&cache_path) {
                println!(
                    "Warning: failed to remove invalid cache file {}: {}",
                    cache_path.display(),
                    remove_err
                );
            }
            should_rebuild_cache = true;
        }
    }

    if should_rebuild_cache {
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

        let cropped = source_image.crop_imm(crop_x, crop_y, crop_width, crop_height);
        let resized = cropped.resize_exact(output_width, output_height, FilterType::Lanczos3);

        let file = fs::File::create(&cache_path)
            .with_context(|| format!("Failed to create cache file {}", cache_path.display()))?;
        let mut writer = BufWriter::new(file);
        match config.cache_format {
            CacheImageFormat::Avif => {
                avif::encode_dynamic_image(
                    &resized,
                    &mut writer,
                    &cache_path,
                    config.avif_encoder_backend,
                    config.avif_threads,
                )?;
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
    let builtin_root = fs::canonicalize(Path::new(BUILTIN_IMAGE_DIR)).ok();
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
        let source_root = fs::canonicalize(dir).unwrap_or_else(|_| dir.clone());
        for source in collect_image_files_recursive(dir, false, sniff_extensionless_images)? {
            extra_sources.push((source_root.clone(), source));
        }
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
    let mut original_cards = HashMap::new();
    let mut sources = HashMap::new();
    let mut loaded_builtin = 0usize;
    let mut loaded_extra = 0usize;
    let mut failed_sources = 0usize;
    let mut sources_to_process = Vec::with_capacity(builtin_sources.len() + extra_sources.len());

    for source in builtin_sources {
        if seen_sources.insert(source.clone()) {
            let source_root = builtin_root
                .clone()
                .unwrap_or_else(|| Path::new(BUILTIN_IMAGE_DIR).to_path_buf());
            sources_to_process.push((SourceKind::Builtin, source_root, source));
        }
    }

    for (source_root, source) in extra_sources {
        if seen_sources.insert(source.clone()) {
            sources_to_process.push((SourceKind::Extra, source_root, source));
        }
    }

    let total_sources = sources_to_process.len();
    if total_sources > 0 {
        println!(
            "Preparing card caches: {} and generating {} cache file{} from {} source image{}.",
            if config.should_validate_cache_hits() {
                "checking existing cache entries for corruption"
            } else {
                "skipping cache-hit corruption checks"
            },
            config.cache_format.env_value(),
            if total_sources == 1 { "" } else { "s" },
            total_sources,
            if total_sources == 1 { "" } else { "s" }
        );
    }

    let progress = create_normalization_progress(total_sources);
    progress.set_message("warming up...");

    for (kind, source_root, source) in sources_to_process {
        progress.set_message(source_progress_message(kind, &source));

        match normalize_source_to_cache(&source, config) {
            Ok((card_id, cache_path)) => {
                let original_info = OriginalCardInfo {
                    path: source.clone(),
                    relative_source_path: relative_source_path(&source_root, &source),
                    content_type: source_image_content_type(&source),
                };
                sources.insert(
                    source.clone(),
                    SourceCardEntry {
                        card_id: card_id.clone(),
                        cache_path: cache_path.clone(),
                        original_info: original_info.clone(),
                    },
                );
                if seen_card_ids.insert(card_id.clone()) {
                    deck.push(card_id.clone());
                    cards.insert(card_id, cache_path);
                    original_cards.insert(
                        deck.last().expect("card id was just pushed").clone(),
                        original_info,
                    );
                    match kind {
                        SourceKind::Builtin => loaded_builtin += 1,
                        SourceKind::Extra => loaded_extra += 1,
                    }
                }
            }
            Err(err) => {
                failed_sources += 1;
                println!(
                    "Warning: failed to normalize {} image {}: {}",
                    kind.label(),
                    source.display(),
                    err
                );
            }
        }

        progress.inc(1);
    }

    progress.finish_with_message(format!(
        "Normalization complete ({} unique cards, {} failed sources)",
        deck.len(),
        failed_sources
    ));

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
        original_cards,
        sources,
        loaded_builtin,
        loaded_extra,
        failed_sources,
    })
}

#[derive(Debug, Clone)]
struct CardCatalog {
    active_deck: Vec<String>,
    active_cards: HashMap<String, PathBuf>,
    active_original_cards: HashMap<String, OriginalCardInfo>,
    serving_cards: HashMap<String, PathBuf>,
    serving_original_cards: HashMap<String, OriginalCardInfo>,
    source_index: HashMap<PathBuf, SourceCardEntry>,
}

impl CardCatalog {
    fn from_loaded_cards(loaded_cards: LoadedCards) -> Self {
        Self {
            active_deck: loaded_cards.deck,
            active_cards: loaded_cards.cards.clone(),
            active_original_cards: loaded_cards.original_cards.clone(),
            serving_cards: loaded_cards.cards,
            serving_original_cards: loaded_cards.original_cards,
            source_index: loaded_cards.sources,
        }
    }

    fn active_deck_snapshot(&self) -> Arc<Vec<String>> {
        Arc::new(self.active_deck.clone())
    }

    fn upsert_extra_source(
        &mut self,
        source_root: &Path,
        source: &Path,
        config: &NormalizationConfig,
        sniff_extensionless_images: bool,
    ) -> Result<Option<String>> {
        let source_key = canonical_source_key(source);
        if !source_key.is_file() || !is_supported_image(&source_key, sniff_extensionless_images) {
            self.remove_extra_source(&source_key);
            return Ok(None);
        }

        let (card_id, cache_path) = normalize_source_to_cache(&source_key, config)?;
        let original_info = OriginalCardInfo {
            path: source_key.clone(),
            relative_source_path: relative_source_path(source_root, &source_key),
            content_type: source_image_content_type(&source_key),
        };

        if let Some(previous) = self.source_index.insert(
            source_key,
            SourceCardEntry {
                card_id: card_id.clone(),
                cache_path: cache_path.clone(),
                original_info: original_info.clone(),
            },
        ) {
            self.serving_cards
                .entry(previous.card_id.clone())
                .or_insert(previous.cache_path);
            self.serving_original_cards
                .entry(previous.card_id)
                .or_insert(previous.original_info);
        }

        self.serving_cards.insert(card_id.clone(), cache_path);
        self.serving_original_cards
            .insert(card_id.clone(), original_info);
        self.rebuild_active_catalog();

        Ok(Some(card_id))
    }

    fn remove_extra_source(&mut self, source: &Path) -> Option<String> {
        let source_key = canonical_source_key(source);
        let removed = self.source_index.remove(&source_key)?;
        self.rebuild_active_catalog();
        Some(removed.card_id)
    }

    fn rebuild_active_catalog(&mut self) {
        self.active_cards.clear();
        self.active_original_cards.clear();
        for entry in self.source_index.values() {
            self.active_cards
                .entry(entry.card_id.clone())
                .or_insert_with(|| entry.cache_path.clone());
            self.active_original_cards
                .entry(entry.card_id.clone())
                .or_insert_with(|| entry.original_info.clone());
        }
        self.active_deck = self.active_cards.keys().cloned().collect();
        self.active_deck.sort();
    }
}

fn canonical_source_key(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        path.parent()
            .and_then(|parent| fs::canonicalize(parent).ok())
            .and_then(|parent| path.file_name().map(|name| parent.join(name)))
            .unwrap_or_else(|| path.to_path_buf())
    })
}

#[derive(Debug, Deserialize)]
struct CreateRoomRequest {
    win_condition: Option<WinCondition>,
    creator_name: Option<String>,
    password: Option<String>,
}

#[derive(Debug)]
struct CreateRoomConfig {
    win_condition: WinCondition,
    creator_name: Option<String>,
    password: Option<String>,
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
        WinCondition::FixedRounds { target_rounds } => {
            if target_rounds == 0 {
                return Err(anyhow!("target_rounds must be >= 1"));
            }
            Ok(WinCondition::FixedRounds { target_rounds })
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
            win_condition: room::default_win_condition_for_game_mode(room::GameMode::DixitPlus),
            creator_name: None,
            password: None,
        });
    }

    let request: CreateRoomRequest =
        serde_json::from_slice(body).context("Failed to parse create-room request payload")?;
    let requested = request
        .win_condition
        .unwrap_or(room::default_win_condition_for_game_mode(
            room::GameMode::DixitPlus,
        ));
    let creator_name = request.creator_name.map(|name| name.trim().to_string());
    let password = request.password.map(|password| password.trim().to_string());
    let password = password.filter(|password| !password.is_empty());
    if password
        .as_ref()
        .map(|password| password.len() > 200)
        .unwrap_or(false)
    {
        return Err(anyhow!("room password too long"));
    }
    Ok(CreateRoomConfig {
        win_condition: validate_win_condition(requested)?,
        creator_name: creator_name.filter(|name| !name.is_empty()),
        password,
    })
}

// main object for server
#[derive(Debug, Clone)]
struct ServerState {
    rooms: DashMap<String, Arc<Room>>,
    catalog: Arc<RwLock<CardCatalog>>,
    card_content_type: &'static str,
    show_image_source_paths: bool,
    normalization_config: NormalizationConfig,
    extra_image_dirs: Arc<Vec<PathBuf>>,
    sniff_extensionless_images: bool,
    most_beautiful_stats: Arc<MostBeautifulStatsStore>,
    default_stella_word_pack: Arc<Vec<String>>,
    stella_word_pack_presets: Arc<Vec<StellaWordPackPreset>>,
    default_win_points_target: u16,
    max_members: usize,
}

impl ServerState {
    fn new() -> Result<Self> {
        cleanup_legacy_generated_cards()?;

        let config = NormalizationConfig::from_env()?;
        let default_win_points_target = parse_default_win_points_from_env();
        let max_members = parse_max_members_from_env();
        let most_beautiful_stats = Arc::new(MostBeautifulStatsStore::new(
            parse_mb_stats_db_path_from_env(),
        )?);
        let word_pack_presets = load_word_pack_presets(Path::new(WORD_PACKS_DIR))?;
        let default_word_pack = choose_default_word_pack(&word_pack_presets)?;
        let extra_image_dirs = get_extra_image_dirs();
        let disable_builtin_images = env_is_truthy(DISABLE_BUILTIN_IMAGES_ENV);
        let sniff_extensionless_images = env_is_truthy(SNIFF_EXTENSIONLESS_IMAGES_ENV);

        let loaded_cards = load_cards(
            &config,
            &extra_image_dirs,
            disable_builtin_images,
            sniff_extensionless_images,
        )?;
        most_beautiful_stats.register_card_paths(get_time_s(), &loaded_cards.cards)?;
        let loaded_card_count = loaded_cards.deck.len();
        let loaded_builtin = loaded_cards.loaded_builtin;
        let loaded_extra = loaded_cards.loaded_extra;
        let failed_sources = loaded_cards.failed_sources;

        println!(
            "Loaded {} cards ({} built-in, {} extra, {} failed; builtins {}; extensionless sniff {}; ratio {}:{}, long side {}; cache format {}; avif encoder {}; avif threads {}; cache validation {}; cache {}; default word pack {}; loaded word packs {}; default points target {}; max members {})",
            loaded_card_count,
            loaded_builtin,
            loaded_extra,
            failed_sources,
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
            config.avif_encoder_backend.env_value(),
            config.avif_threads.env_value(),
            config.cache_validation_status_label(),
            config.cards_cache_dir.display(),
            default_word_pack.name,
            word_pack_presets.len(),
            default_win_points_target,
            max_members
        );

        Ok(ServerState {
            rooms: DashMap::new(),
            catalog: Arc::new(RwLock::new(CardCatalog::from_loaded_cards(loaded_cards))),
            card_content_type: config.cache_format.mime_type(),
            show_image_source_paths: !config.production_mode && env_is_truthy(SHOW_IMAGE_PATH_ENV),
            normalization_config: config,
            extra_image_dirs: Arc::new(extra_image_dirs),
            sniff_extensionless_images,
            most_beautiful_stats,
            default_stella_word_pack: Arc::new(default_word_pack.words.clone()),
            stella_word_pack_presets: Arc::new(word_pack_presets),
            default_win_points_target,
            max_members,
        })
    }

    async fn create_room(
        &self,
        win_condition: WinCondition,
        creator_name: Option<String>,
        room_password: Option<String>,
    ) -> Result<ServerMsg> {
        let mut room_id = generate_room_id(4);

        while (self.get_room(&room_id)).is_some() {
            room_id = generate_room_id(4);
        }

        let room_password_hash = room_password
            .as_ref()
            .map(|password| hash_room_password(&room_id, password));
        let base_deck = self.catalog.read().await.active_deck_snapshot();
        let room = Room::new(
            &room_id,
            base_deck,
            win_condition,
            creator_name,
            self.max_members,
            room_password_hash,
            self.most_beautiful_stats.clone(),
            self.default_stella_word_pack.clone(),
            self.stella_word_pack_presets.clone(),
        );
        let msg = room.get_room_state().await;
        self.rooms.insert(room_id.clone(), Arc::new(room));
        Ok(msg)
    }

    async fn join_room(
        &self,
        room_id: &str,
        socket: &mut WebSocket,
        name: &str,
        token: &str,
        room_password: Option<&str>,
    ) -> Result<()> {
        if let Some(room) = self.get_room(room_id) {
            room.on_connection(socket, name, token, room_password).await;
        } else {
            socket.send(ServerMsg::InvalidRoomId {}.into()).await?;
            return Ok(());
        }

        Ok(())
    }

    fn get_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.get(room_id).map(|r| r.value().clone())
    }

    fn watch_images_enabled(&self) -> bool {
        env_is_truthy(WATCH_IMAGE_ENV) && !self.extra_image_dirs.is_empty()
    }

    async fn handle_image_path_changed(&self, path: PathBuf) {
        let Some(source_root) = self.extra_image_root_for_path(&path) else {
            return;
        };

        let changed_path = canonical_source_key(&path);
        if changed_path.exists() {
            let mut catalog = self.catalog.write().await;
            match catalog.upsert_extra_source(
                &source_root,
                &changed_path,
                &self.normalization_config,
                self.sniff_extensionless_images,
            ) {
                Ok(Some(card_id)) => {
                    if let Some(cache_path) = catalog.serving_cards.get(&card_id).cloned() {
                        let mut changed_cards = HashMap::new();
                        changed_cards.insert(card_id.clone(), cache_path);
                        if let Err(err) = self
                            .most_beautiful_stats
                            .register_card_paths(get_time_s(), &changed_cards)
                        {
                            println!(
                                "Warning: failed to register updated card path for {}: {}",
                                changed_path.display(),
                                err
                            );
                        }
                    }
                    println!(
                        "Updated watched image {} as card {}",
                        changed_path.display(),
                        card_id
                    );
                }
                Ok(None) => {}
                Err(err) => {
                    println!(
                        "Warning: failed to update watched image {}: {}",
                        changed_path.display(),
                        err
                    );
                }
            }
        } else {
            let mut catalog = self.catalog.write().await;
            if let Some(card_id) = catalog.remove_extra_source(&changed_path) {
                println!(
                    "Removed watched image {} from active deck (old card {} remains servable if cached)",
                    changed_path.display(),
                    card_id
                );
            }
        }
    }

    fn extra_image_root_for_path(&self, path: &Path) -> Option<PathBuf> {
        let canonical_path = canonical_source_key(path);
        self.extra_image_dirs
            .iter()
            .map(|dir| canonical_source_key(dir))
            .filter(|root| canonical_path.starts_with(root))
            .max_by_key(|root| root.components().count())
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

async fn watch_extra_images(state: Arc<ServerState>) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>();
    let mut watcher = match notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    }) {
        Ok(watcher) => watcher,
        Err(err) => {
            println!("Warning: failed to start image watcher: {}", err);
            return;
        }
    };

    let mut watched_any = false;
    for dir in state.extra_image_dirs.iter() {
        match watcher.watch(dir, RecursiveMode::Recursive) {
            Ok(()) => {
                watched_any = true;
                println!("Watching extra image directory {}", dir.display());
            }
            Err(err) => {
                println!(
                    "Warning: failed to watch extra image directory {}: {}",
                    dir.display(),
                    err
                );
            }
        }
    }

    if !watched_any {
        println!("Warning: image watch requested but no extra image directories could be watched");
        return;
    }

    let mut pending_paths = HashSet::<PathBuf>::new();
    let debounce = tokio::time::sleep(std::time::Duration::from_secs(24 * 60 * 60));
    tokio::pin!(debounce);

    loop {
        tokio::select! {
            maybe_event = rx.recv() => {
                let Some(event_result) = maybe_event else {
                    break;
                };

                match event_result {
                    Ok(event) => {
                        if is_relevant_image_event(&event.kind) {
                            for path in event.paths {
                                pending_paths.insert(path);
                            }
                            debounce.as_mut().reset(tokio::time::Instant::now() + std::time::Duration::from_millis(500));
                        }
                    }
                    Err(err) => {
                        println!("Warning: image watcher error: {}", err);
                    }
                }
            }
            () = &mut debounce, if !pending_paths.is_empty() => {
                let paths = pending_paths.drain().collect::<Vec<_>>();
                for path in paths {
                    if path.is_dir() {
                        if let Err(err) = watcher.watch(&path, RecursiveMode::Recursive) {
                            println!(
                                "Warning: failed to watch new image directory {}: {}",
                                path.display(),
                                err
                            );
                        }
                        continue;
                    }
                    state.handle_image_path_changed(path).await;
                }
            }
        }
    }
}

fn is_relevant_image_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
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
    if state.watch_images_enabled() {
        tokio::spawn(watch_extra_images(state.clone()));
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/cards/:card_id", get(card_handler))
        .route("/cards/:card_id/source-info", get(card_source_info_handler))
        .route("/create", post(create_room_handler))
        .route("/exists", post(exists_handler))
        .route("/stats", get(stats_handler))
        .route("/most-beautiful-stats", get(most_beautiful_stats_handler))
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
    if let Some(original_card_id) = card_id.strip_suffix("_original") {
        let original_info = {
            let catalog = state.catalog.read().await;
            catalog
                .serving_original_cards
                .get(original_card_id)
                .cloned()
        };
        let Some(original_info) = original_info else {
            return (StatusCode::NOT_FOUND, "Card not found").into_response();
        };

        return match tokio::fs::read(&original_info.path).await {
            Ok(bytes) => (
                [
                    (header::CONTENT_TYPE, original_info.content_type),
                    (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                ],
                bytes,
            )
                .into_response(),
            Err(err) => {
                println!(
                    "Warning: failed to read original card image {}: {}",
                    original_info.path.display(),
                    err
                );
                (StatusCode::NOT_FOUND, "Original card image unavailable").into_response()
            }
        };
    }

    let cache_path = {
        let catalog = state.catalog.read().await;
        catalog.serving_cards.get(&card_id).cloned()
    };
    let Some(cache_path) = cache_path else {
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

#[derive(Debug, Serialize)]
struct CardSourceInfoResponse {
    relative_source_path: String,
}

async fn card_source_info_handler(
    AxumPath(card_id): AxumPath<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    if !state.show_image_source_paths {
        return (StatusCode::NOT_FOUND, "Card source info unavailable").into_response();
    }

    let original_info = {
        let catalog = state.catalog.read().await;
        catalog.serving_original_cards.get(&card_id).cloned()
    };
    let Some(original_info) = original_info else {
        return (StatusCode::NOT_FOUND, "Card not found").into_response();
    };

    Json(CardSourceInfoResponse {
        relative_source_path: original_info.relative_source_path,
    })
    .into_response()
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
        .create_room(
            room_config.win_condition,
            room_config.creator_name,
            room_config.password,
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_image_bool_parser_accepts_true_false_spellings() {
        assert_eq!(parse_bool_env_value("true"), Some(true));
        assert_eq!(parse_bool_env_value("y"), Some(true));
        assert_eq!(parse_bool_env_value("yes"), Some(true));
        assert_eq!(parse_bool_env_value("1"), Some(true));
        assert_eq!(parse_bool_env_value("false"), Some(false));
        assert_eq!(parse_bool_env_value("n"), Some(false));
        assert_eq!(parse_bool_env_value("no"), Some(false));
        assert_eq!(parse_bool_env_value("0"), Some(false));
        assert_eq!(parse_bool_env_value("sometimes"), None);
    }

    #[test]
    fn watch_image_catalog_processes_changed_files_incrementally() -> Result<()> {
        let temp_dir = test_temp_dir("watch-image-catalog");
        let source_dir = temp_dir.join("sources");
        let cache_dir = temp_dir.join("cache");
        fs::create_dir_all(&source_dir)?;
        fs::create_dir_all(&cache_dir)?;
        let config = test_normalization_config(cache_dir);

        let first_source = source_dir.join("first.png");
        write_test_image(&first_source, [255, 0, 0])?;
        let loaded = load_cards(&config, &[source_dir.clone()], true, false)?;
        let mut catalog = CardCatalog::from_loaded_cards(loaded);
        assert_eq!(catalog.active_deck_snapshot().len(), 1);

        let first_key = canonical_source_key(&first_source);
        let first_id = catalog
            .source_index
            .get(&first_key)
            .expect("first source should be indexed")
            .card_id
            .clone();

        let second_source = source_dir.join("second.png");
        write_test_image(&second_source, [0, 255, 0])?;
        catalog.upsert_extra_source(&source_dir, &second_source, &config, false)?;
        assert_eq!(
            catalog.active_deck_snapshot().len(),
            2,
            "adding one changed file should add one active card"
        );

        write_test_image(&first_source, [0, 0, 255])?;
        catalog.upsert_extra_source(&source_dir, &first_source, &config, false)?;
        let updated_first_id = catalog
            .source_index
            .get(&first_key)
            .expect("first source should still be indexed")
            .card_id
            .clone();
        assert_ne!(
            first_id, updated_first_id,
            "modifying one source should replace that source's active card id"
        );
        assert!(
            catalog.serving_cards.contains_key(&first_id),
            "old modified card ids should remain servable for existing rooms"
        );
        assert!(
            !catalog.active_deck_snapshot().contains(&first_id),
            "old modified card ids should leave the active deck for new rooms"
        );

        let second_key = canonical_source_key(&second_source);
        let second_id = catalog
            .source_index
            .get(&second_key)
            .expect("second source should be indexed")
            .card_id
            .clone();
        fs::remove_file(&second_source)?;
        assert_eq!(
            canonical_source_key(&second_source),
            second_key,
            "deleted source paths should still map to the same canonical source key"
        );
        catalog.remove_extra_source(&second_source);
        assert!(
            !catalog.active_deck_snapshot().contains(&second_id),
            "deleted files should leave the active deck for new rooms"
        );
        assert!(
            catalog.serving_cards.contains_key(&second_id),
            "deleted card ids should remain servable from cache for existing rooms"
        );

        Ok(())
    }

    #[test]
    fn watch_image_catalog_keeps_existing_room_deck_snapshots() -> Result<()> {
        let temp_dir = test_temp_dir("watch-image-room-snapshot");
        let source_dir = temp_dir.join("sources");
        let cache_dir = temp_dir.join("cache");
        fs::create_dir_all(&source_dir)?;
        fs::create_dir_all(&cache_dir)?;
        let config = test_normalization_config(cache_dir);

        let first_source = source_dir.join("first.png");
        write_test_image(&first_source, [255, 0, 0])?;
        let loaded = load_cards(&config, &[source_dir.clone()], true, false)?;
        let mut catalog = CardCatalog::from_loaded_cards(loaded);
        let existing_room_deck = catalog.active_deck_snapshot();

        let second_source = source_dir.join("second.png");
        write_test_image(&second_source, [0, 255, 0])?;
        catalog.upsert_extra_source(&source_dir, &second_source, &config, false)?;
        let new_room_deck = catalog.active_deck_snapshot();

        assert_eq!(
            existing_room_deck.len(),
            1,
            "rooms that already cloned the deck should keep their original card list"
        );
        assert_eq!(
            new_room_deck.len(),
            2,
            "rooms created after the incremental update should see the new deck"
        );

        Ok(())
    }

    #[test]
    fn create_room_defaults_to_single_dixit_cycle_when_body_is_empty() {
        let config = parse_create_room_win_condition(&[], DEFAULT_WIN_POINTS).unwrap();
        assert_eq!(
            config.win_condition,
            room::default_win_condition_for_game_mode(room::GameMode::DixitPlus)
        );
    }

    #[test]
    fn create_room_defaults_to_single_dixit_cycle_when_win_condition_is_omitted() {
        let config =
            parse_create_room_win_condition(br#"{"creator_name":"host"}"#, DEFAULT_WIN_POINTS)
                .unwrap();
        assert_eq!(
            config.win_condition,
            room::default_win_condition_for_game_mode(room::GameMode::DixitPlus)
        );
    }

    fn test_temp_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "talespin-{name}-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("failed to clear stale test temp dir");
        }
        fs::create_dir_all(&dir).expect("failed to create test temp dir");
        dir
    }

    fn test_normalization_config(cards_cache_dir: PathBuf) -> NormalizationConfig {
        NormalizationConfig {
            ratio_width: 2,
            ratio_height: 3,
            long_side: 6,
            cache_format: CacheImageFormat::Jpeg,
            avif_encoder_backend: DEFAULT_CARD_AVIF_ENCODER,
            avif_threads: DEFAULT_CARD_AVIF_THREADS,
            validate_cache_hits: true,
            production_mode: true,
            cards_cache_dir,
        }
    }

    fn write_test_image(path: &Path, rgb: [u8; 3]) -> Result<()> {
        let image = image::RgbImage::from_pixel(4, 6, image::Rgb(rgb));
        image
            .save(path)
            .with_context(|| format!("failed to write test image {}", path.display()))
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

async fn most_beautiful_stats_handler(
    Query(query): Query<MostBeautifulStatsQuery>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<MostBeautifulStatsResponse>, StatusCode> {
    state
        .most_beautiful_stats
        .filtered_stats(query.room_id.as_deref(), query.games.unwrap_or(0))
        .map(Json)
        .map_err(|err| {
            println!(
                "Failed to load Most Beautiful stats from {}: {}",
                state.most_beautiful_stats.path().display(),
                err
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[derive(Debug, Deserialize)]
struct MostBeautifulStatsQuery {
    room_id: Option<String>,
    games: Option<usize>,
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
            if let room::ClientMsg::JoinRoom {
                room_id,
                name,
                token,
                room_password,
            } = msg
            {
                let name = canonical_member_name(&name);
                if name.len() > MAX_MEMBER_NAME_LEN {
                    socket
                        .send(room::ServerMsg::ErrorMsg("Name too long".to_string()).into())
                        .await?;
                    return Err(anyhow!("Name too long"));
                }
                if token.len() > 200 {
                    socket
                        .send(room::ServerMsg::ErrorMsg("Token too long".to_string()).into())
                        .await?;
                    return Err(anyhow!("Token too long"));
                }

                let room_password = room_password
                    .map(|password| password.trim().to_string())
                    .filter(|password| !password.is_empty());
                if room_password
                    .as_ref()
                    .map(|password| password.len() > 200)
                    .unwrap_or(false)
                {
                    socket
                        .send(
                            room::ServerMsg::ErrorMsg("Room password too long".to_string()).into(),
                        )
                        .await?;
                    return Err(anyhow!("Room password too long"));
                }

                state
                    .join_room(
                        &room_id.to_lowercase(),
                        socket,
                        name,
                        &token,
                        room_password.as_deref(),
                    )
                    .await?
            }
        }
    }

    Ok(())
}
