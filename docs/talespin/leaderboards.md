# Leaderboards

Talespin/Dixit and Resonance/Stella leaderboards can show raw room scores or a local
**since I joined** view.

## Since-I-joined view

When the checkbox is available, it recomputes the leaderboard as if the current viewer's first
active round were the start of the game.

- The viewer's first active round is the first recorded round whose active-player list includes
  their name.
- Replaying from the first recorded round is an identity operation: the filtered totals match the
  normal leaderboard because the replay uses the server's cumulative `total_after_round` and
  `beauty_total_after_round` snapshots as authoritative history.
- For later cutoffs, the client records a per-member snapshot offset the first time that member
  appears in the filtered replay, then subtracts that offset from later cumulative snapshots. This
  keeps later score changes aligned with server history instead of summing deltas from scratch.
- Players active in the first filtered round start from `0` in the filtered view.
- Players who first appear as active players after the viewer's cutoff start at the current minimum
  filtered active-player score, matching normal mid-game join floor behavior within the simulated
  leaderboard.
- Live active players whose first scored round has not been recorded yet also start at the current
  minimum filtered active-player score.
- Non-active scored members and observers start from `0` when they first appear; they do not receive
  the active-player join floor unless they first enter the filtered replay as active players.
- The view is local to the current browser session and defaults to off.
- Dixit score views that use total/story/beauty values can be filtered; the Clue Stars view is not
  filtered.
- Stella uses the same round-history replay, with total-only scoring.

The server sends compact leaderboard round history in room state so the live sidebar and end-game
leaderboards use the same calculation.
