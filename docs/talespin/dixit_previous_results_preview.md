# Dixit storyteller-choosing previous-results preview

This note covers the Dixit-only moderator toggle that keeps the previous results visible while the
next storyteller is choosing and while nominations are happening.

## Room setting

- Setting name: **show previous results during storyteller choosing**
- Scope: **room-global**
- Default: **on**
- Editable during Dixit **Joining** and any live Dixit stage: **ActiveChooses**,
  **PlayersChoose**, **Voting**, **BeautyVoting**, **Results**, and **BeautyResults**

## ActiveChooses behavior

- **Current storyteller** defaults to the normal hand + clue UI and can switch to **Previous
  Results**.
- **Waiting active players** default to a local **Previous Results** view and can switch to **My
  Cards**.
- **Observers** keep the previous-results view only.
- The local view choice resets each round: storytellers default to **My Cards**, while waiting
  active players default to **Previous Results**.

## PlayersChoose behavior

- **Storyteller** and **guessers** default to **My Cards** and can switch to **Previous Results**.
- **Observers** keep the previous-results view only.
- Switching views is local-only and does not clear current nominations or the storyteller card
  highlight.

## What is shown

- If the last completed Dixit stage was normal storyteller `Results`, the preview mirrors that
  results board.
- If Most Beautiful uses **separate** results, the next `ActiveChooses` preview mirrors the prior
  `BeautyResults` board instead.
- The data is sent in room state so reconnecting players can still see the same preview.
- Preview card highlights mirror live Dixit results states: correct card = animated green sweep,
  beauty winner = silver border, overlap = green sweep with a moving silver glint.

## Next-round auto-advance

- On the final Dixit results screen, the first player who clicks **Next Round** and is among the
  current lowest storyteller-count candidates immediately becomes the next storyteller.
- Non-eligible clicks still only mark that player ready.
