# Stage 1: Build Rust application
FROM rust:1.83.0 AS builder

WORKDIR /app

# Install required dependencies (if any)
RUN apt update && apt install -y pkg-config libssl-dev

# Copy and build Rust dependencies first to optimize caching
COPY Cargo.toml Cargo.lock ./
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Stage 2: Build frontend
FROM node:20 AS frontend-builder

WORKDIR /app/ui

# Copy frontend files
COPY ui/package.json ui/package-lock.json ./
RUN npm install

COPY ui ./
RUN npm run build

# Stage 3: Final runtime image
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt install -y openssl

WORKDIR /app

# Copy built Rust binary
COPY --from=builder /app/target/release/gametank-games ./gametank-games

# Copy frontend assets
COPY --from=frontend-builder /app/ui/dist ./target/ui

# Expose the port the app runs on (change if needed)
EXPOSE 41123

# Run the application
CMD ["./gametank-games"]
