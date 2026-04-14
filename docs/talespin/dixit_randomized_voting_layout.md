# Dixit per-viewer randomized voting layout

This note covers the Dixit-only moderator toggle that randomizes each viewer's card layout during
voting while keeping the canonical number badges.

## Room setting

- Setting name: **Randomize voting card order per player**
- Scope: **room-global**
- Default: **off**
- Editable during any live Dixit stage: **ActiveChooses**, **PlayersChoose**, **Voting**,
  **BeautyVoting**, **Results**, and **BeautyResults**
- Available from the live game sidebar (not the Joining screen)

## Voting-stage behavior

- When enabled, each player and observer gets a **personal deterministic shuffle** during
  `Voting` and `BeautyVoting`.
- The shuffle is **stable for the whole round**.
- Turning the toggle **off** and then **on** again during the same round restores the same layout
  for that viewer.
- The shuffle is derived from the round's voting seed plus the viewer name, so different viewers
  can see different layouts.

## Number badges and results

- The visible `#1`, `#2`, etc. badges always reflect the **canonical server order** for the round,
  not the viewer's shuffled layout.
- `Results`, `BeautyResults`, and the next round's previous-results preview always show the
  **canonical/original** order for everyone.
- Toggling the setting during results updates the room setting immediately, but results themselves
  stay canonical.

## Local navigation aid

- The local **Sticky card navigator** view option is enabled by default, and players can still turn
  it off if they prefer.
- That navigator stays client-side, shows the same canonical numbers, and can jump to any card in
  `Voting`, `BeautyVoting`, `Results`, and `BeautyResults`.
