use anyhow::{anyhow, Context, Result};
use image::DynamicImage;
use libavif::{Encoder as NativeAvifEncoder, RgbPixels, YuvFormat};
use ravif::{Encoder as RavifEncoder, Img as RavifImg};
use rgb::FromSlice;
use std::{io::Write, path::Path};

pub const QUALITY: u8 = 80;
pub const SPEED: u8 = 6;

#[derive(Debug, Clone, Copy)]
pub enum EncoderBackend {
    Native,
    Ravif,
}

impl EncoderBackend {
    pub fn from_env_value(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "native" => Some(Self::Native),
            "ravif" => Some(Self::Ravif),
            _ => None,
        }
    }

    pub fn env_value(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Ravif => "ravif",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThreadSetting {
    Auto,
    Fixed(usize),
}

impl ThreadSetting {
    pub fn from_env_value(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.eq_ignore_ascii_case("auto") {
            return Some(Self::Auto);
        }

        let parsed = trimmed.parse::<usize>().ok()?;
        if parsed == 0 {
            return None;
        }

        Some(Self::Fixed(parsed))
    }

    pub fn env_value(self) -> String {
        match self {
            Self::Auto => "auto".to_string(),
            Self::Fixed(threads) => threads.to_string(),
        }
    }

    fn to_ravif_threads(self) -> Option<usize> {
        match self {
            Self::Auto => None,
            Self::Fixed(threads) => Some(threads),
        }
    }

    fn to_native_threads(self) -> Option<usize> {
        match self {
            Self::Auto => None,
            Self::Fixed(threads) => Some(threads),
        }
    }
}

pub fn encoding_descriptor(backend: EncoderBackend, threads: ThreadSetting) -> String {
    format!(
        "fmt=avif|backend={}|quality={QUALITY}|speed={SPEED}|threads={}|channels=rgb",
        backend.env_value(),
        threads.env_value()
    )
}

pub fn encode_dynamic_image<W: Write>(
    resized: &DynamicImage,
    writer: &mut W,
    cache_path: &Path,
    backend: EncoderBackend,
    threads: ThreadSetting,
) -> Result<()> {
    let rgb = resized.to_rgb8();
    let avif_file = match backend {
        EncoderBackend::Native => encode_with_native(&rgb, threads)
            .with_context(|| format!("Failed to encode cached image {}", cache_path.display()))?,
        EncoderBackend::Ravif => encode_with_ravif(&rgb, threads)
            .with_context(|| format!("Failed to encode cached image {}", cache_path.display()))?,
    };

    writer
        .write_all(&avif_file)
        .with_context(|| format!("Failed to write cached image {}", cache_path.display()))?;
    Ok(())
}

fn encode_with_ravif(rgb: &image::RgbImage, threads: ThreadSetting) -> Result<Vec<u8>> {
    let ravif_encoder = RavifEncoder::new()
        .with_quality(QUALITY as f32)
        .with_speed(SPEED)
        .with_num_threads(threads.to_ravif_threads());
    let (width, height) = rgb.dimensions();
    let width = usize::try_from(width).context("AVIF width does not fit usize")?;
    let height = usize::try_from(height).context("AVIF height does not fit usize")?;
    let pixels = rgb.as_raw().as_slice().as_rgb();
    let avif_file = ravif_encoder
        .encode_rgb(RavifImg::new(pixels, width, height))
        .context("ravif encoder failed")?
        .avif_file;

    Ok(avif_file)
}

fn encode_with_native(rgb: &image::RgbImage, threads: ThreadSetting) -> Result<Vec<u8>> {
    let (width, height) = rgb.dimensions();

    let mut native_encoder = NativeAvifEncoder::new();
    native_encoder
        .set_quality(QUALITY)
        .set_alpha_quality(QUALITY)
        .set_speed(SPEED);
    if let Some(max_threads) = threads.to_native_threads() {
        native_encoder.set_max_threads(max_threads);
    }

    let rgb_pixels = RgbPixels::new(width, height, rgb.as_raw())
        .map_err(|err| anyhow!("failed to prepare libavif RGB pixels: {err}"))?;
    let image = rgb_pixels.to_image(YuvFormat::Yuv444);
    let encoded = native_encoder
        .encode(&image)
        .map_err(|err| anyhow!("libavif encoder failed: {err}"))?;
    Ok(encoded.to_vec())
}
