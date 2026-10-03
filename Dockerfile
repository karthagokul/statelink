# Copyright (C) 2026 Gokul Kartha
# SPDX-License-Identifier: GPL-3.0-or-later

FROM rust:1.79-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release \
    -p statelink-server \
    -p statelink-mqtt \
    -p statelink-client --examples

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/statelink-server /usr/local/bin/statelink-server
COPY --from=builder /src/target/release/statelink-mqtt /usr/local/bin/statelink-mqtt
COPY --from=builder /src/target/release/examples/producer /usr/local/bin/statelink-producer
COPY --from=builder /src/target/release/examples/consumer /usr/local/bin/statelink-consumer

USER nobody:nogroup
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/statelink-server"]
