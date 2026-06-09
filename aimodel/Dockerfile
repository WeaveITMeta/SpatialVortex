# Vortex API — OpenAI-compatible server with API keys + usage billing
#
# Build from the REPO ROOT (path deps live in ../eustress):
#   docker build -f aimodel/Dockerfile -t vortex-api .
# Run:
#   docker run -p 7000:7000 \
#     -e VORTEX_HOST=0.0.0.0 \
#     -e VORTEX_ADMIN_KEY=<your-admin-key> \
#     -v vortex-keys:/app/data \
#     -e VORTEX_KEYS_PATH=/app/data/vortex_keys.json \
#     vortex-api

FROM rust:1.87-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Path dependencies referenced from aimodel/Cargo.toml
COPY eustress/crates/vortex-core ./eustress/crates/vortex-core
COPY eustress/crates/vortex-grid2d ./eustress/crates/vortex-grid2d
COPY eustress/EustressEngine/eustress/.patches/iggy_common-0.9.0 ./eustress/EustressEngine/eustress/.patches/iggy_common-0.9.0

# The crate itself
COPY aimodel ./aimodel

WORKDIR /build/aimodel
RUN cargo build --release --bin vortex-api --features web

# ── Runtime ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 vortex && mkdir -p /app/data && chown -R vortex:vortex /app

WORKDIR /app
COPY --from=builder /build/aimodel/target/release/vortex-api /app/vortex-api

USER vortex
EXPOSE 7000

ENV RUST_LOG=info \
    VORTEX_HOST=0.0.0.0 \
    VORTEX_PORT=7000 \
    VORTEX_KEYS_PATH=/app/data/vortex_keys.json

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:7000/health || exit 1

CMD ["/app/vortex-api"]
