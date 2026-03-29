#!/usr/bin/env bash

set -euo pipefail

cat <<'EOF' >&2
scripts/install.sh has been retired.

This script used to install the legacy `wayvid-gui` and `wayvid-ctl` binaries, which are no longer active product deliverables.
The current repository path is centered on `apps/lwe/src-tauri`, and this branch does not define a replacement install flow for that LWE shell.

If you need to clean up an old legacy install, use `scripts/uninstall.sh`.
If you need current verification, run:

  cargo metadata --no-deps
  cargo test -p lwe-app-shell
EOF

exit 1
