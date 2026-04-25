# Talespin Ops Notes

This folder contains practical notes for running and maintaining the self-hosted Talespin deployment.

Tooling note: frontend lint/format runs intentionally ignore generated outputs such as `build/`, `.svelte-kit/`, and `talespin-server/target/`.
Deploy note: routine Talespin redeploys now keep benign Browserslist/Caddy noise out of successful runs and skip dangling-symlink image entries from custom card folders.

- `docs/self-hosting.md`: deployment layout, tmux scripts, ports, and checks
- `docs/images.md`: where card images live and what constraints apply
- `docs/win_condition.md`: supported win modes, defaults, and deck-finish behavior
- `docs/user_auth.md`: player identity persistence, room-scoped device migration links, and password-carrying migrate URLs
- `docs/fit_to_height_cards.md`: fit-to-height option notes and caveats
- `docs/talespin/dixit_previous_results_preview.md`: Dixit storyteller-stage previous-results preview and next-round auto-advance
- `docs/talespin/leaderboards.md`: live and end-game leaderboard views, including since-joined filtering for Dixit and Stella
- `docs/talespin/clue_rating.md`: Dixit clue-rating stage, storyteller star bonus, and clue-stars leaderboard
- `docs/talespin/mod/options.md`: Dixit moderator option lock windows after storyteller clue lock
- `docs/talespin/mod/force_actions.md`: current-stage moderator force buttons, including auto-observerify for offline blockers
- `docs/talespin/scoring/storyteller_success.md`: configurable storyteller success points and how they differ from `W`
- `docs/talespin/scoring/double_vote_bonus.md`: configurable double-vote bonuses for normal, too-many-wrong, and too-many-correct Dixit rounds
- `docs/talespin/dixit_randomized_voting_layout.md`: Dixit per-viewer randomized voting layout with canonical number badges
- `docs/talespin/sticky_card_navigator.md`: local sticky card-number navigator for Dixit voting and results
- `docs/talespin/stage_change_cues.md`: local stage-change sound + visual cues, supported stages, and persistence
- `docs/talespin/stage_timers.md`: shared countdown sync behavior for live stage changes, reconnects, and untimed stages
- `docs/most-beautiful/README.md`: Most Beautiful settings, leaderboard modes, tie splitting, and audit history

# Unrelated Docs for Other Projects

- `docs/secret-hitler-users.md`: portable product spec for any-time-join + creator/mod/observer dynamics in Secret Hitler
