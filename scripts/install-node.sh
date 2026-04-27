#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Install and configure an SBFB nexus-grid node.
#
# Detects OS (Linux/macOS), installs system dependencies, builds
# the requested components (daemon, worker, coordinator), and
# optionally installs systemd units.
#
# Usage:
#   ./scripts/install-node.sh                # interactive mode
#   ./scripts/install-node.sh --yes          # non-interactive (accept all)
#   ./scripts/install-node.sh --daemon-only  # build daemon only
#   ./scripts/install-node.sh --help

set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────

INSTALL_DIR="/opt/nexus-grid"
AUTO_YES=false
INSTALL_DAEMON=true
INSTALL_WORKER=true
INSTALL_COORDINATOR=true
REPO_URL="https://github.com/user/nexus-grid.git"

# ── Helpers ───────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { printf "${GREEN}[INFO]${NC}  %s\n" "$*"; }
warn()  { printf "${YELLOW}[WARN]${NC}  %s\n" "$*"; }
error() { printf "${RED}[ERROR]${NC} %s\n" "$*" >&2; }

confirm() {
    if "$AUTO_YES"; then
        return 0
    fi
    local prompt="$1"
    local answer
    printf "%s [y/N] " "$prompt"
    read -r answer
    case "$answer" in
        [yY]|[yY][eE][sS]) return 0 ;;
        *) return 1 ;;
    esac
}

command_exists() { command -v "$1" >/dev/null 2>&1; }

# ── Argument parsing ─────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case "$1" in
        --yes|-y)       AUTO_YES=true; shift ;;
        --daemon-only)  INSTALL_WORKER=false; INSTALL_COORDINATOR=false; shift ;;
        --no-worker)    INSTALL_WORKER=false; shift ;;
        --no-coordinator) INSTALL_COORDINATOR=false; shift ;;
        --dir)          INSTALL_DIR="$2"; shift 2 ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --yes, -y          Non-interactive mode (accept all prompts)"
            echo "  --daemon-only      Build daemon only (skip worker + coordinator)"
            echo "  --no-worker        Skip worker build"
            echo "  --no-coordinator   Skip coordinator build"
            echo "  --dir PATH         Install directory (default: /opt/nexus-grid)"
            echo "  --help, -h         Show this help"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# ── OS detection ─────────────────────────────────────────────

detect_os() {
    local uname_s
    uname_s="$(uname -s)"
    case "$uname_s" in
        Linux)  OS="linux" ;;
        Darwin) OS="macos" ;;
        *)
            error "Unsupported OS: $uname_s"
            error "This script supports Linux and macOS."
            exit 1
            ;;
    esac
}

detect_pkg_manager() {
    if [[ "$OS" == "macos" ]]; then
        PKG_MGR="brew"
    elif command_exists apt-get; then
        PKG_MGR="apt"
    elif command_exists dnf; then
        PKG_MGR="dnf"
    else
        error "No supported package manager found (apt, dnf, or brew)."
        exit 1
    fi
}

# ── System dependencies ─────────────────────────────────────

install_system_deps() {
    info "Installing system dependencies via $PKG_MGR..."

    case "$PKG_MGR" in
        apt)
            sudo apt-get update -qq
            sudo apt-get install -y -qq \
                build-essential pkg-config libssl-dev libdbus-1-dev \
                git curl
            ;;
        dnf)
            sudo dnf install -y \
                gcc gcc-c++ make pkg-config openssl-devel dbus-devel \
                git curl
            ;;
        brew)
            brew install openssl pkg-config git curl
            ;;
    esac

    info "System dependencies installed."
}

# ── Rust toolchain ───────────────────────────────────────────

install_rust() {
    if command_exists rustup; then
        info "Rust already installed ($(rustc --version))."
        rustup update stable --no-self-update 2>/dev/null || true
    else
        info "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
        info "Rust installed ($(rustc --version))."
    fi
}

# ── uv (Python package manager) ─────────────────────────────

install_uv() {
    if command_exists uv; then
        info "uv already installed ($(uv --version))."
    else
        info "Installing uv..."
        curl -LsSf https://astral.sh/uv/install.sh | sh
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env" 2>/dev/null || true
        if ! command_exists uv; then
            export PATH="$HOME/.local/bin:$PATH"
        fi
        info "uv installed ($(uv --version))."
    fi
}

# ── Clone / update repo ─────────────────────────────────────

setup_repo() {
    if [[ -d "$INSTALL_DIR/.git" ]]; then
        info "Repository already exists at $INSTALL_DIR, pulling latest..."
        git -C "$INSTALL_DIR" pull --ff-only
    else
        info "Cloning repository to $INSTALL_DIR..."
        sudo mkdir -p "$INSTALL_DIR"
        sudo chown "$(id -u):$(id -g)" "$INSTALL_DIR"
        git clone "$REPO_URL" "$INSTALL_DIR"
    fi
}

# ── Build binaries ───────────────────────────────────────────

