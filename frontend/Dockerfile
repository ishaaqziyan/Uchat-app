# build with:
#   docker build -f frontend/Dockerfile -t uchat-frontend --build-arg API_URL=https://api.example.com/ .

FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked

WORKDIR /app

ARG API_URL=http://127.0.0.1:8070/
RUN echo "API_URL = \"${API_URL}\"" > .env

COPY . .

RUN npm install
RUN trunk build --release --config Trunk-release.toml

FROM nginx:alpine

COPY --from=builder /app/target/dist /usr/share/nginx/html
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
