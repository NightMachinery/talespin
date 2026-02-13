# Svelte Reactivity Case Study: "Vote selection CSS bug" that was actually a reactivity bug

## TL;DR
The cards looked like a **CSS issue** (no border/glow when selected), but the core bug was **Svelte dependency tracking**:

- In template code, we used `voteCountForCard(image)`.
- That function read `selectedVotes`, but the template expression did **not** explicitly reference `selectedVotes`.
- Svelte did not re-run that expression when `selectedVotes` changed.
- Result: selection visuals stayed stale.

We fixed it by deriving a reactive map (`selectedVoteCounts`) and referencing it directly in markup.

---

## 1) What happened
During voting, clicking cards changed internal vote state, but card visuals (`ring`, badge `×1/×2`, glow) did not update reliably.

It looked like:
- click works,
- but selected border/glow does not appear,
- so it feels like CSS classes are broken.

---

## 2) The subtle Svelte rule behind this
Svelte tracks dependencies **statically** per expression.

If markup has this:

```svelte
{@const selectedCount = voteCountForCard(image)}
```

Svelte sees dependency on:
- `voteCountForCard` (function reference)
- `image`

It does **not** inspect function internals to discover `selectedVotes`.
So updates to `selectedVotes` may not invalidate this expression.

That is the key gotcha.

---

## 3) Why this looked like a CSS bug
The class logic depended on `selectedCount`:

- `selectedCount === 1` -> show selected styling
- `selectedCount >= 2` -> show stronger styling

If `selectedCount` does not recompute, class list never switches, so it **looks** like style rules failed.

---

## 4) The fix we used
### Before (problematic)

```ts
function voteCountForCard(card: string) {
  return selectedVotes.filter((value) => value === card).length;
}
```

```svelte
{@const selectedCount = voteCountForCard(image)}
```

### After (reactive + explicit dependency)

```ts
let selectedVoteCounts: Record<string, number> = {};

$: {
  const nextCounts: Record<string, number> = {};
  for (const card of selectedVotes) {
    nextCounts[card] = (nextCounts[card] ?? 0) + 1;
  }
  selectedVoteCounts = nextCounts;
}
```

```svelte
{@const selectedCount = selectedVoteCounts[image] ?? 0}
```

Now markup explicitly depends on `selectedVoteCounts`, which is updated from `selectedVotes`, so visuals update correctly.

---

## 5) Practical debugging heuristic for Svelte newbies
When UI seems stale:

1. Ask: **Is template expression directly referencing reactive state?**
2. If template calls a function, ask: **does that function hide dependencies?**
3. Prefer one of:
   - reactive derived values (`$:`), or
   - stores/derived stores,
   instead of hidden reads in helper functions used by template expressions.

---

## 6) How you could have pointed this out earlier (to avoid my mistake)
Great prompt pattern:

> "I suspect this is not CSS; please verify whether the card template depends on `selectedVotes` explicitly, and check for function-call dependency issues in Svelte reactivity."

Or even shorter:

> "Please debug this as a Svelte reactivity invalidation issue first, then CSS second."

Concrete evidence request that helps fast:

- "After click, does `selectedVotes.length` change?"
- "After click, does `selectedCount` in markup change for that card?"
- "If not, check whether selectedCount is computed via function call hiding dependencies."

That framing would have focused the investigation on the real cause much earlier.

---

## 7) Takeaway
In Svelte, **what the template references directly matters**.
If you hide state reads inside helper functions called from markup, updates can look "random" or "CSS broken" even when logic is correct.

Use explicit reactive derivations for values that drive rendering.
