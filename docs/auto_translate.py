#!/usr/bin/env python3

import sys


MESSAGE = """Documentation helper automation was retired during the repository reset.

Use the reset-era markdown docs directly:
- docs/product/overview.md
- docs/product/roadmap.md
- docs/product/repository-reset.md
- docs/archive/legacy-wayvid-summary.md
"""


def main() -> int:
    print(MESSAGE)
    return 0


if __name__ == "__main__":
    sys.exit(main())
