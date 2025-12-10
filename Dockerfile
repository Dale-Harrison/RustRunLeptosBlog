# Stage 1: Build Frontend (Leptos)
FROM rust:latest AS frontend-builder
WORKDIR /app/frontend
# Install wasm-pack via binary installer for speed
RUN apt-get update && apt-get install -y curl
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Copy Leptos source
COPY leptos_frontend/ .
# Build Wasm
RUN wasm-pack build --target web --release

# Create the output directory structure
RUN mkdir -p /app/backend/static/pkg
RUN cp -r pkg/* /app/backend/static/pkg/
RUN cp index.html /app/backend/static/

# Stage 2: Builder - Build directly
FROM lukemathwalker/cargo-chef:latest-rust-1 AS builder
WORKDIR /app/backend
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev && rm -rf /var/lib/apt/lists/*
COPY backend/ .
RUN ls -la
# Force usage of lockfile if present
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/backend/target/release/backend ./server

# Copy the static assets
# Note: The frontend build output went to ../backend/static in the frontend-builder stage
COPY --from=frontend-builder /app/backend/static ./static

# Configure environment
ENV PORT=8080
ENV RUST_LOG=info
# Default database URL (can be overridden by env var)
ENV DATABASE_URL="sqlite:blog.db?mode=rwc"

# Expose the port
EXPOSE 8080

# Run the application
CMD ["sh", "-c", "echo 'Container starting...' && ls -la && ./server"]
