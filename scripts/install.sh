#!/usr/bin/env bash
# Install GRIN from GitHub Releases (Linux x86_64, macOS x86_64 / arm64).
# Usage:
#   curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.sh | bash
#   curl ... | bash -s -- --version v0.1.0 --bin-dir ~/.local/bin

set -euo pipefail

DEFAULT_REPO="White11010/GRIN"
REPO="${GRIN_INSTALL_REPO:-$DEFAULT_REPO}"
VERSION=""
BIN_DIR="${GRIN_INSTALL_DIR:-$HOME/.local/bin}"
UPDATE_PATH=1

usage() {
  cat <<'EOF'
Install GRIN from GitHub Releases.

Usage:
  install.sh [--version <tag>] [--bin-dir <path>] [--repo owner/name]

Options:
  --version, -v   Release tag (e.g. v0.1.0). Default: latest GitHub release.
  --bin-dir, -b   Directory to install the binary into. Default: $HOME/.local/bin
  --repo          GitHub repository as owner/name. Default: White11010/GRIN
  --no-path       Do not update shell startup files (PATH)
  -h, --help      Show this help.

Environment:
  GRIN_INSTALL_REPO   Override default repository (owner/name).
  GRIN_INSTALL_DIR    Default install directory if --bin-dir is not passed.

Examples:
  curl -fsSL .../install.sh | bash
  curl -fsSL .../install.sh | bash -s -- --version v0.1.0
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version|-v)
      VERSION="${2:-}"
      if [[ -z "$VERSION" ]]; then
        echo "install.sh: missing value for $1" >&2
        exit 1
      fi
      shift 2
      ;;
    --bin-dir|-b)
      BIN_DIR="${2:-}"
      if [[ -z "$BIN_DIR" ]]; then
        echo "install.sh: missing value for $1" >&2
        exit 1
      fi
      shift 2
      ;;
    --repo)
      REPO="${2:-}"
      if [[ -z "$REPO" ]]; then
        echo "install.sh: missing value for $1" >&2
        exit 1
      fi
      shift 2
      ;;
    --no-path)
      UPDATE_PATH=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "install.sh: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

command -v curl >/dev/null 2>&1 || {
  echo "install.sh: curl is required but not found in PATH." >&2
  exit 1
}

CURL_GITHUB=(curl --proto '=https' --tlsv1.2 -fsSL -A 'GRIN-install-script')

