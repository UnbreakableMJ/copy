<!--
SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# CLAUDE.md — copy

This file is a strict superset of `AGENTS.md`. Keep the shared sections in sync
when project invariants, commands, packaging, or release workflow change.

## Project Identity

`copy` is a Linux-first Rust 2024 CLI that replaces `cp` with parallel copying,
resume support, reflinks, symlink/hardlink modes, and configurable preservation
behavior. The crate also ships a sibling `move` binary (a `mv` replacement) that
reuses the copy engine: it renames in place when possible and otherwise copies
then removes the source. It is a Spacecraft Software-maintained fork of upstream
`cpx`; upstream MIT attribution is preserved in `LICENSES/MIT.txt`, and fork
modifications are distributed under GPL-3.0-or-later.

## Build, Test, Lint

- Build: `cargo build`
- Release build: `cargo build --release`
- Test: `cargo test`
- Integration tests only: `cargo test --test intergration`
- One test by name: `cargo test test_copy_single_file`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format check: `cargo fmt --check`
- REUSE compliance: `nix-shell -p reuse --run "reuse lint"`
- Optional SELinux feature build: `cargo build --features selinux-support` after installing libselinux development headers that provide `selinux/selinux.h`
- Daily-drive install: always build and install the latest binary locally so the user can run `copy` from any directory — `cargo install --path . --locked` puts it on `PATH` at `~/.cargo/bin/copy`. Re-run after changes so the daily driver stays current.

CI runs build, test, clippy with warnings denied, and format check. Keep all four
green before handing work back.

## Architectural Invariants

- The binary in `src/main.rs` is intentionally thin: parse args, validate, set the signal abort flag, then dispatch to `copy()` or `multiple_copy()`.
- Copying is a two-phase pipeline: build a complete `CopyPlan` in `utility/preprocess.rs`, then execute it in `core/copy.rs`.
- `copy` is the root action. `config` is the only subcommand and is special-cased in `CLIArgs::parse()` because greedy source positionals would otherwise swallow `copy config show`.
- Config precedence is defaults → system → user → project → CLI. Invalid discovered config files are errors, not ignored fallbacks. `--config PATH` means defaults → that file → CLI.
- `CopyOptions::none()` and `CopyOptions::from_config()` must stay in sync when adding fields.
- `preprocess.rs` decides whether each item becomes a file, symlink, hardlink, resume skip, or directory task. Do not duplicate that routing in execution.
- Directory creation is sequential. Non-interactive file copies use a Rayon thread pool sized by `options.parallel`; `--interactive` is sequential.
- `copy_core()` tries hardlink preservation, reflink, Linux `copy_file_range`, then buffered fallback.
- Worker failures request cooperative cancellation for remaining parallel work. User SIGINT/SIGTERM uses the separate `options.abort` flag and maps to exit code 130 in `main.rs`.
- `README.md` and `README_CRATES.md` must stay in sync for user-facing behavior, install instructions, licensing, and release references.
- `move` (`src/bin/move.rs`, `src/core/move_op.rs`, `src/cli/move_args.rs`) is a thin second binary over the same library. It tries `std::fs::rename` first and only falls back to `copy()` + source removal on `EXDEV` (cross-device) or when `--exclude` must leave part of a directory behind. Because `copy()` always nests a directory under its destination, the directory fallback stages the copy in a temp dir on the destination filesystem and renames it into place. Never delete the source unless the copy succeeded.

## Forbidden Patterns

- Do not ignore filesystem errors in production copy paths. Backup creation, destination removal, xattr preservation, and destination creation failures must propagate.
- Do not print progress, logs, or diagnostics to stdout when stdout is data. Current code is not fully SFRS-compliant yet; do not make this worse.
- Do not add `unwrap()` or `expect()` in production paths for user input, filesystem state, config parsing, or CLI parsing.
- Do not make `selinux-support` a default feature; it requires system headers.
- Do not rename `tests/intergration.rs`; the misspelling is part of existing command/docs references.
- Do not remove upstream MIT attribution or `LICENSES/MIT.txt`.
- Do not touch unrelated dirty worktree files. In particular, inspect existing changes before editing `CLAUDE.md` or other root docs.

