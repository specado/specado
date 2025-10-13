#!/usr/bin/env python3

import pathlib
import sys

try:
    import tomllib  # Python 3.11+
except ImportError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


CRATES = (
    "specado-core",
    "specado-schemas",
    "specado-providers",
    "specado",
    "specado-cli",
)


def main(expected_version: str) -> int:
    root = pathlib.Path(__file__).resolve().parent.parent

    errors = []
    for crate in CRATES:
        data = tomllib.loads((root / "crates" / crate / "Cargo.toml").read_text())
        crate_version = data["package"]["version"]
        if isinstance(crate_version, dict) and crate_version.get("workspace", False):
            continue
        if crate_version != expected_version:
            errors.append(f"{crate} version {crate_version} != {expected_version}")

    if errors:
        for error in errors:
            print(f"::error::{error}")
        return 1

    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: check_crate_versions.py <expected-version>", file=sys.stderr)
        sys.exit(2)
    sys.exit(main(sys.argv[1]))
