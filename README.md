# GRIN

**GRIN** is a small command-line tool for Git repository analytics. Run it inside any Git repo to see a **timeline**, **top contributors**, and **file churn** in the terminal — no servers, no database.

## Contents

- [Quick start](#quick-start)
- [What you get](#what-you-get)
- [Commands and flags](#commands-and-flags)
- [Terminal notes](#terminal-notes)
  - [Colors](#colors)
  - [Windows and symbol rendering](#windows-and-symbol-rendering)
- [Installation](#installation)
  - [crates.io (Rust toolchain)](#cratesio-rust-toolchain)
  - [Install script](#install-script)
  - [Prebuilt binaries](#prebuilt-binaries)
  - [Build from source](#build-from-source)
- [Development](#development)
- [License](#license)

![image](/docs/images/demo.png)

## Quick start

Requires [Git](https://git-scm.com/) on your `PATH`. From a repository root (or any subfolder):

```bash
grin timeline
grin who
grin churn --limit 10
```

Run `grin` or `grin help` for usage.

## What you get

- `**grin timeline**` — first commit, contributor joins, activity peaks, quiet periods, yearly sparklines
- `**grin who**` — contributors by commit count, with feat / fix / chore / other breakdown
- `**grin churn**` — files touched most often; filter with `--ext ts,tsx`

## Commands and flags

| Command    | Description                                    |
| ---------- | ---------------------------------------------- |
| `timeline` | Chronological activity from `git log`          |
| `who`      | Top contributors by commit count               |
| `churn`    | Files with the most changes                    |
| `help`     | Show usage (`grin` with no args does the same) |

Flags work **before or after** the command:

| Flag         | Applies to   | Description                                  |
| ------------ | ------------ | -------------------------------------------- |
| `--limit N`  | all          | Max rows (default `5`)                       |
| `--ext LIST` | `churn` only | Comma-separated extensions, e.g. `ts,tsx`    |
| `--no-color` | all          | Plain text, no ANSI colors (see below)       |
| `--ascii`    | all          | ASCII symbols instead of Unicode (see below) |

Examples:

```bash
grin churn --ext rs,toml --limit 20
grin --no-color timeline
grin who --ascii
```

Environment variables (same effect as flags):

- `NO_COLOR` — disable colors ([no-color.org](https://no-color.org))
- `GRIN_ASCII` — ASCII symbol set

## Terminal notes

### Colors

Colored output is tuned for a **dark terminal background**. On a light background or if colors look wrong, use plain mode:

```bash
grin timeline --no-color
# or
NO_COLOR=1 grin who
```

### Windows and symbol rendering

Use **Windows Terminal** (not legacy conhost) with a monospace font such as **Cascadia Mono** or **JetBrains Mono**. Set UTF-8 if needed: `chcp 65001` in cmd, or enable **Beta: Use Unicode UTF-8 for worldwide language support** in Windows region settings.

If sparklines or lines show empty boxes (“tofu”), use ASCII mode:

```bash
grin timeline --ascii
# or
GRIN_ASCII=1 grin timeline
```

## Installation

### Install script

**Linux / macOS** — latest release into `~/.local/bin`, adds it to your shell startup files when needed (override with `--bin-dir`):

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.sh | bash \
  && export PATH="$HOME/.local/bin:$PATH"
```

The `export` line makes `grin` work in the **current** terminal right away; new terminals pick up `PATH` from `~/.zshrc` (macOS) or `~/.profile` / `~/.bashrc` (Linux) automatically.

**Windows** — latest release into `%USERPROFILE%\.local\bin` (adds to user `PATH` when needed):

```powershell
irm https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.ps1 | iex
```

Specific version: `bash -s -- --version v0.1.0` (Unix) or `$env:GRIN_INSTALL_VERSION = 'v0.1.0'` before `iex` (Windows).

To inspect the script first on Windows:

```powershell
irm https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.ps1 -OutFile install.ps1
.\install.ps1
```

If you used a custom install directory, run the `export PATH=…` line printed at the end of `install.sh` (with your path), or open a **new** terminal on Windows after install. Then run `grin help`.

### Prebuilt binaries

Download archives from **[Releases](https://github.com/White11010/GRIN/releases)**:

| Platform            | Archive pattern                              |
| ------------------- | -------------------------------------------- |
| Linux x86_64        | `grin-<tag>-x86_64-unknown-linux-gnu.tar.gz` |
| macOS x86_64        | `grin-<tag>-x86_64-apple-darwin.tar.gz`      |
| macOS Apple Silicon | `grin-<tag>-aarch64-apple-darwin.tar.gz`     |
| Windows             | Release zip or the install script above      |

Extract the `grin` binary into a directory on your `PATH`.

### Build from source

Requires **Rust 1.85+** (edition 2024).

```bash
git clone https://github.com/White11010/GRIN.git
cd GRIN
cargo install --path . --locked
```

Or build only:

```bash
cargo build --release --locked
# target/release/grin
```

## Development

```bash
cargo fmt --all
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

## License

MIT — see [LICENSE](LICENSE).
