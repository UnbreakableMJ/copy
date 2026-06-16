# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`copy` is a modern, parallel `cp` replacement for Linux, written in Rust (edition 2024). Explicit binary `[[bin]] name = "copy"` (`src/main.rs`) plus a library (`src/lib.rs`, crate name `copy`) that holds all the logic so it can be unit-tested. The binary is a thin shell: parse args → validate → dispatch to `core::copy`.

This is a fork: `origin` is `github.com/UnbreakableMJ/copy`, `upstream` is `github.com/11happy/cpx`. License is MIT (not the umbrella GPL/Standard posture — this crate predates and lives outside the Spacecraft Software Standard; do not retrofit Standard headers/posture files here unless asked).

## Commands

```bash
cargo build                       # debug build
cargo build --release             # optimized binary at target/release/copy
cargo test                        # all unit + integration tests
cargo test test_copy_single_file  # run one test by name
cargo test --test intergration    # integration suite only (note: file is mis-spelled "intergration.rs")
cargo clippy -- -D warnings       # CI gate: warnings are errors
cargo fmt -- --check              # CI gate: formatting (rustfmt.toml uses defaults)
cargo run -- -r src_dir/ dst_dir/ # run locally; args after `--` go to copy
```

CI (`.github/workflows/ci.yml`) runs exactly: build → test → `clippy -D warnings` → `fmt --check`. All four must pass.

The `selinux-support` feature is **off by default** and gates the `selinux` dependency:
```bash
cargo build --features selinux-support
```

### GNU compatibility tests

`tests/gnu/*.sh` are standalone shell scripts (reimplementations of coreutils `cp` tests) that invoke a `copy` binary on `PATH` — they are **not** wired into `cargo test`. To run one, build first and put the binary on PATH:
```bash
cargo build --release
PATH="$PWD/target/release:$PATH" sh tests/gnu/abuse.sh
```

## Architecture

Two-phase pipeline: **plan, then execute.** Nothing is copied until a complete `CopyPlan` is built; this is what makes parallelism, resume, and conflict-detection tractable.

```
main.rs            → installs SIGINT/SIGTERM handler that flips an AtomicBool ("abort"),
                     then calls copy() / multiple_copy()
cli/args.rs        → CLIArgs::parse (copy is the root action; `config` subcommand intercepted),
                     validate() → (sources, destination, CopyOptions)
config/            → layered config load, merged BEFORE cli overrides
utility/preprocess → walks the tree (jwalk), builds CopyPlan (files/dirs/symlinks/hardlinks)
core/copy.rs       → execute_copy(): rayon thread pool fans plan.files out to copy_core()
core/fast_copy.rs  → Linux copy_file_range() zero-copy fast path
utility/preserve   → applies mode/ownership/timestamps/xattr/ACL/SELinux after each copy
```

### Options precedence (in `args.rs::validate`)
`CopyOptions::none()` defaults → overlaid by `from_config()` if a config file applies → overlaid by `apply_cli_overrides(&mut options, &CLIArgs)` (CLI always wins) → `validate_conflicts()`. The two constructors (`none`, `from_config`) must stay in sync when you add a field to `CopyOptions` — there is no derive doing this. The unit tests in `args.rs` construct `CLIArgs` with every field listed explicitly, so adding a copy field touches those tests too.

### CLI structure (flattened: copy is root, `config` is the only subcommand)
Copy is the **root** command — `copy <src> <dst>` — so `CLIArgs` holds the copy args directly, plus `command: Option<Commands>` for the `config` subcommand (`copy config show`). Because `sources` is a greedy variadic that would swallow `config show` as two paths, `CLIArgs::parse()` special-cases a **leading** `config` argument and routes it through the dedicated `ConfigInvocation` parser (then `with_config()` builds a config-mode `CLIArgs`); everything else is parsed as a copy invocation. `validate()` runs the config subcommand and exits when `command` is `Some`, otherwise returns the copy inputs. The `command` field is still declared on `CLIArgs` so `--help` lists `config`. Consequence: a source literally named `config` as the first argument must be written `./config`.

### CopyPlan construction (`preprocess.rs`)
`preprocess_file` / `preprocess_directory` / `preprocess_multiple` produce a `CopyPlan`. Key invariants:
- **Last source wins:** `add_file_with_inode` calls `remove_existing_task` first, so a later source targeting the same destination replaces earlier file/symlink/hardlink tasks (prevents symlink write-through bugs — see `tests/gnu/abuse.sh`).
- **Resume:** when `options.resume`, `should_skip_file` compares xxh3 (`Xxh3`) checksums of source vs existing destination and marks the file skipped instead of queuing it.
- **Hard-link preservation** (`preserve.links`): files with `nlink > 1` are grouped by inode in `inode_groups`; at execution a `HardLinkTracker` (behind a `Mutex`) creates a hardlink for the 2nd+ member instead of re-copying.
- Mutually exclusive routing: a `FileTask` becomes a symlink, hardlink, resume-skip, or real copy — decided here, not at execution time.

### Execution & concurrency (`core/copy.rs::execute_copy`)
- Directories are created sequentially first, then files are copied in parallel via a `rayon::ThreadPoolBuilder` sized to `options.parallel` (`-j`).
- `--interactive` forces a **sequential** path (prompts can't be parallel).
- Per file, `copy_core` tries in order: hardlink-tracker shortcut → reflink (`reflink_copy`) → `fast_copy` (Linux `copy_file_range`) → buffered read/write fallback. Buffer size scales with file size (64 KiB → 2 MiB).
- **Cooperative cancellation:** every copy loop polls `options.abort` (the AtomicBool from `main.rs`); on abort it deletes the partial destination and returns `io::ErrorKind::Interrupted`. `main.rs` maps that to exit code **130** and prints the `--resume` hint. Other errors → exit **1**.
- Errors from parallel workers are collected, not fail-fast; up to 3 are printed.

### Errors (`error.rs`)
Domain-specific enums (`CopyError`, `ConfigError`, `ExcludeError`, `PreserveError`) each with a `*Result<T>` alias, funneled into the top-level `CliError`. Add new failure modes to the matching enum rather than stringly-typing.

## Conventions

- **Platform:** Linux-first. Linux-only code (`fast_copy`, `copy_file_range`) is behind `#[cfg(target_os = "linux")]`; unix-only metadata behind `#[cfg(unix)]`. Keep new platform-specific code gated so the crate still type-checks elsewhere.
- **Edition 2024** idioms are used heavily — `let ... &&` let-chains in `if`, `std::fs::exists`. Match the surrounding style.
- Two README files: `README.md` (GitHub) and `README_CRATES.md` (the `readme =` in `Cargo.toml`, shown on crates.io). Update both when user-facing behavior changes.
- User-facing docs live in `docs/` (`configuration.md`, `examples.md`, `benchmarks.md`); benchmark scripts/results in `benchmarks/`.
