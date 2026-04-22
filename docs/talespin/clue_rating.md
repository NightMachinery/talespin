# Clue rating stage

Talespin now supports an optional Dixit-only **Clue Rating** stage between storyteller/beauty
voting and results.

## What it does

- Only **non-storytellers** can rate the clue.
- Each rater submits **1..N stars**, where `N` is room-configurable.
- The storyteller gets a round bonus of:

`max(round(average submitted stars) - 1, 0)`

- Missing raters are skipped instead of counting as zero.
- If **nobody** submits a rating, the storyteller gets **+0** and that round records **no clue-star
  average** for leaderboard purposes.

## Moderator settings

- **Enable clue rating stage**
- **Max stars**: `1..10`
- **Clue rating timer enabled**
- **Clue rating timer duration**: default `20s`
- **Force timeout by skipping missing star votes**

Entry settings lock when `ClueRating` starts. Timer settings stay editable through live Dixit
stages like the other stage timers.

When Most Beautiful is enabled for the room, the clue-rating stage keeps the normal Dixit sidebar
leaderboard modes available too, including the **Combined** view.

## Results + leaderboards

- Live `Results` now shows the round's **average clue stars**, submitted count, and storyteller
  clue bonus.
- During live `Results`, the sidebar leaderboard also shows a temporary **per-player star column**
  beside names for submitted clue ratings in that round. It disappears on the next stage.
- If players switch to the **previous results** preview during the next storyteller-choosing /
  nominations stages, that same temporary per-player star column is also shown there for the
  previewed round when ratings were recorded.
- A persistent **Clue Stars** leaderboard view shows each player's average recorded storyteller
  clue rating to **1 decimal**, or `NA` if they have never recorded one.
