# Leaderboards

Talespin/Dixit and Resonance/Stella leaderboards can show raw room scores or a local
**since I joined** view.

## Personal since-joined view

When the checkbox is available, it recomputes each leaderboard row as that member's personal
score since they first became an active player.

- A member's first active round is the first recorded round whose active-player list includes their
  name.
- Each active member starts from `0` at their own first active round. Automatic mid-game join-floor
  points are excluded; only points earned in that round and later rounds are shown.
- Original players who were active in the first recorded round still match the normal leaderboard.
- The client records a per-member snapshot offset at that member's first active round, then
  subtracts that offset from later cumulative `total_after_round` and `beauty_total_after_round`
  snapshots. This keeps later score changes aligned with server history instead of summing deltas
  from scratch.
- Live active players whose first round has not been recorded yet show `0` in the filtered view.
- Scored observers with no active-player history keep their raw score because they have no personal
  active-player join round to filter from.
- The view is local to the current browser session and defaults to off.
- Dixit score views that use total/story/beauty values can be filtered; the Clue Stars view is not
  filtered.
- Stella uses the same round-history replay, with total-only scoring.

The server sends compact leaderboard round history in room state so the live sidebar and end-game
leaderboards use the same calculation.
