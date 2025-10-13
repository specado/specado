#!/usr/bin/env python3
"""Emit workspace crate package names for GitHub Actions outputs."""

from __future__ import annotations

import os
import pathlib
from typing import Dict

try:
    import tomllib  # Python 3.11+
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


TARGET_CRATES = ("specado-schemas", "specado-core", "specado", "specado-cli")


def load_workspace(root: pathlib.Path) -> Dict:
    cargo_toml = root / "Cargo.toml"
    return tomllib.loads(cargo_toml.read_text())


def crate_package_name(root: pathlib.Path, deps: Dict, crate: str) -> str:
    pkg = None
    spec = deps.get(crate)
    if isinstance(spec, dict):
        pkg = spec.get("package")

    manifest = root / "crates" / crate / "Cargo.toml"
    if pkg is None and manifest.exists():
        data = tomllib.loads(manifest.read_text())
        pkg = data.get("package", {}).get("name")

    return pkg or crate


def main() -> None:
    root = pathlib.Path(__file__).resolve().parent.parent
    workspace = load_workspace(root)
    deps = workspace.get("workspace", {}).get("dependencies", {})

    output_path = os.environ.get("GITHUB_OUTPUT")
    if not output_path:
        raise SystemExit("GITHUB_OUTPUT is required")

    with open(output_path, "a", encoding="utf-8") as fh:
        for crate in TARGET_CRATES:
            pkg = crate_package_name(root, deps, crate)
            key = crate.replace("-", "_") + "_pkg"
            fh.write(f"{key}={pkg}\n")


if __name__ == "__main__":
    main()
