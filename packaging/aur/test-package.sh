#!/usr/bin/env bash

set -euo pipefail

cat <<'EOF' >&2
packaging/aur/test-package.sh has been retired.

It targeted AUR packaging for the legacy `wayvid-gui` / `wayvid-ctl` binaries, which are no longer current deliverables.
This branch keeps the AUR files only as legacy packaging reference and does not define an AUR package test flow for `apps/lwe/src-tauri`.

Use the active workspace verification commands instead:

  cargo metadata --no-deps
  cargo test -p lwe-app-shell
EOF

exit 1
