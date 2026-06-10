# Card Images

## Sources

Cards are loaded at backend startup from:

- built-in directory: `static/assets/cards/`
- optional extra directories from `TALESPIN_EXTRA_IMAGE_DIRS`

`TALESPIN_EXTRA_IMAGE_DIRS` is newline-separated and supports `~/...` paths. Extra directories are scanned recursively, including symlinked directories.

Set `TALESPIN_WATCH_IMAGE_P=true` to watch those extra image directories while the backend is
running. Startup still performs the normal full scan once. After startup, watch events process only
the changed file path instead of rescanning the whole image tree.

## Supported Formats

- `.jpg`
- `.jpeg`
- `.png`
- `.webp`

Extensionless files are ignored by default.  
Set `TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P=y` to sniff extensionless files and accept only JPEG/PNG/WebP payloads.

## Runtime Flow

The backend normalizes all source images into cached cards and serves them through:

- `GET /cards/:card_id`
- `GET /cards/:card_id_original`

Cards are not copied/symlinked into `static/assets/cards/` at runtime anymore.

The `_original` route serves the source image bytes for the card without Talespin's crop, resize,
or cache encoding. The in-game card preview uses this route for copy, open, and download actions.

## Watching Extra Images

Watch mode applies only to `TALESPIN_EXTRA_IMAGE_DIRS`, not the built-in card directory. Every configured extra directory is registered with the watcher, and recursive watching follows symlinked directories the same way startup loading does.

When a watched file is added or updated, the backend normalizes that one source image and updates
the active card catalog. When a watched file is deleted, that source is removed from the active
catalog. New game rooms use the latest active catalog when they are created.

Existing game rooms keep the deck snapshot they were created with, so they do not gain or lose
cards after watch updates. Old card IDs remain registered for serving cached `/cards/:card_id`
images when the cache file still exists. If the original source file was deleted, the corresponding
`/cards/:card_id_original` route may return 404.

## Dev Source Metadata

Set `TALESPIN_IMAGES_SHOW_PATH_P=y` while `TALESPIN_PRODUCTION_P` is not `y` to expose:

- `GET /cards/:card_id/source-info`

That endpoint returns the card's path relative to its source image root so local/dev operators can
find the file. It returns 404 unless explicitly enabled outside production mode.

## Cache

- env var: `TALESPIN_CACHE_DIR`
- default: `~/.cache/talespin`
- card files are stored under: `~/.cache/talespin/cards/`

Cache key is based on:

- SHA-256 of source file bytes
- output/transform spec (ratio, long side, format + encoder settings)
- normalization pipeline version

Cache hit integrity checks:

- env var: `TALESPIN_VALIDATE_CACHE_HITS_P`
- default: `y` (enabled)
- when enabled, existing cache files are fully decoded/validated (AVIF decode uses the native dav1d stack via `image`'s `avif-native` feature); corrupt/truncated files are deleted and rebuilt
- when disabled, startup is faster but corrupt cache files can slip through

## Aspect Ratio and Size

- env var: `TALESPIN_CARD_ASPECT_RATIO` (default `2:3`)
- env var: `TALESPIN_CARD_LONG_SIDE` (default `1536`)
- env var: `TALESPIN_CARD_CACHE_FORMAT` (default `avif`; supported: `avif`, `jpeg`)
- env var: `TALESPIN_CARD_AVIF_ENCODER` (default `native`; supported: `native`, `ravif`)
- env var: `TALESPIN_CARD_AVIF_THREADS` (default `auto`; `auto` uses encoder default, or set a positive integer)

Behavior:

- image is center-cropped to target ratio
- then resized to target dimensions derived from ratio and long side
- output is encoded with the selected cache format

Defaults:

- AVIF (`quality=80`, `speed=6`, backend `native` via `libavif+codec-aom`, threads `auto` / encoder default)

For default `2:3`, output size is `1024x1536`.

## Built-in Toggle and Errors

- `TALESPIN_DISABLE_BUILTIN_IMAGES_P=y` disables built-in cards.
- `TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P=y` enables content-sniffing for extensionless files.
- Startup prints a short “Preparing card caches …” line before showing normalization progress.
- If extra dirs are set but no supported images are found, startup fails with an error.
- If built-ins are disabled and extra dirs yield zero images, startup fails with an error.
