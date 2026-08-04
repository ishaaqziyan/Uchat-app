# build with:
#   docker build -f frontend/Dockerfile -t uchat-frontend --build-arg API_URL=https://api.example.com/ .

# syntax=docker/dockerfile:1.7

FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

# Debian's apt `nodejs` package is stuck on 18.x, but @tailwindcss/oxide's
# native binary requires Node >= 20 (fails silently under 18 - EBADENGINE).
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
    apt-get install -y --no-install-recommends nodejs && \
    rm -rf /var/lib/apt/lists/*

# Prebuilt trunk binary instead of `cargo install trunk` (which compiles trunk
# itself from source, adding a couple of minutes to every image build).
ARG TARGETARCH
ARG TRUNK_VERSION=v0.21.14
RUN set -eux; \
    case "${TARGETARCH}" in \
      amd64) trunk_arch=x86_64-unknown-linux-gnu ;; \
      arm64) trunk_arch=aarch64-unknown-linux-gnu ;; \
      *) echo "unsupported arch: ${TARGETARCH}" >&2; exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-${trunk_arch}.tar.gz" \
      | tar -xz -C /usr/local/bin

# Builds just the dependency graph first (cached across builds as long as
# Cargo.toml/Cargo.lock don't change) so editing application source doesn't
# force a full recompile of every dependency.
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json -p frontend --target wasm32-unknown-unknown

ARG API_URL=http://127.0.0.1:8070/
RUN echo "API_URL = \"${API_URL}\"" > .env

COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci

COPY . .

# dist is intentionally outside /app/target: that dir is a cache mount below,
# so anything written under it disappears once the RUN step finishes.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    trunk build --release --config Trunk-release.toml

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
