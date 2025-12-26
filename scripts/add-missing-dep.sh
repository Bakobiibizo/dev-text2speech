#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MISSING_REQ="$ROOT_DIR/scripts/missing-deps.txt"

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <package> [<package> ...]" >&2
  exit 2
fi

for pkg in "$@"; do
  echo "$pkg" >> "$MISSING_REQ"
  echo "[missing-deps] recorded: $pkg"
done

echo "[missing-deps] current list:"
sort -u "$MISSING_REQ"
