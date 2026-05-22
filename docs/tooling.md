# Tooling Notes

## Vitest test discovery

Vitest excludes the repository-local `.worktrees/` directory in `vite.config.ts`.
This keeps tests from discovering copied checkouts under worktrees when `npm test` is run from the main checkout.
Keep this exclusion alongside Vitest's default excludes via `configDefaults.exclude` so normal generated and dependency directories remain ignored.

## Vite icon imports

Avoid package barrel imports for large icon libraries in Svelte components. Even when the final bundle
tree-shakes unused icons, Vite/Rollup can still spend noticeable build time parsing the package root.

Prefer direct per-icon imports for Lucide:

```ts
import Copy from 'lucide-svelte/icons/copy';
import Power from '@lucide/svelte/icons/power';
```

Prefer direct per-icon imports for Feather:

```ts
import MaximizeIcon from 'svelte-feather-icons/src/icons/MaximizeIcon.svelte';
import MinimizeIcon from 'svelte-feather-icons/src/icons/MinimizeIcon.svelte';
```
