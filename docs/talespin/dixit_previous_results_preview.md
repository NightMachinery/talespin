# Dixit storyteller-choosing previous-results preview

This note covers the Dixit-only moderator toggle that keeps the previous results visible while the
next storyteller is choosing.

## Room setting

- Setting name: **show previous results during storyteller choosing**
- Scope: **room-global**
- Default: **on**
- Editable during Dixit **Joining** and any live Dixit stage: **ActiveChooses**,
  **PlayersChoose**, **Voting**, **BeautyVoting**, **Results**, and **BeautyResults**

## ActiveChooses behavior

- **Current storyteller** keeps the normal hand + clue UI.
- **Waiting active players** default to a local **Previous Results** view and can switch to **My
  Cards**.
- **Observers** keep the previous-results view only.
- The local view choice resets to **Previous Results** each new round.

## What is shown

- If the last completed Dixit stage was normal storyteller `Results`, the preview mirrors that
  results board.
- If Most Beautiful uses **separate** results, the next `ActiveChooses` preview mirrors the prior
  `BeautyResults` board instead.
- The data is sent in room state so reconnecting players can still see the same preview.

## Next-round auto-advance

- On the final Dixit results screen, the first player who clicks **Next Round** and is among the
  current lowest storyteller-count candidates immediately becomes the next storyteller.
- Non-eligible clicks still only mark that player ready.
