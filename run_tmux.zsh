#!/usr/bin/env zsh

tmuxnew () {
	tmux kill-session -t "$1" &> /dev/null || true
	tmux new -d -s "$@"
}

export ALL_PROXY=http://127.0.0.1:1087 all_proxy=http://127.0.0.1:1087 http_proxy=http://127.0.0.1:1087 https_proxy=http://127.0.0.1:1087 HTTP_PROXY=http://127.0.0.1:1087 HTTPS_PROXY=http://127.0.0.1:1087

typeset -a talespin_custom_image_dirs=(
	~/Pictures/Dixit/great
)
export TALESPIN_EXTRA_IMAGE_DIRS="${(@F)talespin_custom_image_dirs}"
tmux set-environment -g TALESPIN_EXTRA_IMAGE_DIRS "$TALESPIN_EXTRA_IMAGE_DIRS"
export TALESPIN_DISABLE_BUILTIN_IMAGES_P="${TALESPIN_DISABLE_BUILTIN_IMAGES_P:-n}"
tmux set-environment -g TALESPIN_DISABLE_BUILTIN_IMAGES_P "$TALESPIN_DISABLE_BUILTIN_IMAGES_P"

tmuxnew talespin_backend zsh -lc 'cd ~/base/talespin/talespin-server && ./target/release/talespin-server'
tmuxnew talespin_frontend zsh -lc 'source ~/.shared.sh; nvm-load; nvm use 20; cd ~/base/talespin; npm run dev -- --host 127.0.0.1 --port 4173'
