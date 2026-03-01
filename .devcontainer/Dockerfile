FROM rust:1.85-slim

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl git \
    && rm -rf /var/lib/apt/lists/*

# WASM compilation target
RUN rustup target add wasm32-unknown-unknown

# cargo-leptos — download pre-built binary (avoids long compile + Rust version constraints)
RUN curl -fL https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.5/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz \
    -o /tmp/cl.tar.gz \
    && tar -xzf /tmp/cl.tar.gz -C /tmp \
    && find /tmp -name 'cargo-leptos' -type f -exec install -m755 {} /usr/local/cargo/bin/cargo-leptos \; \
    && rm /tmp/cl.tar.gz

# Create non-root user (Claude Code blocks --dangerously-skip-permissions for root)
RUN groupadd --gid 1000 vscode \
    && useradd --uid 1000 --gid 1000 -m vscode \
    && chown -R vscode:vscode /usr/local/cargo /usr/local/rustup

USER vscode
