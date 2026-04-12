# Most Beautiful

Most Beautiful is the optional Dixit/Talespin post-voting beauty round.

## Settings

- **Enable Most Beautiful**: adds a second vote after storyteller voting.
- **Beauty votes per player**: how many beauty votes each active player can cast.
- **Allow duplicate beauty votes**: allows spending multiple beauty votes on the same card.
- **Split beauty bonus among ties**: when enabled, tied winning owners split the beauty bonus and each gets `ceil(bonus / tied_owner_count)`.
- **Beauty winner bonus**: the per-round beauty bonus before any tie split.
- **Beauty results display**:
  - `summary`: storyteller results include beauty totals only
  - `separate`: storyteller results and beauty results are separate stages
  - `combined`: storyteller and beauty chooser overlays appear together in results
- Beauty result badges show rank plus total beauty votes per card, such as `1st Beauty: 3`.
- Beauty ranks use competition ranking for ties (`1st`, `1st`, `3rd`).
- Top-three beauty badges use distinct gold/silver/bronze styling, and there is no separate winner pill.

## Leaderboards

Dixit leaderboards expose four view modes:

- **Total**: raw score
- **Story Only**: total minus accumulated beauty points
- **Beauty Only**: accumulated beauty points only
- **Combined**: aligned `T / S / B` columns for total, story, and beauty

Moderators can force-push the current leaderboard view to everyone in the room. In Resonance/Stella mode, Most Beautiful leaderboard controls are hidden and ignored.

## End-game history

The end-game Dixit leaderboard can be filtered to start from the first round where the current viewer became an active player.

The server also persists per-game Most Beautiful audit data in SQLite, including:

- clue + storyteller per round
- which player submitted each card
- storyteller votes and beauty votes
- per-player story delta, beauty delta, and running totals
