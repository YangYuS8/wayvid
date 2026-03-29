#!/usr/bin/env bash

set -euo pipefail

INSTALL_MODE="user"
REMOVE_CONFIG="false"

usage() {
	cat <<'EOF'
Usage: uninstall.sh [--user|--system] [--remove-config]

Removes retired legacy wayvid install artifacts if they are still present.
This cleanup targets the old `wayvid`, `wayvid-gui`, and `wayvid-ctl` install locations only.
It is not an uninstall flow for the active `apps/lwe/src-tauri` shell.
EOF
}

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
		REMOVE_CONFIG="true"
		shift
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		usage >&2
		exit 1
		;;
	esac
done

if [[ "$INSTALL_MODE" == "system" ]]; then
	BIN_DIR="/usr/local/bin"
	DESKTOP_DIR="/usr/share/applications"
	ICON_DIR="/usr/share/icons/hicolor/scalable/apps"
	SERVICE_DIR="/etc/systemd/user"
	RM=(sudo rm -f)
else
	BIN_DIR="${HOME}/.local/bin"
	DESKTOP_DIR="${HOME}/.local/share/applications"
	ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"
	SERVICE_DIR="${HOME}/.config/systemd/user"
	RM=(rm -f)
fi

echo "Retired legacy cleanup for old wayvid installs"
echo "Mode: $INSTALL_MODE"

systemctl --user stop wayvid 2>/dev/null || true
systemctl --user disable wayvid 2>/dev/null || true
pkill -f "wayvid " 2>/dev/null || true
pkill -f "wayvid-gui" 2>/dev/null || true

for path in \
	"$BIN_DIR/wayvid" \
	"$BIN_DIR/wayvid-gui" \
	"$BIN_DIR/wayvid-ctl" \
	"$DESKTOP_DIR/wayvid.desktop" \
	"$ICON_DIR/wayvid.svg" \
	"$SERVICE_DIR/wayvid.service"; do
	if [[ -e "$path" || -L "$path" ]]; then
		"${RM[@]}" "$path"
		echo "removed: $path"
	fi
done

if [[ "$REMOVE_CONFIG" == "true" ]]; then
	rm -rf \
		"${XDG_CONFIG_HOME:-$HOME/.config}/wayvid" \
		"${XDG_CACHE_HOME:-$HOME/.cache}/wayvid" \
		"${XDG_DATA_HOME:-$HOME/.local/share}/wayvid"
	echo "removed: legacy wayvid config directories"
fi

echo "Legacy cleanup finished. No active LWE uninstall flow is defined here."
