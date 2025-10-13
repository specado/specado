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
    workspace_deps = workspace["workspace"].get("dependencies", {})
    for crate in ("specado-core", "specado-schemas", "specado"):
        spec = workspace_deps.get(crate)
        if not spec:
            errors.append(f"Missing workspace dependency entry for {crate}")
            continue
        version = spec.get("version")
        expected_pin = f"={expected_version}"
        if version != expected_pin:
            errors.append(
                f"workspace dependency {crate} version {version} != {expected_pin}"
            )

    node_pkg = json.loads(
        (root / "crates" / "specado-node" / "package.json").read_text()
    )
    node_version = node_pkg["version"]
    if node_version != expected_version:
        errors.append(
            f"specado-node package.json version {node_version} != {expected_version}"
        )
    optional = node_pkg.get("optionalDependencies", {})
    for name, value in sorted(optional.items()):
        if name.startswith("specado-") and value != expected_version:
            errors.append(
                f"specado-node optional dependency {name} version {value} != {expected_version}"
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
