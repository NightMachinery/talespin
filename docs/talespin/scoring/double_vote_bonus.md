# Double-vote bonus

Talespin now lets moderators configure the extra reward for a **double-correct** vote: when a guesser puts **2 or more vote tokens** on the storyteller card in a round with multi-vote guessing enabled.

## Available settings

There are three double-vote bonus values:

- **Normal round**: used when the storyteller wins the round.
- **Too many guessed wrong**: used when the storyteller loses because too many guessers missed.
- **Too many guessed correctly**: used when the storyteller loses because too many guessers found the storyteller card.

Each storyteller-loss row has a **Follow normal** checkbox.

- When follow is on, that branch always mirrors the current normal-round value.
- When follow is turned off, the branch starts from the current normal value and can then be edited separately.

Default and bounds:

- normal default: `1`
- too-many-wrong default: follow normal
- too-many-correct default: follow normal
- allowed range for every branch: `0..10`

## What it changes

- The bonus is awarded **once per eligible guesser per round**.
- It does **not** stack per extra correct token beyond the `2+` threshold.
- Setting a branch bonus to `0` disables the extra reward for that branch.

## Relation to the storyteller-loss scope toggle

The existing storyteller-loss scope toggle now affects only the **correct-guess +3 base bonus**.

The double-vote bonus no longer uses that toggle, because it already has separate numeric settings for:

- normal rounds
- too-many-wrong storyteller-loss rounds
- too-many-correct storyteller-loss rounds
