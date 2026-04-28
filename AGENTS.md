- Keep `./docs` updated as we implement and change things.
- Do not offer or use the brainstorming visual companion unless the user explicitly requests it.

## Worktrees

- Do NOT use worktrees or a new branch unless the user explicitly asks you to.
- On this VPS, disk is tight; do not duplicate large dependency/build directories inside worktrees.
- Reuse the main checkout's frontend dependencies from worktrees by symlinking:
  - `.worktrees/<name>/node_modules -> ../../node_modules`
- Reuse the main checkout's Rust build output from worktrees by symlinking:
  - `.worktrees/<name>/talespin-server/target -> ../../../talespin-server/target`
- Do not run `npm install` in a worktree unless dependencies actually changed and disk space has been checked first.
- Do not allow Cargo builds in a worktree to create a separate `talespin-server/target`; set up the shared symlink before running Cargo commands.
- When a worktree needs its own runnable backend binary while sharing Cargo `target`, avoid `cargo run` and build a branch-suffixed binary instead:
  - `cd talespin-server && cargo rustc --bin talespin-server -- -C extra-filename=-<worktree-or-branch-name>`
  - Run the resulting executable from `talespin-server/target/debug/deps/talespin-server-<worktree-or-branch-name>`.
  - For release builds, add `--release` and run from `talespin-server/target/release/deps/`.
