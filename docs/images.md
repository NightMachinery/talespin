# Card Images

## Sources

Cards are loaded at backend startup from:

- built-in directory: `static/assets/cards/`
- optional extra directories from `TALESPIN_EXTRA_IMAGE_DIRS`

`TALESPIN_EXTRA_IMAGE_DIRS` is newline-separated and supports `~/...` paths. Extra directories are scanned recursively.

## Supported Formats

- `.jpg`
- `.jpeg`
- `.png`
- `.webp`

## Runtime Flow

The backend normalizes all source images into cached JPEG cards and serves them through:

- `GET /cards/:card_id`

Cards are not copied/symlinked into `static/assets/cards/` at runtime anymore.

## Cache

- env var: `TALESPIN_CACHE_DIR`
- default: `~/.cache/talespin`
- card files are stored under: `~/.cache/talespin/cards/`

Cache key is based on:

- SHA-256 of source file bytes
- output/transform spec (ratio, long side, format/quality)
- normalization pipeline version

## Aspect Ratio and Size

- env var: `TALESPIN_CARD_ASPECT_RATIO` (default `2:3`)
- env var: `TALESPIN_CARD_LONG_SIDE` (default `1536`)

Behavior:

- image is center-cropped to target ratio
- then resized to target dimensions derived from ratio and long side
- output is JPEG quality 90

For default `2:3`, output size is `1024x1536`.

## Built-in Toggle and Errors

- `TALESPIN_DISABLE_BUILTIN_IMAGES_P=y` disables built-in cards.
- If extra dirs are set but no supported images are found, startup fails with an error.
- If built-ins are disabled and extra dirs yield zero images, startup fails with an error.
