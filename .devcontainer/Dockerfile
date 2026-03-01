FROM rust:1.83-slim

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl git \
    && rm -rf /var/lib/apt/lists/*

# WASM compilation target
RUN rustup target add wasm32-unknown-unknown

# cargo-leptos — built once into the image layer, not re-installed on rebuild
RUN cargo install cargo-leptos --version 0.3.5 --locked
