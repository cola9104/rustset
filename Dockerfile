# RustSet - Multi-stage Docker Build
# Build stage
FROM rust:1.75-bookworm as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/
COPY frontend/Cargo.toml ./frontend/
COPY shared/Cargo.toml ./shared/

# Create dummy main.rs to build dependencies
RUN mkdir -p backend/src frontend/src shared/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "fn main() {}" > frontend/src/main.rs && \
    echo "pub fn dummy() {}" > shared/src/lib.rs

# Build dependencies
RUN cargo build --release && rm -rf backend/src frontend/src shared/src

# Copy source code
COPY backend/src ./backend/src
COPY frontend/src ./frontend/src
COPY shared/src ./shared/src

# Build application
RUN cargo build --release -p backend

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/backend /app/backend

# Copy static files (frontend)
COPY --from=builder /app/target/dx/frontend/release/web/public /app/static

# Create non-root user
RUN useradd -m -u 1000 rustset && chown -R rustset:rustset /app
USER rustset

# Environment
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://rustset:rustset_password@db:5432/rustset

EXPOSE 3003

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3003/api/health || exit 1

CMD ["./backend"]
