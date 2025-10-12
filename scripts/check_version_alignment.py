#!/usr/bin/env python3

import json
import pathlib
import sys

try:
    import tomllib  # Python 3.11+
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


def main(expected_version: str) -> int:
    root = pathlib.Path(__file__).resolve().parent.parent

    errors = []

    workspace = tomllib.loads((root / "Cargo.toml").read_text())
    workspace_version = workspace["workspace"]["package"]["version"]
    if workspace_version != expected_version:
        errors.append(
            f"Cargo workspace version {workspace_version} != {expected_version}"
        )

    node_pkg = json.loads(
        (root / "crates" / "specado-node" / "package.json").read_text()
    )
    node_version = node_pkg["version"]
    if node_version != expected_version:
        errors.append(
            f"specado-node package.json version {node_version} != {expected_version}"
        )

    pyproject = tomllib.loads((root / "pyproject.toml").read_text())
    python_version = pyproject["project"]["version"]
    if python_version != expected_version:
        errors.append(
            f"pyproject.toml version {python_version} != {expected_version}"
        )

    if errors:
        for error in errors:
            print(f"::error::{error}")
        return 1

    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: check_version_alignment.py <expected-version>", file=sys.stderr)
        sys.exit(2)
    sys.exit(main(sys.argv[1]))
