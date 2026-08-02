# build with:
#   docker build -f backend/Dockerfile -t uchat-backend --build-arg API_URL=https://api.example.com/ .

FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ARG API_URL=http://127.0.0.1:8070/
RUN echo "API_URL = \"${API_URL}\"" > .env

COPY . .

RUN cargo build --release -p uchat_server --bin api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/api ./api

ENV API_BIND=0.0.0.0:8070
EXPOSE 8070

ENTRYPOINT ["./api"]
