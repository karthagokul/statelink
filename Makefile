# Copyright (C) 2026 Gokul Kartha
# SPDX-License-Identifier: GPL-3.0-or-later

.PHONY: test check docker-test demo

test:
	cargo test --workspace --all-targets

check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cargo test --workspace --all-targets

docker-test:
	docker run --rm -v "$(CURDIR):/workspace" -w /workspace rust:1.85-bookworm bash -lc \
		'rustup component add rustfmt clippy && cargo fmt --all -- --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace --all-targets'

demo:
	docker compose up --build --abort-on-container-exit
