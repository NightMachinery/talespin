# Win Conditions

## Supported Win Conditions

Currently, the game supports one win/end condition:

- the game ends when at least one player reaches **10 or more points** (`>= 10`).

There are no alternate victory modes or configurable point thresholds implemented.

## When the Win Condition Is Checked

The server checks for game end when processing a `Ready` message in `Joining`/`Results` flow (round transition time).

This means:

- the game does not end immediately in the middle of vote/score calculation UI updates
- it ends at the next round-transition check once `max_points >= 10`

## End-State Behavior

When the condition is met:

- server sets room stage to `End`
- server sends `EndGame` to clients
- clients render the end screen with final ranking

## Ties and Multiple Players at Threshold

If multiple players are at or above 10 points, the game still ends (same threshold check).

There is no explicit tie-breaker rule in code; end ranking is sorted by points descending, and equal-score ordering is not defined by a dedicated tie-break policy.

## Not Supported (Current Implementation)

- custom win threshold (for example first to 15)
- alternate win conditions (round count, elimination, sudden death, etc.)

## Code References

- `talespin-server/src/room.rs` (win check and transition to `End`)
- `src/routes/game/[roomCode]/Leaderboard.svelte` (`First to 10 points!` UI hint)
- `src/routes/game/[roomCode]/End.svelte` (game-over ranking presentation)
