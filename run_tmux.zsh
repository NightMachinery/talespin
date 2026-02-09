#!/usr/bin/env zsh

tmuxnew () {
	tmux kill-session -t "$1" &> /dev/null || true
	tmux new -d -s "$@"
}

export ALL_PROXY=http://127.0.0.1:1087 all_proxy=http://127.0.0.1:1087 http_proxy=http://127.0.0.1:1087 https_proxy=http://127.0.0.1:1087 HTTP_PROXY=http://127.0.0.1:1087 HTTPS_PROXY=http://127.0.0.1:1087

export TALESPIN_PRODUCTION_P="${TALESPIN_PRODUCTION_P:-n}"
tmux set-environment -g TALESPIN_PRODUCTION_P "$TALESPIN_PRODUCTION_P"
##
export TALESPIN_DISABLE_BUILTIN_IMAGES_P="${TALESPIN_DISABLE_BUILTIN_IMAGES_P:-y}"
export TALESPIN_AUTO_DOWNLOAD_EXTRA_IMAGES_P="${TALESPIN_AUTO_DOWNLOAD_EXTRA_IMAGES_P:-y}"
typeset -a talespin_custom_image_dirs=(
	# ~/Pictures/SurrealPictures
	~/Pictures/SurrealPictures/chosen_1
	~/base/virtualtabletop/library/games/Dreamcatcher/assets
)
typeset -A talespin_image_repo_map=(
	"$HOME/Pictures/SurrealPictures" "https://github.com/NightMachinery/SurrealPictures.git"
	"$HOME/base/virtualtabletop" "https://github.com/ArnoldSmith86/virtualtabletop.git"
)

for image_dir in "${talespin_custom_image_dirs[@]}"; do
	resolved_image_dir="${~image_dir}"
	if [[ -d "$resolved_image_dir" || ! "$TALESPIN_AUTO_DOWNLOAD_EXTRA_IMAGES_P" == [yY] ]]; then
		continue
	fi

	for repo_root repo_url in ${(kv)talespin_image_repo_map}; do
		if [[ "$resolved_image_dir" == "$repo_root" || "$resolved_image_dir" == "$repo_root/"* ]]; then
			if [[ -d "$repo_root" ]]; then
				break
			fi
			echo "Cloning missing image source repo: $repo_url -> $repo_root"
			mkdir -p "${repo_root:h}"
			if git clone --depth 1 "$repo_url" "$repo_root"; then
				echo "Clone complete: $repo_root"
			else
				echo "Failed to clone $repo_url"
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
		echo "Skipping missing image directory: $resolved_image_dir"
	fi
done

if (( ${#talespin_existing_image_dirs[@]} == 0 )); then
	echo "No valid custom image directories found; forcing built-in images on."
	TALESPIN_DISABLE_BUILTIN_IMAGES_P='n'
fi

export TALESPIN_EXTRA_IMAGE_DIRS="${(@F)talespin_existing_image_dirs}"
tmux set-environment -g TALESPIN_EXTRA_IMAGE_DIRS "$TALESPIN_EXTRA_IMAGE_DIRS"
tmux set-environment -g TALESPIN_DISABLE_BUILTIN_IMAGES_P "$TALESPIN_DISABLE_BUILTIN_IMAGES_P"
export TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P="${TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P:-y}"
tmux set-environment -g TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P "$TALESPIN_SNIFF_EXTENSIONLESS_IMAGES_P"
export TALESPIN_CACHE_DIR="${TALESPIN_CACHE_DIR:-~/.cache/talespin}"
tmux set-environment -g TALESPIN_CACHE_DIR "$TALESPIN_CACHE_DIR"
export TALESPIN_CARD_ASPECT_RATIO="${TALESPIN_CARD_ASPECT_RATIO:-2:3}"
tmux set-environment -g TALESPIN_CARD_ASPECT_RATIO "$TALESPIN_CARD_ASPECT_RATIO"
export TALESPIN_CARD_LONG_SIDE="${TALESPIN_CARD_LONG_SIDE:-1536}"
tmux set-environment -g TALESPIN_CARD_LONG_SIDE "$TALESPIN_CARD_LONG_SIDE"
export TALESPIN_CARD_CACHE_FORMAT="${TALESPIN_CARD_CACHE_FORMAT:-avif}"
tmux set-environment -g TALESPIN_CARD_CACHE_FORMAT "$TALESPIN_CARD_CACHE_FORMAT"
export TALESPIN_CARD_AVIF_ENCODER="${TALESPIN_CARD_AVIF_ENCODER:-native}"
tmux set-environment -g TALESPIN_CARD_AVIF_ENCODER "$TALESPIN_CARD_AVIF_ENCODER"
export TALESPIN_CARD_AVIF_THREADS="${TALESPIN_CARD_AVIF_THREADS:-auto}"
tmux set-environment -g TALESPIN_CARD_AVIF_THREADS "$TALESPIN_CARD_AVIF_THREADS"
export TALESPIN_VALIDATE_CACHE_HITS_P="${TALESPIN_VALIDATE_CACHE_HITS_P:-y}"
tmux set-environment -g TALESPIN_VALIDATE_CACHE_HITS_P "$TALESPIN_VALIDATE_CACHE_HITS_P"
##

tmuxnew talespin_backend zsh -lc 'cd ~/base/talespin/talespin-server && ./target/release/talespin-server'

if [[ "$TALESPIN_PRODUCTION_P" == [yY] ]]; then
	tmux kill-session -t talespin_frontend &> /dev/null || true
	zsh -lc 'source ~/.shared.sh; nvm-load; nvm use 20; cd ~/base/talespin; npm run build'
else
	tmuxnew talespin_frontend zsh -lc 'source ~/.shared.sh; nvm-load; nvm use 20; cd ~/base/talespin; npm run dev -- --host 127.0.0.1 --port 4173 --debug'
fi

TALESPIN_PRODUCTION_P="$TALESPIN_PRODUCTION_P" caddy reload --config ~/Caddyfile --adapter caddyfile
#: If you’re running multiple Caddy instances, a reload will contact the admin endpoint (localhost:2019 by default) and tell the instance to reload without restarting. Existing connections stay, and the new config takes effect.
# tmuxnew caddy caddy run --config ~/Caddyfile
