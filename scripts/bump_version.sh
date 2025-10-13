#!/bin/bash
# Version bump script for Specado
# Usage: ./scripts/bump_version.sh <new_version>
# Example: ./scripts/bump_version.sh 0.2.0-beta.1

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <new_version>"
    echo "Example: $0 0.2.0-beta.1"
    exit 1
fi

NEW_VERSION=$1
OLD_VERSION=$(grep 'version = ' Cargo.toml | head -1 | sed 's/.*version = "\(.*\)"/\1/')

echo "Bumping version from $OLD_VERSION to $NEW_VERSION"

# Update workspace Cargo.toml (package version and internal pinned dependencies)
sed -i.bak "s/version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
sed -i.bak "s/\"=$OLD_VERSION\"/\"=$NEW_VERSION\"/g" Cargo.toml

# Update Node.js package.json (package + native optional dependencies)
python - "$OLD_VERSION" "$NEW_VERSION" <<'PY'
import json
import pathlib
import sys

old, new = sys.argv[1:3]
path = pathlib.Path("crates/specado-node/package.json")
data = json.loads(path.read_text())
data["version"] = new
optional = data.get("optionalDependencies", {})
for name, value in list(optional.items()):
    if value == old:
        optional[name] = new
path.write_text(json.dumps(data, indent=2) + "\n")
PY

# Update Python pyproject.toml
sed -i.bak "s/version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" pyproject.toml

# Remove backup files
find . -name "*.bak" -delete

echo "Version bump complete!"
echo ""
echo "Next steps:"
echo "1. Review changes: git diff"
echo "2. Run tests: cargo test --workspace"
echo "3. Commit changes: git add -A && git commit -m 'chore: bump version to $NEW_VERSION'"
echo "4. Create PR or push to trigger release workflow"
