# Card Images

## Location

Card images are loaded from:

- `static/assets/cards/`

Backend source of truth:

- `talespin-server/src/main.rs` reads `../static/assets/cards/`
- Allowed extensions: `.jpg`, `.jpeg`, `.png` (lowercase match in code)

## Can I Add Images Directly?

Yes. Copy new files into `static/assets/cards/`, then restart `talespin_backend` so the backend reloads the deck at startup.

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
