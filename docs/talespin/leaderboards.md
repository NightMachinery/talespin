# Leaderboards

Talespin/Dixit and Resonance/Stella leaderboards can show raw room scores or a local
**since I joined** view.

## Since-I-joined view

When the checkbox is available, it recomputes the leaderboard as if the current viewer's first
active round were the start of the game.

- The viewer's first active round is the first recorded round whose active-player list includes
  their name.
- Players active in that first filtered round start from `0` in the filtered view.
- Players who become active later start at the current minimum filtered score, matching the normal
  mid-game join floor behavior.
- The view is local to the current browser session and defaults to off.
- Dixit score views that use total/story/beauty values can be filtered; the Clue Stars view is not
  filtered.
- Stella uses the same round-history replay, with total-only scoring.

The server sends compact leaderboard round history in room state so the live sidebar and end-game
leaderboards use the same calculation.
