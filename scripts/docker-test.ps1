# Copyright (C) 2026 Gokul Kartha
# SPDX-License-Identifier: GPL-3.0-or-later

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

docker run --rm `
  -v "${Root}:/workspace" `
  -w /workspace `
  rust:1.79-bookworm `
  bash -lc "rustup component add rustfmt clippy && cargo fmt --all -- --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace --all-targets"
