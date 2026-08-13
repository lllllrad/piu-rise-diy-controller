# syntax=docker/dockerfile:1.7
FROM rust:1.97-bookworm

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        build-essential \
        file \
        libasound2-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        libssl-dev \
        libwebkit2gtk-4.1-dev \
        libxdo-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

COPY Cargo.toml Cargo.lock build.rs ./
COPY assets ./assets
COPY src ./src
COPY src-tauri ./src-tauri
COPY web ./web

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/workspace/target \
    cargo check --manifest-path src-tauri/Cargo.toml

CMD ["cargo", "check", "--manifest-path", "src-tauri/Cargo.toml"]
