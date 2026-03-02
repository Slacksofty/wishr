# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:slim AS builder

ARG CARGO_LEPTOS_VERSION=0.3.5

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl \
    && rm -rf /var/lib/apt/lists/*

# WASM compilation target
RUN rustup target add wasm32-unknown-unknown

# Install cargo-leptos via prebuilt binary (avoids Rust MSRV chasing)
RUN curl -L \
    "https://github.com/leptos-rs/cargo-leptos/releases/download/v${CARGO_LEPTOS_VERSION}/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xz --strip-components=1 -C /usr/local/bin \
      "cargo-leptos-x86_64-unknown-linux-gnu/cargo-leptos"

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
