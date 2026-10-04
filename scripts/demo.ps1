# Copyright (C) 2026 Gokul Kartha
# SPDX-License-Identifier: GPL-3.0-or-later

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ComposeFile = Join-Path $RepoRoot "demos/basic/docker-compose.yml"

Write-Host "Starting StateLink basic demo..."
docker compose -f $ComposeFile up --build --abort-on-container-exit
