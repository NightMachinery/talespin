# Fit-To-Height Cards (Notes)

## Why this exists

Short record of the desktop "Cards fit to height" toggle behavior and fixes.

## Symptom

Cards were still stretching to container height on desktop even when the toggle was off.

## Root causes found

- `StageShell` used an always-on desktop `h-full` wrapper.
- Grid layout could stretch rows (`align-content: stretch`) in some states.
- `Results` view had hardcoded `h-full` sizing independent of the toggle.

## Fix pattern

- Keep `cardsFitToHeight` default `false`.
- Apply desktop `h-full` / fixed grid rows only when toggle is enabled.
- Derive desktop fit row counts from the actual number of cards (`ceil(cardCount / 3)`, min `2`) instead of hardcoding two rows.
- In non-fit mode, prefer `auto-rows-max` + `content-start` so cards keep natural size and scroll.
- Keep hand, voting, and results on the same row-count helper/pattern so one screen cannot drift from the others.
- Keep mobile behavior unchanged (always scroll-oriented).

## Ops note

If UI looks stale after changes, hard refresh once to bypass transient HMR/cache state.
