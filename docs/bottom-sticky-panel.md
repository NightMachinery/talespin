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
  - `stack` gives each action a full-width row.
  - `row` puts the action group into one equal-width row.

## Current conventions

- View switches are icon tabs by default. They still carry labels through `aria-label` and
  `title`, so the labels remain available to assistive tech and browser tooltips.
- Real actions stay labeled. Icon-only action buttons are avoided in the bottom sticky panel
  because they can be unclear and can waste a full row when mixed with stacked actions.
- Primary player actions such as **Choose**, **Submit Votes**, and **Next Round** remain
  full-width buttons.
- Moderator action groups can be stacked or changed to a single row through the shared action
  group layout. `PlayersChoose` uses this for **Reset clue**, **Force Random**, and
  **Auto-observerify**.

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

To make grouped bottom-panel actions occupy a single row, change:

```ts
export const BOTTOM_STICKY_PANEL_ACTION_LAYOUT: BottomStickyPanelActionLayout = 'row';
```

Prefer changing those defaults before editing individual stage files. Override a single component
instance only when one stage needs a deliberate exception.
