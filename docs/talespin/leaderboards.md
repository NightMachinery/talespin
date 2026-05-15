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
- For later cutoffs, the server records a per-member snapshot offset the first time that member
  appears in the filtered replay, then subtracts that offset from later cumulative snapshots. This
  keeps later score changes aligned with server history instead of summing deltas from scratch.
- Players active in the first filtered round start from `0` in the filtered view.
- Players who first appear as active players after the viewer's cutoff start at the current minimum
  filtered active-player score, matching normal mid-game join floor behavior within the simulated
  leaderboard.
- Pending observers who appear in score snapshots before their first active round do not anchor a
  zero-point offset; if they become active later, they receive that later round's simulated filtered
  minimum instead.
- Live active players whose first scored round has not been recorded yet are included in available
  cutoff rounds immediately. In older viewers' filtered leaderboards they start at the current
  minimum filtered active-player score; in their own current-round cutoff, all active players start
  from `0` until an in-scope scored round is recorded.
- Non-active scored members and observers start from `0` when they first appear; they do not receive
  the active-player join floor unless they first enter the filtered replay as active players.
- The view is local to the current browser session and defaults to off.
- Dixit score views that use total/story/beauty values can be filtered; the Clue Stars view is not
  filtered.
- Stella uses the same round-history replay, with total-only scoring.

The server sends both compact leaderboard round history and precomputed since-joined score maps in
room state. The live sidebar and end-game screens select the current viewer's first-active-round
map instead of replaying scoring rules in the browser.

## Delta display

The frontend uses one delta-score selector for sidebar leaderboard deltas. It first returns the
current round's delta scores when the room is in a result stage. If no current-round delta payload
is available, it can fall back to the previous round's cached Dixit result deltas.

- `TALESPIN_DELTA_DISPLAY_MODE=always` is the default and enables previous-round fallback whenever
  previous deltas are available.
- `TALESPIN_DELTA_DISPLAY_MODE=only_in_previous_results` only enables previous-round fallback while
  the viewer's local view is **Previous Results**.
- The server includes current delta scores in room state for result stages and sends previous Dixit
  results in `StartRound` and `PlayersChoose` stage messages.

## Ranking ties

Leaderboard ranks use shared competition-ranking logic. Equal displayed values share the same rank
and the following rank skips ahead, e.g. `1, 1, 3`. This applies to total/story/beauty views and to
Clue Stars, so same-average clue-rating players display with the same rank.
