#!/usr/bin/env zsh
emulate -L zsh -o errexit -o nounset -o pipefail

readonly ROOT_DIR="${0:A:h}"
readonly STATE_DIR="$ROOT_DIR/.self_host"
readonly CONFIG_FILE="$STATE_DIR/config.env"
readonly CADDYFILE="${CADDYFILE:-$HOME/Caddyfile}"
readonly DEFAULT_PUBLIC_URL='https://talespin.pinky.lilf.ir'
readonly DEFAULT_NODE_VERSION='20'
readonly BACKEND_PORT='8081'
readonly FRONTEND_PORT='4173'
readonly BACKEND_SESSION='talespin_backend'
readonly FRONTEND_SESSION='talespin_frontend'
readonly CADDY_BEGIN='# BEGIN talespin self-host'
readonly CADDY_END='# END talespin self-host'
readonly DEFAULT_PROXY_URL='http://127.0.0.1:1087'

usage() {
	cat <<USAGE
Usage: ./self_host.zsh [setup|redeploy|start|stop] [public_url]

setup     Persist config, build frontend/backend, update ~/Caddyfile, reload Caddy, and start Talespin.
redeploy  Rebuild from the latest local working tree changes, reload Caddy, and restart Talespin.
start     Start Talespin from existing artifacts and persisted config.
stop      Stop the tmux-managed Talespin sessions.

Default public_url: $DEFAULT_PUBLIC_URL
Set TALESPIN_PRODUCTION_P=n to run the frontend dev server behind Caddy.
USAGE
}

die() {
	print -u2 -- "Error: $*"
	exit 1
}

note() {
	print -- "==> $*"
}

require_cmd() {
	command -v "$1" >/dev/null 2>&1 || die "Missing required command: $1"
}

tmuxnew () {
	tmux kill-session -t "$1" &> /dev/null || true
	tmux new -d -s "$@"
}

is_yes() {
	[[ "${1:-}" == [yY] ]]
}

production_mode_enabled() {
	is_yes "${TALESPIN_PRODUCTION_P:-y}"
}

ensure_dirs() {
	mkdir -p "$STATE_DIR"
}

normalize_public_url() {
	local input_url="${1:-$DEFAULT_PUBLIC_URL}"
	python3 - "$input_url" <<'PY'
import sys
from urllib.parse import urlparse

raw = sys.argv[1].strip()
if not raw:
    raise SystemExit('public_url must not be empty')
parsed = urlparse(raw)
if parsed.scheme not in {'http', 'https'}:
    raise SystemExit('public_url must begin with http:// or https://')
if not parsed.netloc:
    raise SystemExit('public_url must include a hostname')
if parsed.path not in ('', '/'):
    raise SystemExit('public_url must not include a path')
if parsed.params or parsed.query or parsed.fragment:
    raise SystemExit('public_url must not include params, query, or fragment')
print(f'{parsed.scheme}://{parsed.netloc}')
PY
}

caddy_site_address() {
	local normalized_url
	normalized_url="$(normalize_public_url "$1")"
	python3 - "$normalized_url" <<'PY'
import sys
from urllib.parse import urlparse

parsed = urlparse(sys.argv[1])
if parsed.scheme == 'https':
    print(parsed.netloc)
else:
    print(f'{parsed.scheme}://{parsed.netloc}')
PY
}

load_config() {
	[[ -f "$CONFIG_FILE" ]] || die "Missing config file: $CONFIG_FILE. Run ./self_host.zsh setup first."
	source "$CONFIG_FILE"
	[[ -n "${PUBLIC_URL:-}" ]] || die "Config file is missing PUBLIC_URL: $CONFIG_FILE"
}

persist_config() {
	local public_url="$1"
	ensure_dirs
	cat > "$CONFIG_FILE" <<EOF_CONFIG
PUBLIC_URL='${public_url}'
EOF_CONFIG
}

resolve_configured_url() {
	if [[ -n "${1:-}" ]]; then
		normalize_public_url "$1" || die 'Invalid public URL'
	elif [[ -f "$CONFIG_FILE" ]]; then
		load_config
		normalize_public_url "$PUBLIC_URL" || die 'Saved public URL is invalid'
	else
		normalize_public_url "$DEFAULT_PUBLIC_URL" || die 'Default public URL is invalid'
	fi
}