build_components() {
    cd "$INSTALL_DIR"

    local targets=()

    if "$INSTALL_DAEMON"; then
        targets+=("-p" "nexus-shell-daemon")
    fi
    if "$INSTALL_WORKER"; then
        targets+=("-p" "nexus-worker")
    fi

    if [[ ${#targets[@]} -gt 0 ]]; then
        info "Building Rust components (--release)..."
        cargo build --release "${targets[@]}"
        info "Rust build complete."
    fi

    if "$INSTALL_COORDINATOR"; then
        info "Setting up Python environment for coordinator..."
        install_uv
        uv sync --frozen 2>/dev/null || uv sync
        info "Coordinator Python environment ready."
    fi
}

# ── Post-build note ──────────────────────────────────────────

post_build_note() {
    if "$INSTALL_DAEMON"; then
        info "The daemon generates its iroh keypair and config directory"
        info "on first 'nexus-shell-daemon start'. No separate init step needed."
    fi
}

# ── systemd units ────────────────────────────────────────────

install_systemd_units() {
    if [[ "$OS" != "linux" ]]; then
        warn "systemd units are Linux-only, skipping on $OS."
        return
    fi

    if ! command_exists systemctl; then
        warn "systemctl not found, skipping systemd setup."
        return
    fi

    if ! confirm "Install and enable systemd units?"; then
        return
    fi

    local systemd_src="$INSTALL_DIR/configs/systemd"
    local systemd_dst="/etc/systemd/system"

    if "$INSTALL_DAEMON"; then
        sudo cp "$systemd_src/nexus-daemon.service" "$systemd_dst/"
        info "Installed nexus-daemon.service"
    fi

    if "$INSTALL_WORKER"; then
        sudo cp "$systemd_src/nexus-worker.service" "$systemd_dst/"
        info "Installed nexus-worker.service"
    fi

    if "$INSTALL_COORDINATOR"; then
        sudo cp "$systemd_src/nexus-coordinator.service" "$systemd_dst/"
        info "Installed nexus-coordinator.service"
    fi

    sudo systemctl daemon-reload

    # Create nexus user if missing
    if ! id -u nexus >/dev/null 2>&1; then
        info "Creating system user 'nexus'..."
        sudo useradd --system --create-home --shell /usr/sbin/nologin nexus
    fi

    if "$INSTALL_DAEMON" && confirm "Enable and start nexus-daemon now?"; then
        sudo systemctl enable --now nexus-daemon
        info "nexus-daemon started."
    fi

    if "$INSTALL_WORKER" && confirm "Enable and start nexus-worker now?"; then
        sudo systemctl enable --now nexus-worker
        info "nexus-worker started."
    fi

    if "$INSTALL_COORDINATOR" && confirm "Enable and start nexus-coordinator now?"; then
        sudo systemctl enable --now nexus-coordinator
        info "nexus-coordinator started."
    fi
}

# ── Summary ──────────────────────────────────────────────────

print_summary() {
    echo ""
    info "════════════════════════════════════════════════"
    info "  SBFB nexus-grid node installation complete"
    info "════════════════════════════════════════════════"
    echo ""
    info "Install directory: $INSTALL_DIR"
    echo ""

    if "$INSTALL_DAEMON"; then
        local daemon_bin="$INSTALL_DIR/target/release/nexus-shell-daemon"
        if [[ -x "$daemon_bin" ]]; then
            info "Daemon:      $daemon_bin"
        fi
    fi

    if "$INSTALL_WORKER"; then
        local worker_bin="$INSTALL_DIR/target/release/nexus-worker"
        if [[ -x "$worker_bin" ]]; then
            info "Worker:      $worker_bin"
        fi
    fi

    if "$INSTALL_COORDINATOR"; then
        info "Coordinator: uv run nexus-coordinator (in $INSTALL_DIR)"
    fi

    echo ""
    info "Quick start:"
    if "$INSTALL_DAEMON"; then
        info "  nexus-shell-daemon start --headless"
    fi
    if "$INSTALL_WORKER"; then
        info "  nexus-worker register my-worker"
        info "  nexus-worker start --headless"
    fi
    if "$INSTALL_COORDINATOR"; then
        info "  nexus-coordinator init my-project"
        info "  nexus-coordinator start my-project --host 0.0.0.0 --port 8765"
    fi

    if [[ "$OS" == "linux" ]] && command_exists systemctl; then
        echo ""
        info "systemd management:"
        info "  sudo systemctl status nexus-daemon"
        info "  sudo journalctl -u nexus-daemon -f"
    fi

    echo ""
}

# ── Main ─────────────────────────────────────────────────────

main() {
    info "SBFB nexus-grid node installer"
    echo ""

    detect_os
    detect_pkg_manager
    info "Detected: OS=$OS, package manager=$PKG_MGR"

    if confirm "Install system dependencies ($PKG_MGR)?"; then
        install_system_deps
    fi

    install_rust

    if confirm "Clone/update repository to $INSTALL_DIR?"; then
        setup_repo
    fi

    build_components
    post_build_note
    install_systemd_units
    print_summary
}

main
