# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl \
    && rm -rf /var/lib/apt/lists/*

# WASM compilation target
RUN rustup target add wasm32-unknown-unknown

# Install cargo-leptos (cached as its own layer — only rebuilt on FROM/apt changes)
RUN cargo install cargo-leptos --version 0.3.5 --locked

WORKDIR /app
COPY . .

RUN cargo leptos build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Server binary and pre-compiled static assets (JS, WASM, CSS)
COPY --from=builder /app/target/release/wishr ./wishr
COPY --from=builder /app/target/site ./site

# Persistent volume mount point for the SQLite database
RUN mkdir -p /app/data

ENV LEPTOS_SITE_ADDR=0.0.0.0:3000
ENV LEPTOS_SITE_ROOT=/app/site
ENV DATABASE_URL=sqlite:/app/data/wishr.db

EXPOSE 3000

CMD ["/app/wishr"]
