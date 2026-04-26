- Keep `./docs` updated as we implement and change things.

## Worktrees

- On this VPS, disk is tight; do not duplicate large dependency/build directories inside worktrees.
- Reuse the main checkout's frontend dependencies from worktrees by symlinking:
  - `.worktrees/<name>/node_modules -> ../../node_modules`
- Reuse the main checkout's Rust build output from worktrees by symlinking:
  - `.worktrees/<name>/talespin-server/target -> ../../../talespin-server/target`
- Do not run `npm install` in a worktree unless dependencies actually changed and disk space has been checked first.
- Do not allow Cargo builds in a worktree to create a separate `talespin-server/target`; set up the shared symlink before running Cargo commands.
- Before Node/npm build or check commands on the VPS, follow `~/.codex/VPS.md` preflight guidance and avoid concurrent builds.
