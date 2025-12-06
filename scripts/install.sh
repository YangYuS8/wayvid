#!/usr/bin/env bash
# wayvid installation script
# Supports user (~/.local) and system (/usr/local) installation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# Default values
INSTALL_MODE="user"
BUILD_RELEASE=true
INSTALL_SERVICE=false
VERBOSE=false

# Paths (will be set based on install mode)
BIN_DIR=""
SHARE_DIR=""
DESKTOP_DIR=""
ICON_DIR=""
SERVICE_DIR=""

# Script directory (project root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Install wayvid wallpaper manager.

Options:
    --user          Install to ~/.local (default)
    --system        Install to /usr/local (requires sudo)
    --no-build      Skip building (use existing binaries)
    --service       Also install systemd user service
    --verbose       Show verbose output
    -h, --help      Show this help message

Examples:
    $(basename "$0") --user           # User installation
    $(basename "$0") --system         # System-wide installation
    $(basename "$0") --user --service # User install with systemd service
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --user)
                INSTALL_MODE="user"
                shift
                ;;
            --system)
                INSTALL_MODE="system"
                shift
                ;;
            --no-build)
                BUILD_RELEASE=false
                shift
                ;;
            --service)
                INSTALL_SERVICE=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

setup_paths() {
    if [[ "$INSTALL_MODE" == "user" ]]; then
        BIN_DIR="${HOME}/.local/bin"
        SHARE_DIR="${HOME}/.local/share"
        DESKTOP_DIR="${SHARE_DIR}/applications"
        ICON_DIR="${SHARE_DIR}/icons/hicolor/scalable/apps"
        SERVICE_DIR="${HOME}/.config/systemd/user"
    else
        BIN_DIR="/usr/local/bin"
        SHARE_DIR="/usr/local/share"
        DESKTOP_DIR="/usr/share/applications"
        ICON_DIR="/usr/share/icons/hicolor/scalable/apps"
        SERVICE_DIR="/etc/systemd/user"
    fi
}

check_dependencies() {
    info "Checking dependencies..."
    
    local missing=()
    
    if ! command -v cargo &> /dev/null; then
        missing+=("cargo (Rust toolchain)")
    fi
    
    if ! command -v pkg-config &> /dev/null; then
        missing+=("pkg-config")
    fi
    
    # Check for required libraries
    if ! pkg-config --exists wayland-client 2>/dev/null; then
        missing+=("wayland-client (libwayland-dev)")
    fi
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing dependencies:"
        for dep in "${missing[@]}"; do
            echo "  - $dep"
        done
        exit 1
    fi
    
    success "All dependencies found"
}

build_release() {
    if [[ "$BUILD_RELEASE" != "true" ]]; then
        info "Skipping build (--no-build)"
        return
    fi
    
    info "Building release binaries..."
    cd "$SCRIPT_DIR"
    
    if [[ "$VERBOSE" == "true" ]]; then
        cargo build --release
    else
        cargo build --release --quiet
    fi
    
    # Verify binaries exist
    local binaries=("wayvid-gui" "wayvid-ctl")
    for bin in "${binaries[@]}"; do
        if [[ ! -f "target/release/$bin" ]]; then
            error "Binary not found: target/release/$bin"
            exit 1
        fi
    done
    
    success "Build complete"
}

install_binaries() {
    info "Installing binaries to $BIN_DIR..."
    
    mkdir -p "$BIN_DIR"
    
    local binaries=("wayvid-gui" "wayvid-ctl")
    for bin in "${binaries[@]}"; do
        local src="$SCRIPT_DIR/target/release/$bin"
        local dst="$BIN_DIR/$bin"
        
        if [[ "$INSTALL_MODE" == "system" ]]; then
            sudo install -m 755 "$src" "$dst"
        else
            install -m 755 "$src" "$dst"
        fi
        
        [[ "$VERBOSE" == "true" ]] && info "  Installed: $dst"
    done
    
    success "Binaries installed"
}

install_desktop_file() {
    info "Installing desktop file..."
    
    mkdir -p "$DESKTOP_DIR"
    
    local desktop_file="$SCRIPT_DIR/packaging/wayvid-gui.desktop"
    local dst="$DESKTOP_DIR/wayvid.desktop"
    
    if [[ "$INSTALL_MODE" == "system" ]]; then
        sudo install -m 644 "$desktop_file" "$dst"
    else
        install -m 644 "$desktop_file" "$dst"
    fi
    
    # Update desktop database
    if command -v update-desktop-database &> /dev/null; then
        if [[ "$INSTALL_MODE" == "system" ]]; then
            sudo update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
        else
            update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
        fi
    fi
    
    success "Desktop file installed"
}

