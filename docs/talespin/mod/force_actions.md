# Talespin moderator force actions

Current-stage moderator panels now expose two separate intervention buttons:

- **Force...** keeps the existing stage-specific behavior (`Switch Storyteller`, `Force Random`,
  `Force Skip`, `Force Random Reveal`, etc.).
- **Auto-observerify** converts any **offline active players who still owe an action in the current
  stage** into observers.

## Auto-observerify behavior

- It only appears on the **current-stage** moderator force controls:
  - `ActiveChooses`
  - `PlayersChoose`
  - `Voting`
  - `BeautyVoting`
  - `ClueRating`
  - `StellaAssociate`
  - manual `StellaReveal`
- It is hidden during queued/auto-play Stella reveal, because there is no pending scout input to
  clear there.
- It only targets **offline** blockers; connected unready players are never observerified by this
  action.
- After each conversion, the server re-runs the normal stage reconciliation logic. That means the
  room may:
  - stay on the same stage with a new active player/scout,
  - advance once enough remaining players are ready,
  - or pause if the room drops below the minimum active-player count.

## Stage targeting

- `ActiveChooses`: offline current storyteller.
- `PlayersChoose`: offline non-storytellers who have not locked cards yet.
- `Voting`: offline non-storytellers who have not submitted votes yet.
- `BeautyVoting`: offline players who have not submitted beauty votes yet.
- `ClueRating`: offline non-storytellers who have not submitted a star rating yet.
- `StellaAssociate`: offline players who have not locked selections yet.
- Manual `StellaReveal`: offline current scout.
