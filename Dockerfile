# SpatialVortex Production Dockerfile
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY examples ./examples

# Build release binary
RUN cargo build --bin api_server --features onnx,lake,voice --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 spatialvortex

# Create directories
RUN mkdir -p /app/data /app/models && \
    chown -R spatialvortex:spatialvortex /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/api_server /app/api_server

# Copy configuration example
COPY config.toml.example /app/config.toml.example

# Switch to app user
USER spatialvortex

# Expose port
EXPOSE 8080

# Set environment
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/api/v1/health || exit 1

# Run server
CMD ["/app/api_server"]
