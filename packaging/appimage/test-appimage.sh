#!/usr/bin/env bash

set -euo pipefail

cat <<'EOF' >&2
packaging/appimage/test-appimage.sh has been retired.

It used to validate AppImages built for the legacy `wayvid-gui` / `wayvid-ctl` binaries.
Those binaries are retired references, and this repository does not currently publish an AppImage flow for `apps/lwe/src-tauri`.

Use the active workspace verification commands instead:

  cargo metadata --no-deps
  cargo test -p lwe-app-shell
EOF

exit 1
