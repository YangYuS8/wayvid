#!/usr/bin/env bash

set -euo pipefail

cat <<'EOF' >&2
packaging/appimage/build-appimage.sh has been retired.

This script previously built an AppImage around the legacy `wayvid-gui` / `wayvid-ctl` binaries.
Those binaries are no longer active deliverables, and this branch does not define a replacement AppImage pipeline for `apps/lwe/src-tauri`.

The directory is kept as packaging history only.
If AppImage support is needed for LWE, add it as new packaging work instead of using this retired legacy flow.
EOF

exit 1
