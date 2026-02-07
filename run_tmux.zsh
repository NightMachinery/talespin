#!/usr/bin/env zsh

tmuxnew () {
	tmux kill-session -t "$1" &> /dev/null || true
	tmux new -d -s "$@"
}

export ALL_PROXY=http://127.0.0.1:1087 all_proxy=http://127.0.0.1:1087 http_proxy=http://127.0.0.1:1087 https_proxy=http://127.0.0.1:1087 HTTP_PROXY=http://127.0.0.1:1087 HTTPS_PROXY=http://127.0.0.1:1087

tmuxnew talespin_backend zsh -lc 'cd ~/base/talespin/talespin-server && ./target/release/talespin-server'
tmuxnew talespin_frontend zsh -lc 'source ~/.shared.sh; nvm-load; nvm use 20; cd ~/base/talespin; npm run dev -- --host 127.0.0.1 --port 4173'
