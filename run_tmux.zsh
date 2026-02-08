#!/usr/bin/env zsh

tmuxnew () {
	tmux kill-session -t "$1" &> /dev/null || true
	tmux new -d -s "$@"
}

export ALL_PROXY=http://127.0.0.1:1087 all_proxy=http://127.0.0.1:1087 http_proxy=http://127.0.0.1:1087 https_proxy=http://127.0.0.1:1087 HTTP_PROXY=http://127.0.0.1:1087 HTTPS_PROXY=http://127.0.0.1:1087

export TALESPIN_PRODUCTION_P="${TALESPIN_PRODUCTION_P:-n}"
##
export TALESPIN_DISABLE_BUILTIN_IMAGES_P="${TALESPIN_DISABLE_BUILTIN_IMAGES_P:-y}"
typeset -a talespin_custom_image_dirs=(
	# ~/Pictures/SurrealPictures
	~/Pictures/SurrealPictures/chosen_1
	~/base/virtualtabletop/library/games/Dreamcatcher/assets
)
export TALESPIN_EXTRA_IMAGE_DIRS="${(@F)talespin_custom_image_dirs}"
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
##

tmuxnew talespin_backend zsh -lc 'cd ~/base/talespin/talespin-server && ./target/release/talespin-server'

if [[ "$TALESPIN_PRODUCTION_P" == [yY] ]]; then
	tmux kill-session -t talespin_frontend &> /dev/null || true
	zsh -lc 'source ~/.shared.sh; nvm-load; nvm use 20; cd ~/base/talespin; npm run build'
else
	tmuxnew talespin_frontend zsh -lc 'source ~/.shared.sh; nvm-load; nvm use 20; cd ~/base/talespin; npm run dev -- --host 127.0.0.1 --port 4173 --debug'
fi

TALESPIN_PRODUCTION_P="$TALESPIN_PRODUCTION_P" caddy reload --config ~/Caddyfile --adapter caddyfile
