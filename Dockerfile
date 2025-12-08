# Stage 1: Build Frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app/frontend

# Copy package files
COPY frontend/package*.json ./
RUN npm ci

# Copy source and build
COPY frontend/ ./
# Create the output directory structure expected by vite.config.ts
RUN mkdir -p ../backend/static
RUN npm run build

# Stage 2: Chef - Common base for Rust builds
FROM lukemathwalker/cargo-chef:latest-rust-1.83 AS chef
WORKDIR /app/backend
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev && rm -rf /var/lib/apt/lists/*

# Stage 3: Planner - Generate recipe
FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 4: Builder - Build dependencies and app
FROM chef AS builder
COPY --from=planner /app/backend/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY backend/ .
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
