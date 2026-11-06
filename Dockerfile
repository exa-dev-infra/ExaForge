# Build stage
FROM rust:1-bookworm as builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace sources, migrations, and Askama HTML templates
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations

# Build production binary for exaforge
RUN cargo build --release --bin exaforge

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 git curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/exaforge /app/server
COPY migrations ./migrations
COPY .env.example ./.env

EXPOSE 8080
CMD ["/app/server", "serve"]
