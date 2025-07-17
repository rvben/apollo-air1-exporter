.PHONY: build test lint fmt clean docker-build docker-push run gh-secrets help

# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test

# Run linting
lint:
	cargo clippy -- -D warnings

# Format code
fmt:
	cargo fmt

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Build Docker image
docker-build:
	docker build -t apollo-air1-exporter .

# Build multi-arch Docker image
docker-buildx:
	docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 -t apollo-air1-exporter .

# Run locally
run:
	APOLLO_HOSTS="http://10.10.20.13,http://10.10.20.58" \
	APOLLO_NAMES="Living Room,Master Bedroom" \
	cargo run

# Run with debug logging
run-debug:
	RUST_LOG=debug \
	APOLLO_HOSTS="http://10.10.20.13,http://10.10.20.58" \
	APOLLO_NAMES="Living Room,Master Bedroom" \
	cargo run

# Check code before committing
check: fmt lint test

# Install development dependencies
dev-setup:
	rustup component add rustfmt clippy

# Deploy secrets to GitHub
gh-secrets:
	@if [ ! -f .env ]; then \
		echo "Error: .env file not found. Copy .env.example to .env and fill in your values."; \
		exit 1; \
	fi
	@echo "Loading environment variables from .env..."
	@export $$(grep -v '^#' .env | xargs) && \
		gh secret set DOCKER_USERNAME --body "$$DOCKER_USERNAME" && \
		echo "✓ Set DOCKER_USERNAME" && \
		gh secret set DOCKER_PASSWORD --body "$$DOCKER_PASSWORD" && \
		echo "✓ Set DOCKER_PASSWORD" && \
		if [ ! -z "$$CRATES_IO_TOKEN" ]; then \
			gh secret set CRATES_IO_TOKEN --body "$$CRATES_IO_TOKEN" && \
			echo "✓ Set CRATES_IO_TOKEN"; \
		fi
	@echo "GitHub secrets deployed successfully!"

# Show available commands
help:
	@echo "Available commands:"
	@echo "  make build          - Build the project"
	@echo "  make test           - Run tests"
	@echo "  make lint           - Run linting"
	@echo "  make fmt            - Format code"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make docker-build   - Build Docker image"
	@echo "  make docker-buildx  - Build multi-arch Docker image"
	@echo "  make run            - Run locally"
	@echo "  make run-debug      - Run with debug logging"
	@echo "  make check          - Run fmt, lint, and test"
	@echo "  make dev-setup      - Install development dependencies"
	@echo "  make gh-secrets     - Deploy secrets to GitHub"