# Storyteller win condition (W)

`W` is the moderator-facing storyteller win-condition number shown in the room settings and score cheatsheet.

## Definition

- The storyteller wins when **at least `W` guessers are different from the rest**.
- This covers both main storyteller-win shapes:
  - enough guessers were **correct**
  - enough guessers were **wrong**

In the UI this is phrased as:

- “Storyteller wins when at least `W` people are different from others”
- “Storyteller wins when at least `W` guessers are different from the rest”

## Relation to server scoring

The server stores and computes with `storyteller_loss_complement`, not `W` directly.

The relationship is:

`W = storyteller_loss_complement + 1`

So:

- complement `0` => `W = 1`
- complement `1` => `W = 2`
- complement `2` => `W = 3`

## Why `W` exists

The raw server threshold is based on:

`threshold = guesser_count - storyteller_loss_complement`

That threshold answers:

- “How many correct guessers trigger storyteller loss?”
- “How many wrong guessers trigger storyteller loss?”

But the moderator-facing `W` is easier to reason about because it directly answers:

- “How many people need to break away from the rest for the storyteller to succeed?”

## Auto-tune

When storyteller auto-tune is enabled, the server still sends the effective
`storyteller_loss_complement`, and the UI derives:

`W = storyteller_loss_complement + 1`

So the displayed `W` always reflects the current live server setting.

## Current UI usage

`W` is the number shown in:

- the moderator panel storyteller win-condition control
- the score cheatsheet summary text