get_latest_release_tag() {
  local repo="$1"
  local effective

  effective="$("${CURL_GITHUB[@]}" -o /dev/null -w '%{url_effective}' \
    "https://github.com/${repo}/releases/latest")" || {
    echo "install.sh: failed to resolve latest release." >&2
    echo "install.sh: set a tag manually, e.g. curl ... | bash -s -- --version v0.1.2" >&2
    exit 1
  }

  if [[ "$effective" =~ /releases/tag/(v[^/?#]+) ]]; then
    echo "${BASH_REMATCH[1]}"
    return
  fi

  echo "install.sh: could not parse latest release tag from GitHub redirect." >&2
  exit 1
}

GRIN_PATH_MARKER="# Added by GRIN install script"

resolve_bin_dir_full() {
  mkdir -p "$1"
  (cd "$1" && pwd -P)
}

path_configured_in_file() {
  local file="$1"
  local bin_dir_full="$2"
  [[ -f "$file" ]] || return 1
  grep -qF "$GRIN_PATH_MARKER" "$file" 2>/dev/null && grep -qF "$bin_dir_full" "$file" 2>/dev/null
}

append_path_to_file() {
  local file="$1"
  local bin_dir_full="$2"

  if path_configured_in_file "$file" "$bin_dir_full"; then
    return 0
  fi

  {
    echo ""
    echo "$GRIN_PATH_MARKER"
    echo "export PATH=\"${bin_dir_full}:\$PATH\""
  } >>"$file"
  echo "install.sh: updated ${file}" >&2
}

shell_startup_candidates() {
  local uname_s="$1"
  local -a files=()

  case "$uname_s" in
    Darwin)
      files+=("$HOME/.zshrc")
      if [[ -f "$HOME/.bash_profile" ]]; then
        files+=("$HOME/.bash_profile")
      fi
      ;;
    Linux)
      if [[ -f "$HOME/.bashrc" ]]; then
        files+=("$HOME/.bashrc")
      elif [[ -f "$HOME/.bash_profile" ]]; then
        files+=("$HOME/.bash_profile")
      fi
      if [[ -f "$HOME/.profile" ]]; then
        files+=("$HOME/.profile")
      fi
      if [[ -f "$HOME/.zshrc" ]]; then
        files+=("$HOME/.zshrc")
      fi
      ;;
  esac

  printf '%s\n' "${files[@]}"
}

ensure_shell_path() {
  local bin_dir_full="$1"
  local uname_s="$2"
  local login_shell=""
  local -a candidates=()
  local file
  local updated=0

  login_shell="$(basename "${SHELL:-}")"

  while IFS= read -r file; do
    [[ -n "$file" ]] && candidates+=("$file")
  done < <(shell_startup_candidates "$uname_s")

  if [[ ${#candidates[@]} -eq 0 ]]; then
    case "$uname_s" in
      Darwin)
        if [[ "$login_shell" == "bash" ]]; then
          candidates=("$HOME/.bash_profile")
        else
          candidates=("$HOME/.zshrc")
        fi
        ;;
      Linux)
        candidates=("$HOME/.profile")
        ;;
    esac
  fi

  for file in "${candidates[@]}"; do
    if [[ ! -f "$file" ]] && [[ "$file" == "$HOME/.zshrc" || "$file" == "$HOME/.profile" ]]; then
      : >"$file"
    fi
    if [[ -f "$file" ]]; then
      if ! path_configured_in_file "$file" "$bin_dir_full"; then
        append_path_to_file "$file" "$bin_dir_full"
        updated=1
      fi
    fi
  done

  if [[ "$updated" -eq 0 ]]; then
    local already=0
    for file in "${candidates[@]}"; do
      if path_configured_in_file "$file" "$bin_dir_full"; then
        already=1
        break
      fi
    done
    if [[ "$already" -eq 1 ]]; then
      echo "install.sh: ${bin_dir_full} is already on PATH in your shell startup files." >&2
    fi
  fi
}

uname_s="$(uname -s)"
uname_m="$(uname -m)"
target=""

case "$uname_s" in
  Linux)
    case "$uname_m" in
      x86_64) target="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64)
        echo "install.sh: no prebuilt Linux ARM64 archive in this release channel yet." >&2
        echo "install.sh: install with: cargo install grin" >&2
        exit 1
        ;;
      *)
        echo "install.sh: unsupported Linux machine: $uname_m" >&2
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$uname_m" in
      x86_64) target="x86_64-apple-darwin" ;;
      arm64) target="aarch64-apple-darwin" ;;
      *)
        echo "install.sh: unsupported macOS machine: $uname_m" >&2
        exit 1
        ;;
    esac
    ;;
  *)
    echo "install.sh: unsupported operating system: $uname_s" >&2
    echo "install.sh: on Windows, run: irm https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.ps1 | iex" >&2
    echo "install.sh: or use: cargo install grin" >&2
    exit 1
    ;;
esac

if [[ -z "$VERSION" ]]; then
  VERSION="$(get_latest_release_tag "$REPO")"
fi

case "$VERSION" in
  v*) ;;
  *)
    echo "install.sh: expected tag like v0.1.0, got: $VERSION" >&2
    exit 1
    ;;
esac

asset="grin-${VERSION}-${target}.tar.gz"
url="https://github.com/${REPO}/releases/download/${VERSION}/${asset}"

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

echo "install.sh: installing GRIN ${VERSION} (${target}) from ${url}" >&2
"${CURL_GITHUB[@]}" -o "${tmp}/${asset}" "$url" || {
  echo "install.sh: download failed. Check the tag and your network, or install from source." >&2
  exit 1
}

tar -xzf "${tmp}/${asset}" -C "$tmp"
binary="${tmp}/grin"
if [[ ! -f "$binary" ]]; then
  echo "install.sh: expected binary 'grin' inside archive." >&2
  exit 1
fi

mkdir -p "$BIN_DIR"
if [[ ! -d "$BIN_DIR" ]]; then
  echo "install.sh: install directory is not a directory: $BIN_DIR" >&2
  exit 1
fi

if [[ ! -w "$BIN_DIR" ]]; then
  echo "install.sh: no write permission for: $BIN_DIR" >&2
  echo "install.sh: pick a writable directory or re-run with sudo (not recommended for \$HOME paths)." >&2
  exit 1
fi

install -m 0755 "$binary" "${BIN_DIR}/grin"

BIN_DIR_FULL="$(resolve_bin_dir_full "$BIN_DIR")"

if [[ "$UPDATE_PATH" -eq 1 ]]; then
  ensure_shell_path "$BIN_DIR_FULL" "$uname_s"
fi

echo "install.sh: installed to ${BIN_DIR_FULL}/grin" >&2
echo "" >&2
echo "install.sh: Run this in the current terminal to use grin immediately:" >&2
echo "  export PATH=\"${BIN_DIR_FULL}:\$PATH\"" >&2
if [[ "$UPDATE_PATH" -eq 1 ]]; then
  echo "install.sh: New terminals load PATH from your shell startup files automatically." >&2
else
  echo "install.sh: ensure ${BIN_DIR_FULL} is on your PATH." >&2
fi
echo "install.sh: Then run: grin help" >&2
