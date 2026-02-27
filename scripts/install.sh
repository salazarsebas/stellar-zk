#!/usr/bin/env bash
# stellar-zk installer
# Usage: curl -fsSL https://raw.githubusercontent.com/salazarsebas/stellar-zk/main/scripts/install.sh | bash
#
# Environment variables:
#   STELLAR_ZK_VERSION  - specific version to install (e.g. "0.1.0"), default: latest
#   STELLAR_ZK_HOME     - installation directory, default: $HOME/.stellar-zk

set -euo pipefail

REPO="salazarsebas/stellar-zk"
INSTALL_DIR="${STELLAR_ZK_HOME:-$HOME/.stellar-zk}/bin"

# --- helpers ---

info() { printf '\033[1;34m%s\033[0m\n' "$*"; }
error() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

need_cmd() {
  if ! command -v "$1" > /dev/null 2>&1; then
    error "required command not found: $1"
  fi
}

# --- detect platform ---

detect_os() {
  case "$(uname -s)" in
    Linux*)  echo "unknown-linux-gnu" ;;
    Darwin*) echo "apple-darwin" ;;
    *)       error "unsupported OS: $(uname -s)" ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64)   echo "x86_64" ;;
    arm64|aarch64)   echo "aarch64" ;;
    *)               error "unsupported architecture: $(uname -m)" ;;
  esac
}

# --- resolve version ---

resolve_version() {
  if [ -n "${STELLAR_ZK_VERSION:-}" ]; then
    echo "v${STELLAR_ZK_VERSION#v}"
    return
  fi
  need_cmd curl
  local latest
  latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  if [ -z "$latest" ]; then
    error "could not determine latest version from GitHub"
  fi
  echo "$latest"
}

# --- verify checksum ---

verify_checksum() {
  local file="$1" expected="$2"
  local actual
  if command -v sha256sum > /dev/null 2>&1; then
    actual=$(sha256sum "$file" | awk '{print $1}')
  elif command -v shasum > /dev/null 2>&1; then
    actual=$(shasum -a 256 "$file" | awk '{print $1}')
  else
    info "warning: no sha256sum or shasum found, skipping checksum verification"
    return 0
  fi
  if [ "$actual" != "$expected" ]; then
    error "checksum mismatch for $file\n  expected: $expected\n  actual:   $actual"
  fi
  info "checksum verified"
}

# --- main ---

main() {
  need_cmd curl
  need_cmd tar

  local os arch target version archive url checksum_url
  os=$(detect_os)
  arch=$(detect_arch)
  target="${arch}-${os}"
  version=$(resolve_version)

  info "installing stellar-zk ${version} for ${target}"

  archive="stellar-zk-${version}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${version}/${archive}"
  checksum_url="https://github.com/${REPO}/releases/download/${version}/SHA256SUMS.txt"

  local tmpdir
  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  info "downloading ${archive}..."
  curl -fsSL "$url" -o "${tmpdir}/${archive}" || error "download failed — check that version ${version} exists and has a release for ${target}"

  info "downloading checksums..."
  curl -fsSL "$checksum_url" -o "${tmpdir}/SHA256SUMS.txt" || error "could not download checksums"

  local expected_hash
  expected_hash=$(grep "${archive}" "${tmpdir}/SHA256SUMS.txt" | awk '{print $1}')
  if [ -z "$expected_hash" ]; then
    error "no checksum found for ${archive} in SHA256SUMS.txt"
  fi
  verify_checksum "${tmpdir}/${archive}" "$expected_hash"

  info "extracting..."
  tar xzf "${tmpdir}/${archive}" -C "${tmpdir}"

  mkdir -p "$INSTALL_DIR"
  cp "${tmpdir}/stellar-zk-${version}-${target}/stellar-zk" "${INSTALL_DIR}/stellar-zk"
  chmod +x "${INSTALL_DIR}/stellar-zk"

  info "installed stellar-zk to ${INSTALL_DIR}/stellar-zk"

  # check if already in PATH
  if command -v stellar-zk > /dev/null 2>&1; then
    info "stellar-zk is already in your PATH"
    stellar-zk --version 2>/dev/null || true
    return
  fi

  # suggest PATH addition
  local shell_name rc_file
  shell_name=$(basename "${SHELL:-bash}")
  case "$shell_name" in
    zsh)  rc_file="$HOME/.zshrc" ;;
    bash)
      if [ -f "$HOME/.bash_profile" ]; then
        rc_file="$HOME/.bash_profile"
      else
        rc_file="$HOME/.bashrc"
      fi
      ;;
    fish) rc_file="$HOME/.config/fish/config.fish" ;;
    *)    rc_file="$HOME/.profile" ;;
  esac

  echo ""
  info "add stellar-zk to your PATH by adding this to ${rc_file}:"
  echo ""
  if [ "$shell_name" = "fish" ]; then
    echo "  fish_add_path ${INSTALL_DIR}"
  else
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
  fi
  echo ""
  info "then restart your shell or run:"
  echo ""
  if [ "$shell_name" = "fish" ]; then
    echo "  fish_add_path ${INSTALL_DIR}"
  else
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
  fi
}

main
