# Stage change cues

Local view options now cover both **stage change sound cues** and **stage change visual cues**.

## Behavior

- Cues fire on real in-room stage transitions after the first room state has loaded.
- Initial room load / refresh stays silent and does not flash, even if the room is already mid-game.
- Supported stage entries include:
  - Dixit / Talespin: `ActiveChooses`, `PlayersChoose`, `Voting`, `BeautyVoting`, `ClueRating`,
    `Results`
  - Resonance: `StellaAssociate`, `StellaReveal`, `StellaResults`
  - shared states: `Paused`, `End`
- Visual cue is a short screen-edge flash with a top accent bar.
- Sound cue uses short Web Audio tones. The same sound toggle also covers the Resonance scout-turn cue.

## Persistence and support

- Both toggles are **local-only** and persist in `localStorage`.
- Sound and visual cues can be enabled independently.
- Browsers without Web Audio support simply disable the sound toggle behavior; visual cues still work.
