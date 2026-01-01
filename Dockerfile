# =============================================================================
# AlphaField Docker Build
# Multi-stage build for optimized production images
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder - Build the application
# -----------------------------------------------------------------------------
FROM rust:slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Ensure sqlx doesn't try to connect to DB during compilation
ENV SQLX_OFFLINE=true

# Update cargo
RUN cargo update

# Build application
RUN cargo build --release --bin dashboard_server

# -----------------------------------------------------------------------------
# Stage 2: Runtime - Minimal production image
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# OCI Labels
LABEL org.opencontainers.image.source="https://github.com/adamf123git/AlphaField" \
    org.opencontainers.image.description="AlphaField Trading Dashboard" \
    org.opencontainers.image.licenses="MIT"

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 alphafield

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/dashboard_server /app/dashboard_server

# Copy static assets for dashboard
COPY --from=builder /app/crates/dashboard/static /app/static

# Set ownership
RUN chown -R alphafield:alphafield /app

USER alphafield

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# Default command
CMD ["./dashboard_server"]
