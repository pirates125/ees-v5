# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser && \
    mkdir -p /data/sessions && \
    mkdir -p /app/screenshots && \
    chown -R appuser:appuser /data /app

# Copy binary from builder
COPY --from=builder /app/target/release/sigorta-server /app/sigorta-server

# Set timezone
ENV TZ=Europe/Istanbul

USER appuser

EXPOSE 8099

CMD ["/app/sigorta-server"]

