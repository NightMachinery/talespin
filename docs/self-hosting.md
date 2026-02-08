# Self-Hosting

## Runtime Layout

- Frontend: Svelte dev server at `127.0.0.1:4173`
- Backend: Rust server at `127.0.0.1:8081`
- Public hostname: `https://talespin.pinky.lilf.ir`
- Reverse proxy: Caddy (`~/Caddyfile`)

## Caddy Routing

In `~/Caddyfile`, Talespin is routed as:

- `/create`, `/exists`, `/stats`, `/ws`, `/ws/*`, `/cards`, `/cards/*` -> `127.0.0.1:8081` (backend)
- all other paths -> `127.0.0.1:4173` (frontend)

## Startup Scripts

### Server-wide launcher

File: `~/launch.zsh`

This remains the main server launcher and keeps the original shared services:

- `paqet-client`
- `xray_server`
- `caddy`
- `virtualtabletop`

It then calls the Talespin-local tmux script:

- `~/base/talespin/run_tmux.zsh`

### Game-local tmux script

File: `~/base/talespin/run_tmux.zsh`

This script should only manage Talespin sessions:

- `talespin_backend`
- `talespin_frontend`

It also defines `tmuxnew` and exports proxy env vars required for package/network operations.
It sets `TALESPIN_EXTRA_IMAGE_DIRS` (newline-separated dirs) for loading extra card images.
You can set `TALESPIN_DISABLE_BUILTIN_IMAGES_P=y` to disable built-in cards.
It also exports:

- `TALESPIN_CACHE_DIR` (default `~/.cache/talespin`)
- `TALESPIN_CARD_ASPECT_RATIO` (default `2:3`)
- `TALESPIN_CARD_LONG_SIDE` (default `1536`)
- `TALESPIN_CARD_CACHE_FORMAT` (default `avif`; `avif` or `jpeg`)
- `TALESPIN_VALIDATE_CACHE_HITS_P` (default `y`; when `y`, corrupted cache files are detected and rebuilt)
- `TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P` (default `n`; when `y`, extensionless files are sniffed and JPEG/PNG/WebP are accepted)
- `TALESPIN_DEFAULT_WIN_POINTS` (default `10`, used when lobby creates `points` mode without explicit target)

## Dependencies

- Node: use `nvm-load` then `nvm use 20`
- Rust: stable toolchain via `rustup`
- Ubuntu packages for AVIF decode support (required by backend build with `avif-native`):
  - `pkg-config`
  - `libdav1d-dev`
  - install command:
    ```bash
    sudo apt-get update
    sudo apt-get install -y pkg-config libdav1d-dev
    ```
- No Docker required for Talespin itself

## Useful Checks

```bash
tmux ls
tmux capture-pane -pt talespin_backend | tail -n 40
tmux capture-pane -pt talespin_frontend | tail -n 40
curl -k --resolve talespin.pinky.lilf.ir:443:127.0.0.1 https://talespin.pinky.lilf.ir/ -I
curl -k --resolve talespin.pinky.lilf.ir:443:127.0.0.1 -X POST https://talespin.pinky.lilf.ir/create
```
