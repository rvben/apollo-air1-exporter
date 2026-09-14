.PHONY: build test lint fmt clean docker-build docker-buildx docker-push docker-push-ghcr run gh-secrets release release-patch release-minor release-major help

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

# Static Linux binary for the host CPU, so the image runs on the local Docker engine
MUSL_TARGET ?= $(shell uname -m | sed 's/^arm64$$/aarch64/')-unknown-linux-musl
DOCKER_CONTEXT := target/docker-context

# On macOS, cross-link with the musl-cross toolchain (brew install musl-cross)
ifeq ($(shell uname -s),Darwin)
MUSL_CC := $(subst -unknown-linux-musl,-linux-musl-gcc,$(MUSL_TARGET))
build-musl: export CC_$(subst -,_,$(MUSL_TARGET)) = $(MUSL_CC)
build-musl: export CARGO_TARGET_$(shell echo '$(MUSL_TARGET)' | tr 'a-z-' 'A-Z_')_LINKER = $(MUSL_CC)
endif

.PHONY: build-musl
# Build the static release binary the runtime image copies in
build-musl:
	rustup target add $(MUSL_TARGET)
	cargo build --release --target $(MUSL_TARGET)

# Build and smoke-test the Docker image. The Dockerfile only copies in a pre-built
# binary, so the build context is a directory holding just that binary, as in CI.
docker-build: build-musl
	rm -rf $(DOCKER_CONTEXT)
	mkdir -p $(DOCKER_CONTEXT)
	cp target/$(MUSL_TARGET)/release/apollo-air1-exporter $(DOCKER_CONTEXT)/
	docker build -f Dockerfile -t apollo-air1-exporter $(DOCKER_CONTEXT)
	docker run --rm apollo-air1-exporter --help >/dev/null

# Build multi-arch Docker image (local)
docker-buildx:
	docker buildx build --platform linux/amd64,linux/arm64 -t apollo-air1-exporter .

# Build and push multi-arch Docker image to Docker Hub
docker-push:
	@if [ -z "$$DOCKER_USERNAME" ]; then \
		echo "Error: DOCKER_USERNAME environment variable is required"; \
		echo "Usage: DOCKER_USERNAME=youruser DOCKER_PASSWORD=yourpass make docker-push"; \
		exit 1; \
	fi
	@if [ -z "$$DOCKER_PASSWORD" ]; then \
		echo "Error: DOCKER_PASSWORD environment variable is required"; \
		echo "Usage: DOCKER_USERNAME=youruser DOCKER_PASSWORD=yourpass make docker-push"; \
		exit 1; \
	fi
	@echo "Logging in to Docker Hub..."
	@echo "$$DOCKER_PASSWORD" | docker login -u "$$DOCKER_USERNAME" --password-stdin
	@echo "Building and pushing multi-arch images..."
	docker buildx build --platform linux/amd64,linux/arm64 \
		-t $$DOCKER_USERNAME/apollo-air1-exporter:latest \
		-t $$DOCKER_USERNAME/apollo-air1-exporter:$$(git describe --tags --always) \
		--push .
	@echo "Successfully pushed to Docker Hub!"

# Build and push to GitHub Container Registry
docker-push-ghcr:
	@if [ -z "$$GITHUB_TOKEN" ]; then \
		echo "Error: GITHUB_TOKEN environment variable is required"; \
		exit 1; \
	fi
	@echo "Logging in to GitHub Container Registry..."
	@echo "$$GITHUB_TOKEN" | docker login ghcr.io -u $$GITHUB_ACTOR --password-stdin
	@echo "Building and pushing multi-arch images to GHCR..."
	docker buildx build --platform linux/amd64,linux/arm64 \
		-t ghcr.io/$$GITHUB_REPOSITORY_OWNER/apollo-air1-exporter:latest \
		-t ghcr.io/$$GITHUB_REPOSITORY_OWNER/apollo-air1-exporter:$$(git describe --tags --always) \
		--push .
	@echo "Successfully pushed to GitHub Container Registry!"

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

# Prepare a release
release:
	@if [ -z "$$VERSION" ]; then \
		echo "Error: VERSION environment variable is required"; \
		echo "Usage: VERSION=v0.1.0 make release"; \
		exit 1; \
	fi
	@echo "Preparing release $$VERSION..."
	@echo "1. Running checks..."
	@make check
	@echo "2. Updating version in Cargo.toml..."
	@VERSION_NUM=$$(echo $$VERSION | sed 's/^v//') && \
		sed -i.bak "s/^version = \".*\"/version = \"$$VERSION_NUM\"/" Cargo.toml && \
		rm Cargo.toml.bak
	@echo "3. Updating Cargo.lock..."
	@cargo update
	@echo "4. Committing version changes..."
	@git add Cargo.toml Cargo.lock
	@git commit -m "chore: bump version to $$VERSION" || echo "No changes to commit"
	@echo "5. Creating and pushing tag..."
	@git tag $$VERSION
	@git push origin $$VERSION
	@echo "Release $$VERSION created and pushed!"

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
		echo "✓ Set DOCKER_PASSWORD"
	@echo "GitHub secrets deployed successfully!"

# Show available commands
help:
	@echo "Available commands:"
	@echo "  make build          - Build the project"
	@echo "  make test           - Run tests"
	@echo "  make lint           - Run linting"
	@echo "  make fmt            - Format code"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make docker-build   - Build Docker image (local)"
	@echo "  make docker-buildx  - Build multi-arch Docker image (local)"
	@echo "  make docker-push    - Build and push multi-arch to Docker Hub"
	@echo "  make docker-push-ghcr - Build and push multi-arch to GitHub Container Registry"
	@echo "  make run            - Run locally"
	@echo "  make run-debug      - Run with debug logging"
	@echo "  make check          - Run fmt, lint, and test"
	@echo "  make release        - Prepare and create a release (requires VERSION=v0.1.0)"
	@echo "  make dev-setup      - Install development dependencies"
	@echo "  make gh-secrets     - Deploy secrets to GitHub"

release-patch:
	vership bump patch

release-minor:
	vership bump minor

release-major:
	vership bump major
