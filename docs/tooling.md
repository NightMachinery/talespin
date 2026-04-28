# Tooling Notes

## Vitest test discovery

Vitest excludes the repository-local `.worktrees/` directory in `vite.config.ts`.
This keeps tests from discovering copied checkouts under worktrees when `npm test` is run from the main checkout.
Keep this exclusion alongside Vitest's default excludes via `configDefaults.exclude` so normal generated and dependency directories remain ignored.
