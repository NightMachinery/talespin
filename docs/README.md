# Talespin Ops Notes

This folder contains practical notes for running and maintaining the self-hosted Talespin deployment.

Tooling note: frontend lint/format runs intentionally ignore generated outputs such as `build/`, `.svelte-kit/`, and `talespin-server/target/`.
Deploy note: routine Talespin redeploys now keep benign Browserslist/Caddy noise out of successful runs and skip dangling-symlink image entries from custom card folders.

- `docs/self-hosting.md`: deployment layout, tmux scripts, ports, and checks
- `docs/images.md`: where card images live and what constraints apply
- `docs/win_condition.md`: supported win modes, defaults, and deck-finish behavior
- `docs/user_auth.md`: player identity persistence across refresh
- `docs/fit_to_height_cards.md`: fit-to-height option notes and caveats
- `docs/talespin/dixit_previous_results_preview.md`: Dixit storyteller-stage previous-results preview and next-round auto-advance
- `docs/talespin/mod/options.md`: Dixit moderator option lock windows after storyteller clue lock
- `docs/talespin/dixit_randomized_voting_layout.md`: Dixit per-viewer randomized voting layout with canonical number badges
- `docs/talespin/sticky_card_navigator.md`: local sticky card-number navigator for Dixit voting and results
- `docs/talespin/stage_change_cues.md`: local stage-change sound + visual cues, supported stages, and persistence
- `docs/most-beautiful/README.md`: Most Beautiful settings, leaderboard modes, tie splitting, and audit history

# Unrelated Docs for Other Projects

- `docs/secret-hitler-users.md`: portable product spec for any-time-join + creator/mod/observer dynamics in Secret Hitler
