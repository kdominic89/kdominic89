#!/usr/bin/env python3
"""Compatibility entry point for the canonical locked Rust preview generator."""

from __future__ import annotations

import argparse
import shutil
import subprocess
from pathlib import Path


def main() -> int:
    """Forward explicit preview paths to Rust without maintaining a second model."""
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, default=root / "config/profile.toml")
    parser.add_argument("--snapshot", type=Path, default=root / "config/offline-snapshot.json")
    parser.add_argument("--assets", type=Path, default=root / "assets")
    parser.add_argument("--docs", type=Path, default=root / "docs")
    args = parser.parse_args()
    cargo = shutil.which("cargo")
    if cargo is None or not (root / "Cargo.lock").is_file():
        parser.error("cargo and the committed Cargo.lock are required")

    command = [
        cargo,
        "run",
        "--locked",
        "-p",
        "sourcefield-cli",
        "--",
        "generate",
        "--offline",
        "--no-history",
        "--config",
        str(args.config),
        "--fallback-snapshot",
        str(args.snapshot),
        "--assets",
        str(args.assets),
        "--docs",
        str(args.docs),
    ]

    return subprocess.run(command, cwd=root, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
