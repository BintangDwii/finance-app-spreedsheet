# Workspace-aware build. Context MUST be the repo root:
#   1. Build the SPA first: `cd frontend && trunk build --release`
#   2. `docker build -f backend-rust/Dockerfile .`
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY backend-rust ./backend-rust
COPY frontend ./frontend
RUN cargo build --release -p corporate-finance-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 10001 -s /usr/sbin/nologin appuser
COPY --from=builder /app/target/release/corporate-finance-api /usr/local/bin/app
COPY --from=builder /app/frontend/dist /app/dist
ENV FRONTEND_DIR=/app/dist
USER appuser
EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 CMD curl -fsS http://127.0.0.1:8000/health || exit 1
CMD ["app"]
