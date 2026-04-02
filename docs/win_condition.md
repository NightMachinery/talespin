# Win Conditions

## Supported Win Conditions

The lobby host can now choose one win mode at game creation:

- `points`: game ends when at least one player reaches configured `target_points`
- `cycles`: game ends after configured number of **full storyteller cycles**
  - each **active player** must have been the storyteller at least `target_cycles` times
  - storyteller turns are counted when the storyteller submits their card + clue
  - next storyteller is chosen randomly from the active players tied for the lowest storyteller count
  - brand-new midgame joiners start at the current active-player minimum score and storyteller count
  - returning observers keep the existing score floor rules and get the same upward-only floor rule for storyteller count
  - observers do not block cycle completion
- `cards_finish`: game ends when the server cannot fully deal a new round from the current deck

Only one mode is active per room (single-mode selection).

## Default Points Threshold

The default for `points` mode is controlled by env var:

- `TALESPIN_DEFAULT_WIN_POINTS`

If unset or invalid, fallback is `10`.

## When Win Conditions Are Checked

- `points` and `cycles` are checked during `Ready` handling in round transition flow (`Joining`/`Results`)
- `cards_finish` is checked when starting a new round; if dealing fails due to deck depletion, stage transitions to `End`

## End-State Behavior

When any active condition is met:

- server sets room stage to `End`
- server sends `EndGame` to clients
- clients render the end screen with final ranking

## Cards Remaining / Refill UX

`RoomState` now includes:

- `cards_remaining`: current draw-pile size
- `deck_refill_count`: monotonic counter incremented when server refills the deck

Leaderboard shows cards left and flashes when `deck_refill_count` increases (refill event).

## Q: What Happens When Cards Run Out?

- In `points` / `cycles` modes:
  - server may refill the draw deck from base cards (excluding cards in hands)
  - clients see card count jump and flash indicator
- In `cards_finish` mode:
  - server does **not** refill
  - if next round cannot be fully dealt, game ends

## Ties and Multiple Players at Threshold

For points mode, if multiple players are at/above the threshold, game still ends.

There is no dedicated tie-break policy in code; standings are sorted by points descending, equal scores share the same displayed rank, and the next rank skips accordingly (for example `1, 1, 3`).

## Not Supported (Current Implementation)

- elimination mode
- sudden death mode
- hybrid multi-condition logic (AND/OR of multiple enabled win modes)

## Code References

- `talespin-server/src/main.rs` (`/create` payload parse + default points env fallback)
- `talespin-server/src/room.rs` (win-condition checks, cards-finish behavior, room-state deck metadata)
- `src/routes/+page.svelte` (lobby win-mode controls)
- `src/routes/game/[roomCode]/Leaderboard.svelte` (win label, cards-left display, refill flash)
