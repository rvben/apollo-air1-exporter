# Build stage
FROM rust:1.88 AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/apollo-air1-exporter /usr/local/bin/apollo-air1-exporter

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash exporter

USER exporter

# Expose metrics port
EXPOSE 9926

# Run the exporter
ENTRYPOINT ["apollo-air1-exporter"]