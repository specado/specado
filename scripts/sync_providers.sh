#!/usr/bin/env bash
# Synchronize the bundled provider catalogs for packaging targets.
# Usage: ./scripts/sync_providers.sh
# Copies crates/specado-providers/providers into the Python and Node bundles.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="${ROOT_DIR}/crates/specado-providers/providers"
PY_DEST="${ROOT_DIR}/python/specado/providers"
NODE_DEST="${ROOT_DIR}/crates/specado-node/providers"

if [ ! -d "$SOURCE" ]; then
  echo "Provider catalog not found at $SOURCE" >&2
  exit 1
fi

rsync -a --delete "$SOURCE/" "$PY_DEST/"
rsync -a --delete "$SOURCE/" "$NODE_DEST/"

echo "Synced providers into:"
echo "  - $PY_DEST"
echo "  - $NODE_DEST"
