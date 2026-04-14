# Most Beautiful

Most Beautiful is the optional Dixit/Talespin post-voting beauty round.

## Settings

- **Enable Most Beautiful**: adds a second vote after storyteller voting.
- **Beauty votes per player**: how many beauty votes each active player can cast. Default: `2`.
- **Allow duplicate beauty votes**: allows spending multiple beauty votes on the same card.
- **Split beauty bonus among ties**: when enabled, tied winning owners split the beauty bonus and each gets `ceil(bonus / tied_owner_count)`.
- **Beauty winner bonus**: the per-round beauty bonus before any tie split.
- **Beauty scoring**:
  - `vote_divisor` (default): each owner gets `floor(total beauty votes on their submitted cards / K)`
  - `winner_bonus`: legacy top-card winner bonus flow
- **Beauty vote divisor `K`**: configurable divisor used by `vote_divisor`. Default: `3`.
- **Beauty results display**:
  - `summary`: storyteller results include beauty totals only
  - `separate`: storyteller results and beauty results are separate stages
  - `combined`: storyteller and beauty chooser overlays appear together in results
- Beauty result badges show rank plus total beauty votes per card, such as `1st Beauty: 3`.
- Beauty ranks use competition ranking for ties (`1st`, `1st`, `3rd`).
- Top-three beauty badges use distinct gold/silver/bronze styling, and there is no separate winner pill.
- On Dixit results boards, the storyteller's correct card uses an animated green sweep border, beauty
  winners use a distinct silver border, and a card that is both correct and a beauty winner uses
  the green sweep with a moving silver glint.

## Leaderboards

Dixit leaderboards expose four view modes:

- **Total**: raw score
- **Story Only**: total minus accumulated beauty points
- **Beauty Only**: accumulated beauty points only
- **Combined**: aligned `T / S / B` columns for total, story, and beauty

Moderators can force-push the current leaderboard view to everyone in the room. In Resonance/Stella mode, Most Beautiful leaderboard controls are hidden and ignored.

## End-game history

The end-game Dixit leaderboard can be filtered to start from the first round where the current viewer became an active player.

## Ranking panel

The **Most Beautiful ranking** panel is shown only during **mid-game** and on the **End** screen. It is hidden in the lobby / joining view.

- The mid-game sidebar shows it **after** the score cheatsheet.
- The panel supports **Last N games** filtering:
  - `0` = all history
  - default `1`
  - if the current room has an in-progress Dixit game, that game counts inside `N`
  - remaining slots come from the latest **completed** games globally
- Rankings are capped to the **top 30 players** server-side.

The server also persists per-game Most Beautiful audit data in SQLite, including:

- clue + storyteller per round
- which player submitted each card
- storyteller votes and beauty votes
- per-player story delta, beauty delta, and running totals