install_icon() {
    info "Installing icon..."
    
    mkdir -p "$ICON_DIR"
    
    local icon_src="$SCRIPT_DIR/logo.svg"
    local icon_dst="$ICON_DIR/wayvid.svg"
    
    if [[ -f "$icon_src" ]]; then
        if [[ "$INSTALL_MODE" == "system" ]]; then
            sudo install -m 644 "$icon_src" "$icon_dst"
        else
            install -m 644 "$icon_src" "$icon_dst"
        fi
        
        # Update icon cache
        if command -v gtk-update-icon-cache &> /dev/null; then
            local icon_base
            if [[ "$INSTALL_MODE" == "system" ]]; then
                icon_base="/usr/share/icons/hicolor"
                sudo gtk-update-icon-cache -f -t "$icon_base" 2>/dev/null || true
            else
                icon_base="${HOME}/.local/share/icons/hicolor"
                gtk-update-icon-cache -f -t "$icon_base" 2>/dev/null || true
            fi
        fi
        
        success "Icon installed"
    else
        warn "Icon file not found: $icon_src (skipping)"
    fi
}

install_systemd_service() {
    if [[ "$INSTALL_SERVICE" != "true" ]]; then
        return
    fi
    
    info "Installing systemd user service..."
    
    mkdir -p "$SERVICE_DIR"
    
    local service_src="$SCRIPT_DIR/systemd/wayvid.service"
    local service_dst="$SERVICE_DIR/wayvid.service"
    
    if [[ -f "$service_src" ]]; then
        if [[ "$INSTALL_MODE" == "system" ]]; then
            sudo install -m 644 "$service_src" "$service_dst"
        else
            install -m 644 "$service_src" "$service_dst"
        fi
        
        # Reload systemd
        systemctl --user daemon-reload 2>/dev/null || true
        
        success "Systemd service installed"
        info "  Enable with: systemctl --user enable wayvid"
        info "  Start with:  systemctl --user start wayvid"
    else
        warn "Service file not found: $service_src (skipping)"
    fi
}

verify_installation() {
    info "Verifying installation..."
    
    local failed=false
    
    # Check binaries (only wayvid-gui and wayvid-ctl are required)
    for bin in wayvid-gui wayvid-ctl; do
        if [[ -x "$BIN_DIR/$bin" ]]; then
            [[ "$VERBOSE" == "true" ]] && success "  $bin: OK"
        else
            error "  $bin: NOT FOUND"
            failed=true
        fi
    done
    
    # Check desktop file
    if [[ -f "$DESKTOP_DIR/wayvid.desktop" ]]; then
        [[ "$VERBOSE" == "true" ]] && success "  Desktop file: OK"
    else
        warn "  Desktop file: NOT FOUND"
    fi
    
    # Check if binaries are in PATH
    if command -v wayvid-gui &> /dev/null; then
        success "wayvid-gui is in PATH"
    else
        warn "wayvid-gui not in PATH"
        if [[ "$INSTALL_MODE" == "user" ]]; then
            info "Add to your shell config: export PATH=\"\$HOME/.local/bin:\$PATH\""
        fi
    fi
    
    if [[ "$failed" == "true" ]]; then
        error "Installation verification failed"
        exit 1
    fi
}

print_summary() {
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  wayvid installation complete!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Installed to: $BIN_DIR"
    echo ""
    echo "Commands:"
    echo "  wayvid       - Wallpaper daemon"
    echo "  wayvid-gui   - Graphical interface"
    echo "  wayvid-ctl   - Command-line control"
    echo ""
    echo "Launch from:"
    echo "  - Application menu (search 'wayvid')"
    echo "  - Terminal: wayvid-gui"
    echo ""
    if [[ "$INSTALL_SERVICE" == "true" ]]; then
        echo "Systemd service:"
        echo "  systemctl --user enable --now wayvid"
        echo ""
    fi
    echo "To uninstall:"
    echo "  $SCRIPT_DIR/scripts/uninstall.sh --$INSTALL_MODE"
    echo ""
}

main() {
    echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║           wayvid Installation Script                      ║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    parse_args "$@"
    setup_paths
    
    info "Installation mode: $INSTALL_MODE"
    info "Install directory: $BIN_DIR"
    echo ""
    
    check_dependencies
    build_release
    install_binaries
    install_desktop_file
    install_icon
    install_systemd_service
    verify_installation
    print_summary
}

main "$@"
