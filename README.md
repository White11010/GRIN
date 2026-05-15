# GRIN

**GRIN** is a small, fast command-line tool for Git repository analytics. It reads your local repository via `git` and prints **timeline**, **contributor**, and **file churn** summaries in the terminal—no servers, no extra services.

## Features

- **`grin timeline`** — chronological activity view derived from `git log`
- **`grin who`** — top contributors by commit count
- **`grin churn`** — files touched most often (optional extension filter)

Shared flags (after the command):

- **`--limit N`** — max rows (default: `5`)
- **`--ext LIST`** — `churn` only: comma-separated extensions (e.g. `ts,tsx`, leading dots optional)

## Requirements

- **Git** installed and available on your `PATH` (`git` is invoked as a subprocess).
- Run GRIN from **inside a Git working tree** (or a subdirectory of one).

## Installation

### From crates.io (Rust toolchain required)

```bash
cargo install grin
```

Ensure Cargo’s bin directory is on your `PATH` (often `~/.cargo/bin` on Unix and `%USERPROFILE%\.cargo\bin` on Windows).

### Install script (Linux and macOS)

Installs the latest **GitHub Release** binary into a directory of your choice (default: `~/.local/bin`).

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.sh | bash
```

Install a specific version:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.sh | bash -s -- --version v0.1.0
```

Custom install directory:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.sh | bash -s -- --bin-dir /usr/local/bin
```

After installation, confirm `grin` is on your `PATH`:

```bash
grin help
```

### Prebuilt binaries (Windows and manual installs)

See [**Releases**](https://github.com/White11010/GRIN/releases): each tag publishes archives for:

| Platform        | Archive pattern |
|----------------|-----------------|
| Linux x86_64   | `grin-<tag>-x86_64-unknown-linux-gnu.tar.gz` |
| macOS x86_64   | `grin-<tag>-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `grin-<tag>-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `grin-<tag>-x86_64-pc-windows-msvc.zip` |

Extract the `grin` (or `grin.exe`) binary and place it in a directory on your `PATH`.

### Build from source

Requires **Rust 1.85+** (Rust 2024 edition).

```bash
git clone https://github.com/White11010/GRIN.git
cd GRIN
cargo install --path . --locked
```

Or build without installing:

```bash
cargo build --release --locked
# Binary: target/release/grin (or target/<triple>/release/grin with --target)
```

## Usage

From the repository root (or any subfolder):

```bash
grin who
grin timeline
grin churn --limit 10
grin churn --ext rs,toml
grin help
```

If you run `grin` with no arguments, help is printed.

## Development

```bash
cargo fmt --all
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).