## Environment Expectations

- Rust stable with edition 2024 support; README documents Rust 1.85 or later.
- `cargo`, `rustc`, `clippy`, and `rustfmt` are expected.
- Nix may be available for ephemeral tooling such as `reuse`; prefer `nix-shell -p reuse --run "reuse lint"` over permanent installs.
- Default builds need no SELinux system library. All-features builds require libselinux development headers.
- GNU compatibility scripts in `tests/gnu/*.sh` are standalone and require a built `copy` binary on `PATH`; they are not part of `cargo test`.

## Where To Look

- CLI args and validation: `src/cli/args.rs`
- Config schema and merge behavior: `src/config/schema.rs`, `src/config/loader.rs`
- Copy planning: `src/utility/preprocess.rs`
- Copy execution and cancellation: `src/core/copy.rs`
- Linux fast path: `src/core/fast_copy.rs`
- Preservation logic: `src/utility/preserve.rs`
- Error types: `src/error.rs`
- Integration tests: `tests/intergration.rs`
- Standalone GNU compatibility scripts: `tests/gnu/`
- Move binary, engine, and CLI: `src/bin/move.rs`, `src/core/move_op.rs`, `src/cli/move_args.rs`
- Packaging: `nix/package.nix`, `guix.scm`, `packaging/aur/PKGBUILD`

## Release Notes For Agents

Version bumps are manual. Update `Cargo.toml`, `Cargo.lock`, `nix/package.nix`,
`guix.scm`, `packaging/aur/PKGBUILD`, `packaging/aur/.SRCINFO`, and release
references in both READMEs. The AUR source tarball hash exists only after the
GitHub tag is created, so use `SKIP` during the bump and fill the real hash in
a follow-up commit.

## Standards Compliance

This project is moving toward the Spacecraft Software CLI Standard but still has
legacy `cp`-compatible behavior. The `spacecraft-cli-standard` and
`spacecraft-agentic-cli` skills are authoritative for new CLI surface design;
this file records project-specific invariants and current exceptions only.

## Skills Referenced

- `spacecraft-standard` — Spacecraft Software project standard, licensing, posture, REUSE, time/unit policy.
- `microsoft-rust-guidelines` — required before writing or modifying Rust code.
- `spacecraft-cli-standard` — structural SFRS rules for future CLI surface work.
- `spacecraft-agentic-cli` — agent-facing UX and context-file guidance.
- `spacecraft-cli-preference` — preferred command-line tools, such as `rg`, `fd`, `bat`, `jaq`.
- `spacecraft-cli-shell` — shell syntax and portability checks for command snippets and scripts.
- `spacecraft-missing-pkg` — use when a needed tool is missing; prefer ephemeral Nix/Guix runners.

## MCP Servers Expected

No project-specific MCP server is implemented yet. Do not assume `copy mcp`,
`copy schema`, or `copy describe` exists unless implementing SFRS work explicitly.

## Tool Preferences For Claude

- Use `rg` for search and `fd` for file discovery.
- Use `cargo test` as the baseline; `cargo nextest` is not currently required by the repo.
- Use `nix-shell -p reuse --run "reuse lint"` for REUSE checks when `reuse` is not on `PATH`.
- Avoid broad auto-formatting outside Rustfmt-managed Rust files unless the task explicitly calls for it.

## Notes For Claude Specifically

- Treat `AGENTS.md` as the generic-agent source. If project facts change, update both files in the same patch.
- Claude Code may load skills, but this repo should not rely on Claude-only tooling for ordinary development.
- Do not add Claude-specific TODO markers unless the user asks for a Claude workflow.
