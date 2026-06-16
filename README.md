# copy

<div align="center">

**A modern, fast file copy tool for Linux with progress bars, resume capability, and more.**

[![Crates.io](https://img.shields.io/crates/v/copy.svg)](https://crates.io/crates/copy)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/UnbreakableMJ/copy/actions/workflows/ci.yml/badge.svg)](https://github.com/UnbreakableMJ/copy/actions/workflows/ci.yml)


[Features](#features) •
[Installation](#installation) •
[Quick Start](#quick-start) •
[Documentation](#documentation)

</div>

---

## Why copy?

`copy` is a modern replacement for the traditional `cp` command, built with Rust for maximum performance and safety on Linux systems.

![one](https://github.com/user-attachments/assets/85fdbe39-2635-41b0-a00a-27ba7d2e8e60)

## Features
- 🚀 Fast parallel copying (upto 5x faster than cp [benchmarks](docs/benchmarks.md))
- 📊 Beautiful progress bars (customizable)
- ⏸️ Resume interrupted transfers
- 🛑 Graceful Ctrl+C handling with resume hints
  ![four](https://github.com/user-attachments/assets/11c9ecb8-ea57-4162-9772-bdf071f61848)
- 🎯 Exclude patterns (gitignore-style)
- ⚙️ Flexible configuration





## Installation

### Quick Install
```bash
curl -fsSL https://raw.githubusercontent.com/UnbreakableMJ/copy/main/install.sh | bash
```

Or with wget:
```bash
wget -qO- https://raw.githubusercontent.com/UnbreakableMJ/copy/main/install.sh | bash
```

### From Crates.io
```bash
cargo install copy
```

### Arch Linux (AUR)
A first-party PKGBUILD lives in [`packaging/aur/`](packaging/aur/). Build and install it from a checkout:
```bash
cd packaging/aur && makepkg -si
```
> An official AUR upload will track this PKGBUILD. (The older community `cpx-copy` package predates the rename.)


### Nix / NixOS
This repo is a flake. Run or install `copy` straight from GitHub:
```bash
nix run github:UnbreakableMJ/copy -- --help
nix profile install github:UnbreakableMJ/copy
```
For a dev shell with the Rust toolchain, run `nix develop`.


### GNU Guix
A [`guix.scm`](guix.scm) is provided. Guix builds offline, so vendor the crates once, then build:
```bash
cargo vendor guix/vendor
guix build -f guix.scm
```
(Needs a Guix `rust` ≥ 1.85 for edition 2024.)


### From Source
```bash
cargo install --git https://github.com/UnbreakableMJ/copy
copy --version
```

### Pre-built Binaries

Download from [Releases](https://github.com/UnbreakableMJ/copy/releases)

## Quick Start

### Basic Usage
```bash
# Copy a file
copy source.txt dest.txt

# Copy directory recursively
copy -r source_dir/ dest_dir/

# exclude build artifacts
copy -r -e "node_modules" -e ".git" -e "target" my-project/ /backup/

# Resume interrupted transfer
copy -r --resume large_dataset/ /backup/

# Copy with full attribute preservation
copy -r -p=all photos/ /backup/photos/
```

**See [examples.md](docs/examples.md) for detailed workflows and real-world scenarios.**

## Key Options
```
copy [OPTIONS] <SOURCE>... <DESTINATION>

Arguments:
  <SOURCE>...       Source file(s) or directory(ies)
  <DESTINATION>     Destination file or directory

Input/Output Options:
  -t, --target-directory <DIRECTORY>
                           Copy all SOURCE arguments into DIRECTORY
  -e, --exclude <PATTERN>  Exclude files matching pattern (supports globs, comma-separated)

Copy Behavior:
  -r, --recursive          Copy directories recursively
  -j <N>                   Number of parallel operations [default: 4]
      --resume             Resume interrupted transfers (checksum verified)
  -f, --force              Remove and retry if destination cannot be opened
  -i, --interactive        Prompt before overwrite
      --parents            Use full source file name under DIRECTORY
      --attributes-only    Copy only attributes, not file data
      --remove-destination Remove destination file before copying

Link and Symlink Options:
  -s, --symbolic-link [MODE]
                           Create symlinks instead of copying [auto|absolute|relative]
  -l, --link               Create hard links instead of copying
  -P, --no-dereference     Never follow symbolic links in SOURCE
  -L, --dereference        Always follow symbolic links in SOURCE
  -H, --dereference-command-line
                           Follow symbolic links only on command line

Preservation:
  -p, --preserve [ATTRS]   Preserve attributes [default|all|mode,timestamps,ownership,...]
                           Available: mode, ownership, timestamps, links, context, xattr

Backup and Reflink:
  -b, --backup [MODE]      Backup existing files [none|simple|numbered|existing]
      --reflink [WHEN]     CoW copy if supported [auto|always|never]

Configuration:
      --config <PATH>      Use custom config file
      --no-config          Ignore all config files

Other:
  -h, --help               Print help information
  -V, --version            Print version information
```


For complete usage examples, see [examples.md](docs/examples.md)

For complete option reference, run `copy --help`

## Configuration

Set defaults with configuration files:
```bash
# Create config with defaults
copy config init

# View active configuration
copy config show

# See config file location
copy config path
```

**Config locations (in priority order):**
1. `./copyconfig.toml` (project-level)
2. `~/.config/copy/copyconfig.toml` (user-level)
3. `/etc/copy/copyconfig.toml` (system-level, Unix only)

**Example config** (`~/.config/copy/copyconfig.toml`):
```toml
[exclude]
patterns = ["*.tmp", "*.log", "node_modules", ".git"]

[copy]
parallel = 8
recursive = false

[preserve]
mode = "default"

[progress]
style = "detailed"

[reflink]
mode = "auto"
```

**See [configuration.md](docs/configuration.md) for all options and use cases.**

## Performance

`copy` is built for speed. Quick comparison:

| Task | cp | copy -j16 | speedup |
|------|-----|-------|-----|
| VsCode (~15k files) | 1084ms | 263ms | 4.12x |
| rust (~65k files) | 4.553s | 1.091s  |  4.17x |

**See [benchmarks.md](docs/benchmarks.md) for detailed methodology and more comparisons.**

## Documentation

- **[Configuration Guide](docs/configuration.md)** - Complete config reference
- **[Benchmarks](docs/benchmarks.md)** - Performance analysis and comparisons
- **[Contributing](CONTRIBUTING.md)** - How to contribute

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| **Linux** | ✅ Supported | Fast copy supported for (kernel 4.5+) |
| macOS | 🔄 Planned | To be released |
| Windows | 🔄 Planned | To be released |

## Quick Start for Developers
```bash
git clone https://github.com/UnbreakableMJ/copy.git
cd copy

# Run tests
cargo test

# Run clippy
cargo clippy

# Try it out
cargo run -- -r test_data/ test_dest/
```

## Tests

Some tests are already ported from the [GNU coreutils cp test suite](https://github.com/coreutils/coreutils/tree/master/tests/cp), still porting more [GNU ported tests](https://github.com/UnbreakableMJ/copy/tree/main/tests/gnu).

Found wrong behavior? [File an issue](https://github.com/UnbreakableMJ/copy/issues), PRs for more tests are always welcome!

## License

- MIT [LICENSE](https://github.com/UnbreakableMJ/copy/blob/main/LICENSE)


## Acknowledgments

Inspired by `ripgrep`, `fd`, and the modern Rust CLI ecosystem.

Built with: [clap](https://github.com/clap-rs/clap), [indicatif](https://github.com/console-rs/indicatif), [rayon](https://github.com/rayon-rs/rayon), [jwalk](https://github.com/Byron/jwalk), and more.

---
