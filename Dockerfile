# ==============================
# Builder Stage
# ==============================
FROM rust:1.80-bullseye as builder

WORKDIR /usr/src/app

# Install build dependencies
# libpq-dev is required by diesel for PostgreSQL connection
RUN apt-get update && apt-get install -y \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY . .

# Build the API binary
# Note: This builds the 'api' binary from the 'uchat_server' package
RUN cargo build -p uchat_server --bin api --release

# ==============================
# Runtime Stage
# ==============================
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
# libpq5 is required by diesel at runtime
# ca-certificates is required if the backend makes external HTTPS requests
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/api /usr/local/bin/api

# Create user content directories if required by the application
RUN mkdir -p /app/usercontent/images

# Expose the port that the API server listens on
# Render commonly sets a PORT environment variable, but 8070 is the app's default
EXPOSE 8070

# Run the API server
CMD ["api"]
