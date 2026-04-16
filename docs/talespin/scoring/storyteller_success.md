# Storyteller success score

Talespin now lets moderators set a dedicated **storyteller success score** for Dixit rounds.

## What it changes

- It applies only when the storyteller has a **successful normal round**: not everyone guessed correctly, and not everyone missed.
- It changes only the **storyteller's own reward** for that branch.
- **Correct guessers still keep their normal +3 base score.**
- Decoy bonuses and storyteller-loss branches keep their existing rules.

Default and bounds:

- default: `3`
- allowed range: `0..10`

## Where moderators can edit it

- in the **Joining** lobby settings before the game starts
- during the live Dixit game from the moderator sidebar

Like the other storyteller scoring controls, changing it after a round has already been scored only affects later successful rounds.

## Relation to `W`

`W` still controls **when the storyteller loses** by setting the storyteller-loss threshold.

The storyteller success score is separate:

- `W` answers **"was this storyteller round a loss?"**
- storyteller success score answers **"if it was a success, how many points does the storyteller get?"**

So you can tune the reward for successful clues without changing the loss threshold itself.
