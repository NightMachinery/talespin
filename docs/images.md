# Card Images

## Location

Card images are loaded from:

- `static/assets/cards/`

Backend source of truth:

- `talespin-server/src/main.rs` reads `../static/assets/cards/`
- Allowed extensions: `.jpg`, `.jpeg`, `.png` (lowercase match in code)

## Can I Add Images Directly?

Yes. Copy new files into `static/assets/cards/`, then restart `talespin_backend` so the backend reloads the deck at startup.

## Custom Image Directories

You can also load images from extra directories via:

- env var `TALESPIN_EXTRA_IMAGE_DIRS`
- format: newline-separated absolute/`~/...` directory paths
- optional env var `TALESPIN_DISABLE_BUILTIN_IMAGES_P`; set to `y` to use only extra-dir images

At startup, the backend scans those directories recursively, imports supported files (`.jpg`, `.jpeg`, `.png`), and links/copies them into `static/assets/cards/` with generated names, so they are usable by the existing frontend.
If custom dirs are configured but no supported images are found, the backend exits with an error.

## Size/Resize Behavior

- There is no explicit server-side size limit in code.
- There is no automatic backend resize/compression pipeline.
- Images are served as static files and rendered directly by the browser.
- Frontend layout scales display in the UI grid, but that is visual scaling only.
- Very large images still download at full file size and can slow page loads.

## Practical Recommendations

- Keep a consistent aspect ratio across cards for cleaner UI.
- Prefer optimized images to reduce network usage.
- Stick to lowercase file extensions (`.jpg`, `.jpeg`, `.png`) to ensure they are loaded.