ensure_node_shell() {
	zsh -lc 'source ~/.shared.sh >/dev/null 2>&1 || true; type nvm-load >/dev/null 2>&1' \
		|| die 'nvm-load is required in zsh login shells'
}

run_in_node_shell() {
	local command_string="$1"
	zsh -lc "source ~/.shared.sh >/dev/null 2>&1 || true; nvm-load >/dev/null 2>&1; nvm use ${(q)DEFAULT_NODE_VERSION} >/dev/null; cd ${(q)ROOT_DIR}; ${command_string}"
}

ensure_frontend_dependencies_present() {
	[[ -d "$ROOT_DIR/node_modules" ]] || die 'Missing node_modules. Run npm install first.'
}

set_proxy_env() {
	local proxy_url="${ALL_PROXY:-${all_proxy:-${HTTP_PROXY:-${http_proxy:-${HTTPS_PROXY:-${https_proxy:-$DEFAULT_PROXY_URL}}}}}}"
	export ALL_PROXY="$proxy_url"
	export all_proxy="$proxy_url"
	export http_proxy="$proxy_url"
	export https_proxy="$proxy_url"
	export HTTP_PROXY="$proxy_url"
	export HTTPS_PROXY="$proxy_url"
}

prepare_talespin_env() {
	set_proxy_env
	: "${TALESPIN_PRODUCTION_P:=y}"
	: "${TALESPIN_DISABLE_BUILTIN_IMAGES_P:=y}"
	: "${TALESPIN_AUTO_DOWNLOAD_EXTRA_IMAGES_P:=y}"
	: "${TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P:=y}"
	: "${TALESPIN_CACHE_DIR:=~/.cache/talespin}"
	: "${TALESPIN_CARD_ASPECT_RATIO:=2:3}"
	: "${TALESPIN_CARD_LONG_SIDE:=1536}"
	: "${TALESPIN_CARD_CACHE_FORMAT:=avif}"
	: "${TALESPIN_CARD_AVIF_ENCODER:=native}"
	: "${TALESPIN_CARD_AVIF_THREADS:=auto}"
	: "${TALESPIN_VALIDATE_CACHE_HITS_P:=y}"

	typeset -a talespin_custom_image_dirs=(
		~/Pictures/SurrealPictures/chosen_1
		~/base/virtualtabletop/library/games/Dreamcatcher/assets
	)
	typeset -A talespin_image_repo_map=(
		"$HOME/Pictures/SurrealPictures" "https://github.com/NightMachinery/SurrealPictures.git"
		"$HOME/base/virtualtabletop" "https://github.com/ArnoldSmith86/virtualtabletop.git"
	)

	local image_dir resolved_image_dir repo_root repo_url
	for image_dir in "${talespin_custom_image_dirs[@]}"; do
		resolved_image_dir="${~image_dir}"
		if [[ -d "$resolved_image_dir" || ! "${TALESPIN_AUTO_DOWNLOAD_EXTRA_IMAGES_P}" == [yY] ]]; then
			continue
		fi

		for repo_root repo_url in ${(kv)talespin_image_repo_map}; do
			if [[ "$resolved_image_dir" == "$repo_root" || "$resolved_image_dir" == "$repo_root/"* ]]; then
				if [[ -d "$repo_root" ]]; then
					break
				fi
				note "Cloning missing image source repo: $repo_url -> $repo_root"
				mkdir -p "${repo_root:h}"
				if git clone --depth 1 "$repo_url" "$repo_root"; then
					note "Clone complete: $repo_root"
				else
					note "Failed to clone $repo_url"
				fi
				break
			fi
		done
	done

	typeset -a talespin_existing_image_dirs=()
	for image_dir in "${talespin_custom_image_dirs[@]}"; do
		resolved_image_dir="${~image_dir}"
		if [[ -d "$resolved_image_dir" ]]; then
			talespin_existing_image_dirs+=("$resolved_image_dir")
		else
			note "Skipping missing image directory: $resolved_image_dir"
		fi
	done

	if (( ${#talespin_existing_image_dirs[@]} == 0 )); then
		note 'No valid custom image directories found; forcing built-in images on.'
		TALESPIN_DISABLE_BUILTIN_IMAGES_P='n'
	fi

	export TALESPIN_EXTRA_IMAGE_DIRS="${(@F)talespin_existing_image_dirs}"
	export TALESPIN_DISABLE_BUILTIN_IMAGES_P
	export TALESPIN_PRODUCTION_P
	export TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P
	export TALESPIN_CACHE_DIR
	export TALESPIN_CARD_ASPECT_RATIO
	export TALESPIN_CARD_LONG_SIDE
	export TALESPIN_CARD_CACHE_FORMAT
	export TALESPIN_CARD_AVIF_ENCODER
	export TALESPIN_CARD_AVIF_THREADS
	export TALESPIN_VALIDATE_CACHE_HITS_P
	export TALESPIN_AUTO_DOWNLOAD_EXTRA_IMAGES_P
}

launch_exports() {
	local -a env_vars=(
		ALL_PROXY all_proxy http_proxy https_proxy HTTP_PROXY HTTPS_PROXY
		TALESPIN_PRODUCTION_P
		TALESPIN_DISABLE_BUILTIN_IMAGES_P
		TALESPIN_AUTO_DOWNLOAD_EXTRA_IMAGES_P
		TALESPIN_EXTRA_IMAGE_DIRS
		TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P
		TALESPIN_CACHE_DIR
		TALESPIN_CARD_ASPECT_RATIO
		TALESPIN_CARD_LONG_SIDE
		TALESPIN_CARD_CACHE_FORMAT
		TALESPIN_CARD_AVIF_ENCODER
		TALESPIN_CARD_AVIF_THREADS
		TALESPIN_VALIDATE_CACHE_HITS_P
		TALESPIN_DEFAULT_WIN_POINTS
		TALESPIN_MAX_MEMBERS
	)
	local var_name value exports=''
	for var_name in "${env_vars[@]}"; do
		if [[ -v "$var_name" ]]; then
			value="${(P)var_name}"
			exports+="export ${var_name}=${(q)value}; "
		fi
	done
	print -r -- "$exports"
}

port_is_busy() {
	local port="$1"
	ss -ltn "( sport = :${port} )" | tail -n +2 | grep -q LISTEN
}

ensure_runtime_ports_available() {
	port_is_busy "$BACKEND_PORT" && die "Port $BACKEND_PORT is already in use; stop the conflicting process and retry."
	if ! production_mode_enabled; then
		port_is_busy "$FRONTEND_PORT" && die "Port $FRONTEND_PORT is already in use; stop the conflicting process and retry."
	fi
}

build_backend() {
	note 'Building Talespin backend...'
	(
		cd "$ROOT_DIR/talespin-server"
		cargo build --release
	)
}

build_frontend() {
	note 'Building Talespin frontend...'
	ensure_frontend_dependencies_present
	run_in_node_shell 'npm run build'
}

ensure_backend_artifact() {
	[[ -x "$ROOT_DIR/talespin-server/target/release/talespin-server" ]] \
		|| die 'Missing talespin-server/target/release/talespin-server. Run ./self_host.zsh setup or redeploy first.'
}

ensure_prod_frontend_artifacts() {
	[[ -f "$ROOT_DIR/build/index.html" ]] || die 'Missing build/index.html. Run ./self_host.zsh setup or redeploy first.'
}

render_caddy_block() {
	local site_address="$1"
	if production_mode_enabled; then
		cat <<EOF_BLOCK
${site_address} {
    encode zstd gzip

    @talespin_backend {
        path /create /exists /stats /ws /ws/* /cards /cards/*
    }

    handle @talespin_backend {
        reverse_proxy 127.0.0.1:${BACKEND_PORT}
    }

    handle {
        root * ${ROOT_DIR}/build
        try_files {path} /404.html
        file_server
    }
}
EOF_BLOCK
	else
		cat <<EOF_BLOCK
${site_address} {
    encode zstd gzip

    @talespin_backend {
        path /create /exists /stats /ws /ws/* /cards /cards/*
    }

    handle @talespin_backend {
        reverse_proxy 127.0.0.1:${BACKEND_PORT}
    }

    handle {
        reverse_proxy 127.0.0.1:${FRONTEND_PORT}
    }
}
EOF_BLOCK
	fi
}

update_caddyfile() {
	local public_url="$1"
	local site_address candidate_file block_contents
	ensure_dirs
	site_address="$(caddy_site_address "$public_url")"
	candidate_file="$STATE_DIR/Caddyfile.candidate"
	block_contents="$(render_caddy_block "$site_address")"
	[[ -f "$CADDYFILE" ]] || touch "$CADDYFILE"

	TARGET_CADDYFILE="$CADDYFILE" \
	OUTPUT_PATH="$candidate_file" \
	BLOCK_BEGIN="$CADDY_BEGIN" \
	BLOCK_END="$CADDY_END" \
	BLOCK_CONTENTS="$block_contents" \
	SITE_ADDRESS="$site_address" \
	DEFAULT_PUBLIC_URL="$DEFAULT_PUBLIC_URL" \
	python3 - <<'PY'
import os
import pathlib
import re
from urllib.parse import urlparse

caddyfile = pathlib.Path(os.environ['TARGET_CADDYFILE']).expanduser()
output_path = pathlib.Path(os.environ['OUTPUT_PATH'])
text = caddyfile.read_text() if caddyfile.exists() else ''
begin = os.environ['BLOCK_BEGIN']
end = os.environ['BLOCK_END']
block = os.environ['BLOCK_CONTENTS'].rstrip() + '\n'
site_address = os.environ['SITE_ADDRESS']
default_public_url = os.environ['DEFAULT_PUBLIC_URL']

managed_pattern = re.compile(re.escape(begin) + r'.*?' + re.escape(end) + r'\n?', re.S)
text = managed_pattern.sub('', text)

def legacy_labels() -> list[str]:
    labels = {site_address}
    parsed = urlparse(default_public_url)
    labels.add(parsed.netloc)
    labels.add(default_public_url)
    return [label for label in labels if label]

def remove_labeled_block(source: str, label: str) -> tuple[str, bool]:
    pattern = re.compile(rf'(?m)^[ \t]*{re.escape(label)}[ \t]*\{{')
    match = pattern.search(source)
    if not match:
        return source, False
    brace_start = source.find('{', match.start())
    if brace_start == -1:
        return source, False
    depth = 0
    index = brace_start
    while index < len(source):
        char = source[index]
        if char == '{':
            depth += 1
        elif char == '}':
            depth -= 1
            if depth == 0:
                index += 1
                while index < len(source) and source[index] == '\n':
                    index += 1
                block_text = source[match.start():index]
                markers = (
                    '/create /exists /stats /ws /ws/* /cards /cards/*',
                    'import talespin_front_{$TALESPIN_PRODUCTION_P:y}',
                    'reverse_proxy 127.0.0.1:8081',
                )
                if any(marker in block_text for marker in markers):
                    new_text = (source[:match.start()].rstrip() + '\n\n' + source[index:].lstrip('\n')).strip('\n')
                    if new_text:
                        new_text += '\n'
                    return new_text, True
                return source, False
        index += 1
    return source, False

for label in legacy_labels():
    changed = True
    while changed:
        text, changed = remove_labeled_block(text, label)

managed_block = f'{begin}\n{block.rstrip()}\n{end}\n'
text = text.rstrip()
if text:
    text += '\n\n'
text += managed_block
output_path.write_text(text)
PY

	caddy validate --config "$candidate_file" --adapter caddyfile >/dev/null
	cp "$candidate_file" "$CADDYFILE"
}

reload_caddy() {
	note 'Reloading Caddy...'
	caddy reload --config "$CADDYFILE" --adapter caddyfile >/dev/null \
		|| die "Failed to reload Caddy with $CADDYFILE. Ensure Caddy is running."
}

stop_app() {
	if tmux has-session -t "$BACKEND_SESSION" 2>/dev/null; then
		tmux kill-session -t "$BACKEND_SESSION"
		note "Stopped tmux session: $BACKEND_SESSION"
	fi
	if tmux has-session -t "$FRONTEND_SESSION" 2>/dev/null; then
		tmux kill-session -t "$FRONTEND_SESSION"
		note "Stopped tmux session: $FRONTEND_SESSION"
	fi
}

start_backend() {
	local exports cmd
	exports="$(launch_exports)"
	cmd="${exports}cd ${(q)ROOT_DIR}/talespin-server; exec ./target/release/talespin-server"
	tmuxnew "$BACKEND_SESSION" zsh -lc "$cmd"
	note "Started tmux session: $BACKEND_SESSION"
}

start_frontend_dev() {
	local exports cmd
	exports="$(launch_exports)"
	cmd="${exports}source ~/.shared.sh >/dev/null 2>&1 || true; nvm-load >/dev/null 2>&1; nvm use ${(q)DEFAULT_NODE_VERSION} >/dev/null; cd ${(q)ROOT_DIR}; exec npm run dev -- --host 127.0.0.1 --port ${FRONTEND_PORT} --debug"
	tmuxnew "$FRONTEND_SESSION" zsh -lc "$cmd"
	note "Started tmux session: $FRONTEND_SESSION"
}

start_app() {
	ensure_backend_artifact
	if production_mode_enabled; then
		ensure_prod_frontend_artifacts
	else
		ensure_node_shell
		ensure_frontend_dependencies_present
	fi
	stop_app
	ensure_runtime_ports_available
	start_backend
	if production_mode_enabled; then
		note 'Running in production mode; frontend will be served by Caddy from build/.'
	else
		start_frontend_dev
	fi
}

ensure_common_prereqs() {
	require_cmd zsh
	require_cmd tmux
	require_cmd caddy
	require_cmd git
	require_cmd python3
	require_cmd ss
}

ensure_build_prereqs() {
	ensure_common_prereqs
	require_cmd cargo
	ensure_node_shell
}

ensure_stop_prereqs() {
	require_cmd tmux
}

setup_cmd() {
	local public_url="$1"
	persist_config "$public_url"
	prepare_talespin_env
	build_backend
	build_frontend
	update_caddyfile "$public_url"
	reload_caddy
	start_app
	note "Setup complete for $public_url"
}

redeploy_cmd() {
	local public_url="$1"
	persist_config "$public_url"
	prepare_talespin_env
	build_backend
	build_frontend
	update_caddyfile "$public_url"
	reload_caddy
	start_app
	note "Redeploy complete for $public_url"
}

start_cmd() {
	local public_url="$1"
	persist_config "$public_url"
	prepare_talespin_env
	update_caddyfile "$public_url"
	reload_caddy
	start_app
	note "Start complete for $public_url"
}

stop_cmd() {
	ensure_stop_prereqs
	stop_app
	note 'Stop complete.'
}

main() {
	local command="${1:-}"
	local supplied_url="${2:-}"
	local resolved_url=''

	case "$command" in
		setup)
			(( $# <= 2 )) || die 'setup accepts at most one public_url argument.'
			ensure_build_prereqs
			resolved_url="$(resolve_configured_url "$supplied_url")"
			setup_cmd "$resolved_url"
			;;
		redeploy)
			(( $# <= 2 )) || die 'redeploy accepts at most one public_url argument.'
			ensure_build_prereqs
			resolved_url="$(resolve_configured_url "$supplied_url")"
			redeploy_cmd "$resolved_url"
			;;
		start)
			(( $# <= 2 )) || die 'start accepts at most one public_url argument.'
			ensure_common_prereqs
			if [[ -n "$supplied_url" ]]; then
				resolved_url="$(resolve_configured_url "$supplied_url")"
			else
				load_config
				resolved_url="$(normalize_public_url "$PUBLIC_URL")"
			fi
			start_cmd "$resolved_url"
			;;
		stop)
			(( $# == 1 )) || die 'stop does not accept additional arguments.'
			stop_cmd
			;;
		-h|--help|help)
			usage
			;;
		*)
			usage
			exit 1
			;;
	esac
}

main "$@"
