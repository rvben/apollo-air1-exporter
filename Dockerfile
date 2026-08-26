# Minimal runtime-only Dockerfile
# Binary is provided as the Docker build context
#
# Local example:
#   cargo build --release
#   docker build -f Dockerfile -t apollo-air1-exporter target/release/

FROM alpine:3.24

# OCI labels for GitHub Container Registry
LABEL org.opencontainers.image.source=https://github.com/rvben/apollo-air1-exporter
LABEL org.opencontainers.image.description="Prometheus exporter for Apollo AIR-1 air quality monitors"
LABEL org.opencontainers.image.licenses=MIT

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -g 1000 exporter && \
    adduser -D -u 1000 -G exporter exporter

# Copy pre-built binary
COPY apollo-air1-exporter /usr/local/bin/apollo-air1-exporter

# Set permissions
RUN chmod +x /usr/local/bin/apollo-air1-exporter && \
    chown exporter:exporter /usr/local/bin/apollo-air1-exporter

# Switch to non-root user
USER exporter

# Expose metrics port
EXPOSE 9926

# Set default environment variables
ENV LOG_LEVEL=info
ENV POLL_INTERVAL=60
ENV METRICS_PORT=9926

# Run the exporter
ENTRYPOINT ["/usr/local/bin/apollo-air1-exporter"]
