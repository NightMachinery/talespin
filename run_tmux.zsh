#!/usr/bin/env zsh
emulate -L zsh -o errexit -o nounset -o pipefail

print -u2 -- 'run_tmux.zsh is deprecated; forwarding to ./self_host.zsh redeploy'
case "${1:-}" in
	'' )
		exec "${0:A:h}/self_host.zsh" redeploy
		;;
	setup|redeploy|start|stop|-h|--help|help)
		exec "${0:A:h}/self_host.zsh" "$@"
		;;
	*)
		exec "${0:A:h}/self_host.zsh" redeploy "$@"
		;;
esac
