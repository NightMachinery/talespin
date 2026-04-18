# Recent changes

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
