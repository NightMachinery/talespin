# Self-Hosting

## Runtime Layout

- Frontend (production): static files served by Caddy from `~/base/talespin/build`
- Frontend (development): Vite dev server at `127.0.0.1:4173`
- Backend: Rust server at `127.0.0.1:8081`
- Public hostname: `https://talespin.pinky.lilf.ir`
- Reverse proxy: Caddy (`~/Caddyfile`)

## Caddy Routing

`self_host.zsh` owns a managed Talespin block in `~/Caddyfile`:

- markers: `# BEGIN talespin self-host` / `# END talespin self-host`
- `/create`, `/exists`, `/stats`, `/ws`, `/ws/*`, `/cards`, `/cards/*` -> `127.0.0.1:8081`
- all other paths ->
  - production: static files from `~/base/talespin/build`
  - development: `127.0.0.1:4173`

`setup-caddy` only updates that managed block, or inserts it if missing. Any legacy unmanaged Talespin snippets should be removed manually.

## Startup Script

File: `~/base/talespin/self_host.zsh`

Usage:

```bash
./self_host.zsh [setup|setup-caddy|redeploy|start|stop] [public_url]
```

Commands:

- `setup`: persist config, build frontend/backend, update Caddy, reload Caddy, and start Talespin
- `setup-caddy`: persist config, update Caddy, and reload Caddy without rebuilding or restarting Talespin
- `redeploy`: rebuild from the latest local working tree changes, update Caddy, reload Caddy, and restart Talespin
- `start`: start from existing artifacts and persisted config
- `stop`: stop the tmux-managed Talespin sessions

Default `public_url`: `https://talespin.pinky.lilf.ir`

## Runtime Behavior

The script manages these tmux sessions:

- `talespin_backend`
- `talespin_frontend` (development mode only)

Production is the default when `TALESPIN_PRODUCTION_P` is unset.
Run in development mode with:

```bash
TALESPIN_PRODUCTION_P=n zsh self_host.zsh start
```

The script preserves the existing Talespin self-host env behavior, including:

- proxy exports (`127.0.0.1:1087` by default unless proxy env vars are already set)
- `TALESPIN_EXTRA_IMAGE_DIRS` (newline-separated dirs) for loading extra card images
- optional auto-clone of missing extra-image repos when `TALESPIN_AUTO_DOWNLOAD_EXTRA_IMAGES_P=y`
- `TALESPIN_DISABLE_BUILTIN_IMAGES_P` (default `y`, but forced to `n` if no extra image dirs exist)
- `TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P` (default `y`)
- `TALESPIN_CACHE_DIR` (default `~/.cache/talespin`)
- `TALESPIN_CARD_ASPECT_RATIO` (default `2:3`)
- `TALESPIN_CARD_LONG_SIDE` (default `1536`)
- `TALESPIN_CARD_CACHE_FORMAT` (default `avif`; `avif` or `jpeg`)
- `TALESPIN_CARD_AVIF_ENCODER` (default `native`; `native` or `ravif`)
- `TALESPIN_CARD_AVIF_THREADS` (default `auto`; `auto` uses encoder default, or set a positive integer)
- `TALESPIN_VALIDATE_CACHE_HITS_P` (default `y`; when `y`, corrupted cache files are detected and rebuilt)
- any externally supplied `TALESPIN_DEFAULT_WIN_POINTS` / `TALESPIN_MAX_MEMBERS`

## Build Commands

Backend rebuilds use:

```bash
( cd talespin-server && cargo build --release )
```

Cargo automatically skips work when the backend is already up to date.

Frontend rebuilds use Node 20 through `nvm-load; nvm use 20` and then:

```bash
npm run build
```

## Compatibility Note

`run_tmux.zsh` now exists only as a deprecated compatibility shim. With no arguments it forwards to:

```bash
zsh self_host.zsh redeploy
```

## Server-Wide Launcher Follow-Up

If you use `~/launch.zsh`, update the Talespin stanza manually from:

```bash
cd ~/base/talespin
zsh run_tmux.zsh
```

to:

```bash
cd ~/base/talespin
zsh self_host.zsh start
```

## Dependencies

- Node: `nvm-load` then `nvm use 20`
- Rust: stable toolchain via `rustup`
- `tmux`
- `caddy`
- `python3`
- `git`
- `ss` (from `iproute2`)
- Ubuntu packages for AVIF encode/decode support (required by backend build with `avif-native` and native encoder path):
  - `pkg-config`
  - `libdav1d-dev`
  - `libavif-dev`
  - `libaom-dev`
  - install command:
    ```bash
    sudo apt-get update
    sudo apt-get install -y pkg-config libdav1d-dev libavif-dev libaom-dev
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
