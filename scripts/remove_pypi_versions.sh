#!/usr/bin/env bash

# Remove or yank published versions from PyPI or Test PyPI using the tokens
# stored in ~/.config/specado/.env and the ~/.config/specado/.pypirc config.

set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
Usage: scripts/remove_pypi_versions.sh <pypi|testpypi> <version> [version...]

Examples:
  scripts/remove_pypi_versions.sh testpypi 0.2.0a16 0.2.0a17
  scripts/remove_pypi_versions.sh pypi 0.2.0a18

The script sources ~/.config/specado/.env to export PYPI_TOKEN / TEST_PYPI_TOKEN
and uses ~/.config/specado/.pypirc for repository definitions. Twine must be
available (python -m twine).
USAGE
}

if [[ $# -lt 2 ]]; then
  usage
  exit 1
fi

registry=$1
shift
package=${PACKAGE:-specado}

case "$registry" in
  pypi) repo="pypi" ; token_var="PYPI_TOKEN" ;;
  testpypi|test) repo="testpypi" ; token_var="TEST_PYPI_TOKEN" ;;
  *)
    echo "Unknown registry '$registry' (expected pypi or testpypi)" >&2
    exit 1
    ;;
esac

if [[ ! -f "$HOME/.config/specado/.env" ]]; then
  echo "Missing token file: $HOME/.config/specado/.env" >&2
  exit 1
fi

set -a
source "$HOME/.config/specado/.env"
set +a

if [[ -z ${!token_var:-} ]]; then
  echo "Token $token_var is not set in ~/.config/specado/.env" >&2
  exit 1
fi

export PYPI_RCFILE=${PYPI_RCFILE:-$HOME/.config/specado/.pypirc}

if ! python -m twine --version >/dev/null 2>&1; then
  echo "twine is required (install via 'python -m pip install --upgrade twine')." >&2
  exit 1
fi

reason=${YANK_REASON:-"Superseded prerelease"}

for version in "$@"; do
  echo "Yanking $package==$version from $repo..."
  python -m twine yank -r "$repo" "$package" "$version" \
    --non-interactive \
    --reason "$reason" || {
    echo "Failed to yank $package==$version from $repo" >&2
    exit 1
  }
done

echo "Done yanking versions."
