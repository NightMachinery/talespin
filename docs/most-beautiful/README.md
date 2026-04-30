# Most Beautiful

Most Beautiful is the optional Dixit/Talespin post-voting beauty round.

## Settings

- **Enable Most Beautiful**: adds a second vote after storyteller voting.
- **Beauty votes per player**: how many beauty votes each active player can cast. Default: `2`.
- **Allow duplicate beauty votes**: allows spending multiple beauty votes on the same card.
- **Split beauty bonus among ties**: when enabled, tied winning owners split the beauty bonus and each gets `ceil(bonus / tied_owner_count)`.
- **Beauty winner bonus**: the per-round beauty bonus before any tie split.
- **Beauty scoring**:
  - `vote_divisor`: each owner gets `floor(cumulative beauty votes in the current
vote-divisor segment on their submitted cards / K)`
  - `winner_bonus`: legacy top-card winner bonus flow
- **Beauty vote divisor `K`**:
  - default mode: `manual`
  - manual default: `3.0`
  - manual K uses `0.1` steps
  - `player_count_auto`: `K = round(players / base, 1 decimal)`, with configurable base defaulting to `4`
  - `median_auto`: `K = round(median(cumulative beauty votes received by current active players) / completed vote-divisor rounds, 1 decimal)`
  - all auto modes clamp the effective `K` to at least `1.0`
- **Beauty results display**:
  - `summary`: storyteller results include beauty totals only
  - `separate`: storyteller results and beauty results are separate stages
  - `combined`: storyteller and beauty chooser overlays appear together in results
- Vote-divisor beauty scores can go **down** later in the same game if the effective auto-`K` rises and the room rescales cumulative beauty totals.
- In `separate` results mode, a vote-divisor rescore during `Results` updates only the
  beauty points that were already awarded; the current round's rescored beauty delta is still
  applied when `BeautyResults` opens.
- Auto-`K` vote-divisor rescoring also runs immediately when an active player leaves the order,
  including removals and player→observer conversions.
- Vote-divisor scoring is computed on the server with integer tenths math, not floating-point
  `floor`; for example, `15` scoreable votes at manual `K = 3.0` scores `5` beauty points.
- Prior-round vote remainders carry forward within the active vote-divisor segment. For example,
  a player with `17` cumulative votes at manual `K = 3.0` has `5` beauty points; receiving `2`
  more votes awards `+1` because `floor((17 + 2) / 3) - 5 = 1`.
- When Clue Rating is enabled with Most Beautiful, the round flows `BeautyVoting` →
  `ClueRating` → `Results`. `ClueRating` is only an intermediate stage; entering results from
  it must still record the just-finished beauty delta before audit/history and then advance the
  vote-divisor cumulative segment state after audit/history.
- Switching from `winner_bonus` to `vote_divisor` starts a fresh vote-divisor segment. Prior Most
  Beautiful audit/ranking votes can still appear in the ranking panel, but they are not scoreable
  vote-divisor votes for the fresh segment.
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

The default Dixit leaderboard view is **Combined** for fresh clients / rooms, while existing saved
per-room client preferences keep their previous choice. If **Enable Most Beautiful** is off, the
leaderboard falls back to **Total** only and the view picker is hidden.

Moderators can force-push the current leaderboard view to everyone in the room. In Resonance/Stella mode, or whenever **Enable Most Beautiful** is off, Most Beautiful leaderboard controls are hidden and ignored.

## Since-joined leaderboard filter

Dixit leaderboards can be filtered live and at game end to start from the first round where the current viewer became an active player. Later joiners receive the simulated filtered minimum score, matching normal mid-game join behavior within that viewer's filtered leaderboard. The server computes the filtered score maps and the browser selects the current viewer's map. See `docs/talespin/leaderboards.md` for the shared Dixit/Stella behavior.

## Ranking panel

The **Most Beautiful ranking** panel is shown only during **mid-game** and on the **End** screen while **Enable Most Beautiful** is on. It is hidden in the lobby / joining view and whenever the feature is disabled.

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

The ranking panel totals come from this audit/statistics data. They are useful for historical
Most Beautiful rankings, but they are not the same as the active vote-divisor segment totals used
to award beauty points.
