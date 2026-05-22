# Talespin moderator force actions

Current-stage moderator panels now expose two separate intervention buttons:

- **Force...** keeps the existing stage-specific behavior (`Switch Storyteller`, `Force Random`,
  `Force Skip`, `Force Random Reveal`, etc.).
- **Auto-observerify** converts any **offline active players who still owe an action in the current
  stage** into observers.
- **Reset Clue** appears during `PlayersChoose` for moderators as a compact icon button. It confirms,
  clears the storyteller's committed clue/card and nomination drafts, decrements that storyteller's
  current-game storyteller count for the reverted clue, and returns to `ActiveChooses`.
- **Force End Game** is available in moderator settings. It confirms client-side and ends a live game
  immediately.

Force actions count non-observer players' current unsubmitted drafts before filling or skipping:

- `PlayersChoose`: drafted card picks are deduplicated, used first, then missing center
  cards are random-filled.
- `Voting`: submitted ballots from active members still count after observer conversion, but stale
  observer-only drafts do not unlock forcing because they cannot be materialized. Non-observer
  drafted vote tokens are used first, then missing vote tokens are random-filled. Only the
  random-filled tokens are excluded from storyteller-outcome and decoy scoring.
- `BeautyVoting`: drafted Most Beautiful picks are counted as beauty votes; players with no
  submitted ballot or draft are skipped. Players may also intentionally submit zero beauty votes.
- `StellaAssociate`: drafted associations are locked in, preserving queue order; players with no
  draft or locked selection receive the force-random fallback.

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
  - end immediately when observerifying the `ActiveChooses` storyteller satisfies the configured
    win condition,
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
