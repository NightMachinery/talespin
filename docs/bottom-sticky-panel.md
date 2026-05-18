# Bottom sticky panel

The mobile bottom sticky panel is the small fixed action area rendered through
`StageShell`'s `mobileActions` slot. Keep this panel compact, predictable, and easy to
edit by hand.

## Shared controls

- Use `BottomStickyPanelViewSwitch.svelte` for local view switches such as **Table**,
  **Previous Results**, and **My Cards**.
- The view switch default lives in `src/lib/bottomStickyPanel.ts`:
  `BOTTOM_STICKY_PANEL_VIEW_PRESENTATION`.
  - `icon` renders compact icon tabs.
  - `text` renders labeled segmented buttons.
- Use `BottomStickyPanelActionGroup.svelte` for grouped moderator actions that belong together.
- The grouped-action default lives in `src/lib/bottomStickyPanel.ts`:
  `BOTTOM_STICKY_PANEL_ACTION_LAYOUT`.
  - `row` puts the action group into one equal-width row.
  - `stack` gives each action a full-width row for desktop/sidebar contexts.

## Current conventions

- View switches are icon tabs by default. They still carry labels through `aria-label` and
  `title`, so the labels remain available to assistive tech and browser tooltips.
- Moderator action groups use compact row buttons by default. Prefer icon plus a short visible
  label for actions that are not obvious.
- Icon-only action buttons are only appropriate for obvious reset-style controls, and they must
  provide a full `aria-label`, `tooltip`, and confirmation before changing shared room state.
- Compact action buttons show a custom tooltip on hover, keyboard focus, and touch long-press.
  Do not rely on native `title` tooltips for mobile comprehension.
- Primary player actions such as **Choose**, **Submit Votes**, and **Next Round** remain
  full-width buttons.
- Reset confirmations currently apply to the mobile sticky controls for **Reset clue** and
  **Reset board**. Force/random/skip/reveal actions stay fast.

## Stages using the shared view switch

- `ActiveChooses`: **Previous Results** / **My Cards**
- `PlayersChoose`: **Previous Results** / **My Cards**
- `Voting`: **Table** / **My Cards**
- `BeautyVoting`: **Table** / **My Cards**
- `Results`: **Table** / **My Cards**
- `BeautyResults`: **Table** / **My Cards**

## Editing notes

To switch the bottom panel from icons to text labels, change:

```ts
export const BOTTOM_STICKY_PANEL_VIEW_PRESENTATION: BottomStickyPanelViewPresentation = 'text';
```

To make grouped bottom-panel actions stack full-width, change:

```ts
export const BOTTOM_STICKY_PANEL_ACTION_LAYOUT: BottomStickyPanelActionLayout = 'stack';
```

Prefer changing those defaults before editing individual stage files. Override a single component
instance only when one stage needs a deliberate exception.
