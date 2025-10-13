#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "Regenerating golden snapshots with UPDATE_GOLDEN=1..." >&2
UPDATE_GOLDEN=1 cargo test -p specado-core-temp --test golden
