# Recent changes

- 2026-04-28: Moderators now have a local Dev mode checkbox in the moderation sidebar. When enabled,
  **Copy Current Info** copies Markdown with the current room/game/round context and ready-to-run
  `sqlite3` lookup commands for the current and previous Dixit audit rounds.

- 2026-04-28: Dixit `PlayersChoose` now restores selected-card highlights after refresh/reconnect.
  Submitted nominations are replayed from the server to the submitting player only, while unsent
  local draft selections can rehydrate from browser session storage until the stage changes.

- 2026-04-22: Shared stage timers now resync immediately on live stage changes and reconnects. Stage-specific room payloads carry the same timer metadata as `RoomState`, and untimed stages clear the countdown without needing a page refresh.

- 2026-04-20: Dixit double-vote bonus is now split into three numeric settings: **Normal round**, **Too many guessed wrong**, and **Too many guessed correctly**. The two storyteller-loss rows default to **Follow normal**, the old double-vote storyteller-loss checkbox is gone, and the score cheatsheet / moderator sidebar now explain the branch-specific behavior.

- 2026-04-18: Dixit now has an optional **Clue Rating** stage between storyteller / beauty voting
  and results. Non-storytellers can submit configurable star counts, the storyteller gets
  `max(round(avg) - 1, 0)` bonus points, Results shows the round clue-star summary, and the live /
  end leaderboards now support a persistent **Clue Stars** view plus a temporary results-only
  per-player star column beside names.

- 2026-04-14: Dixit previous-results preview now stays available in both `ActiveChooses` and
  `PlayersChoose`. Storytellers can open it during clue selection, active players can switch between
  **Previous Results** and **My Cards**, and observers get the preview-only version. The server now
  keeps the prior-results snapshot in room state through `PlayersChoose` so reload/reconnect keeps
  working there too.
