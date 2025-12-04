#!/usr/bin/env bash
# wayvid uninstallation script
# Supports user (~/.local) and system (/usr/local) uninstallation

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
REMOVE_CONFIG=false
VERBOSE=false

# Paths (will be set based on install mode)
BIN_DIR=""
SHARE_DIR=""
DESKTOP_DIR=""
ICON_DIR=""
SERVICE_DIR=""

usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Uninstall wayvid wallpaper manager.

Options:
    --user          Uninstall from ~/.local (default)
    --system        Uninstall from /usr/local (requires sudo)
    --remove-config Also remove configuration files
    --verbose       Show verbose output
    -h, --help      Show this help message

Examples:
    $(basename "$0") --user           # User uninstallation
    $(basename "$0") --system         # System-wide uninstallation
    $(basename "$0") --remove-config  # Also remove config
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
            --remove-config)
                REMOVE_CONFIG=true
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

stop_daemon() {
    info "Stopping wayvid daemon if running..."
    
    # Try to stop via systemctl
    if systemctl --user is-active wayvid &>/dev/null; then
        systemctl --user stop wayvid 2>/dev/null || true
        success "Stopped systemd service"
    fi
    
    # Also try to kill any running processes
    pkill -f "wayvid " 2>/dev/null || true
    pkill -f "wayvid-gui" 2>/dev/null || true
}

disable_service() {
    info "Disabling systemd service..."
    
    if systemctl --user is-enabled wayvid &>/dev/null; then
        systemctl --user disable wayvid 2>/dev/null || true
        success "Disabled systemd service"
    fi
}

remove_file() {
    local file="$1"
    local use_sudo="$2"
    
    if [[ -f "$file" || -L "$file" ]]; then
        if [[ "$use_sudo" == "true" ]]; then
            sudo rm -f "$file"
        else
            rm -f "$file"
        fi
        [[ "$VERBOSE" == "true" ]] && info "  Removed: $file"
        return 0
    fi
    return 1
}

remove_binaries() {
    info "Removing binaries..."
    
    local use_sudo="false"
    [[ "$INSTALL_MODE" == "system" ]] && use_sudo="true"
    
    local removed=0
    for bin in wayvid wayvid-gui wayvid-ctl; do
        if remove_file "$BIN_DIR/$bin" "$use_sudo"; then
            ((removed++)) || true
        fi
    done
    
    if [[ $removed -gt 0 ]]; then
        success "Removed $removed binaries"
    else
        warn "No binaries found to remove"
    fi
}

remove_desktop_file() {
    info "Removing desktop file..."
    
    local use_sudo="false"
    [[ "$INSTALL_MODE" == "system" ]] && use_sudo="true"
    
    if remove_file "$DESKTOP_DIR/wayvid-gui.desktop" "$use_sudo"; then
        # Update desktop database
        if command -v update-desktop-database &> /dev/null; then
            if [[ "$use_sudo" == "true" ]]; then
                sudo update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
            else
                update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
            fi
        fi
        success "Removed desktop file"
    else
        warn "Desktop file not found"
    fi
}

remove_icon() {
    info "Removing icon..."
    
    local use_sudo="false"
    [[ "$INSTALL_MODE" == "system" ]] && use_sudo="true"
    
    if remove_file "$ICON_DIR/wayvid.svg" "$use_sudo"; then
        # Update icon cache
        if command -v gtk-update-icon-cache &> /dev/null; then
            local icon_base
            if [[ "$use_sudo" == "true" ]]; then
                icon_base="/usr/share/icons/hicolor"
                sudo gtk-update-icon-cache -f -t "$icon_base" 2>/dev/null || true
            else
                icon_base="${HOME}/.local/share/icons/hicolor"
                gtk-update-icon-cache -f -t "$icon_base" 2>/dev/null || true
            fi
        fi
        success "Removed icon"
    else
        warn "Icon not found"
    fi
}

remove_service() {
    info "Removing systemd service file..."
    
    local use_sudo="false"
    [[ "$INSTALL_MODE" == "system" ]] && use_sudo="true"
    
    if remove_file "$SERVICE_DIR/wayvid.service" "$use_sudo"; then
        systemctl --user daemon-reload 2>/dev/null || true
        success "Removed systemd service"
    else
        [[ "$VERBOSE" == "true" ]] && warn "Service file not found"
    fi
}

remove_config() {
    if [[ "$REMOVE_CONFIG" != "true" ]]; then
        return
    fi
    
    info "Removing configuration files..."
    
    local config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/wayvid"
    local cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/wayvid"
    local data_dir="${XDG_DATA_HOME:-$HOME/.local/share}/wayvid"
    
    for dir in "$config_dir" "$cache_dir" "$data_dir"; do
        if [[ -d "$dir" ]]; then
            rm -rf "$dir"
            [[ "$VERBOSE" == "true" ]] && info "  Removed: $dir"
        fi
    done
    
    success "Configuration files removed"
}

print_summary() {
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  wayvid uninstallation complete!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    
    if [[ "$REMOVE_CONFIG" != "true" ]]; then
        echo "Configuration files were preserved at:"
        echo "  ~/.config/wayvid/"
        echo "  ~/.cache/wayvid/"
        echo "  ~/.local/share/wayvid/"
        echo ""
        echo "To also remove config files, run:"
        echo "  $(basename "$0") --$INSTALL_MODE --remove-config"
        echo ""
    fi
}

main() {
    echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║           wayvid Uninstallation Script                    ║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    parse_args "$@"
    setup_paths
    
    info "Uninstallation mode: $INSTALL_MODE"
    echo ""
    
    stop_daemon
    disable_service
    remove_binaries
    remove_desktop_file
    remove_icon
    remove_service
    remove_config
    print_summary
}

main "$@"
