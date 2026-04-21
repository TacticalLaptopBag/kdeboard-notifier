#!/usr/bin/env bash
# Generates cargo-sources.json from Cargo.lock using flatpak-cargo-generator.
# Run this once (and again after any dependency change) before building the Flatpak.
#
# Requires: python3, git
set -euo pipefail

TOOL_DIR=$(mktemp -d)
trap 'rm -rf "$TOOL_DIR"' EXIT

git clone --depth=1 https://github.com/flatpak/flatpak-builder-tools.git "$TOOL_DIR"
python3 -m venv "$TOOL_DIR/venv"
"$TOOL_DIR/venv/bin/pip" install --quiet aiohttp tomlkit
"$TOOL_DIR/venv/bin/python3" "$TOOL_DIR/cargo/flatpak-cargo-generator.py" Cargo.lock -o cargo-sources.json
echo "cargo-sources.json written."
