# Sticky card navigator

This note covers the **local-only** Dixit view option that adds a small sticky card-number
navigator for the shared table layouts.

## Local preference

- Setting name: **Sticky card navigator**
- Scope: **per browser / per player device**
- Default: **off**
- Lives in the local **Options** card, not in moderator-only room settings

## Behavior

- When enabled, Dixit **Voting**, **BeautyVoting**, **Results**, and **BeautyResults** show a
  compact sticky bar above the card grid.
- The bar lists the round's **canonical card numbers** (`#1`, `#2`, etc.), even if the room's
  badge overlays are hidden.
- Clicking or tapping a number **scrolls to that card** and briefly highlights it.
- The navigator has a **Collapse** control. The collapsed/expanded state is also stored locally, so
  the bar stays collapsed on future visits until that player expands it again.

## Randomized voting layout interaction

- The navigator is especially useful when **Randomize voting card order per player** is enabled.
- Number chips always target the card with that **canonical number**, not the viewer's on-screen
  position.
- Voting stages also mirror local state in the bar:
  - selected cards get highlighted chips
  - locally disabled / non-votable cards get muted chips
