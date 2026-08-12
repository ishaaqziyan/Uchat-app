# build with:
#   docker build -f backend/Dockerfile -t uchat-backend --build-arg API_URL=https://api.example.com/ .

# syntax=docker/dockerfile:1.7

FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

# Builds just the dependency graph first (cached across builds as long as
# Cargo.toml/Cargo.lock don't change) so editing application source doesn't
# force a full recompile of every dependency.
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json -p uchat_server

ARG API_URL=http://127.0.0.1:8070/
RUN echo "API_URL = \"${API_URL}\"" > .env

COPY . .

# /app/target is a cache mount, so its contents disappear once this RUN
# finishes; copy the binary out to a normal image layer before that happens.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release -p uchat_server --bin api && \
    cp target/release/api /app/api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/api ./api

ENV API_BIND=0.0.0.0:8070
EXPOSE 8070

ENTRYPOINT ["./api"]
