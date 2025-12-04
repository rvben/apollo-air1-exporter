# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.10] - 2025-12-04

### Fixed
- Update PM2.5 AQI breakpoints to EPA 2024 specification (effective May 6, 2024)
  - Good: 0-9.0 µg/m³ (was 0-12.0)
  - Unhealthy: 55.5-125.4 µg/m³ (was 55.5-150.4)
  - Very Unhealthy: 125.5-225.4 µg/m³ (was 150.5-250.4)
  - Hazardous: 225.5-325.4 µg/m³ (was 250.5-500.4)
- AQI metrics not returning to baseline when air quality improves (#16)
- CI workflow using wrong make target

### Added
- Per-pollutant sub-AQI metrics (apollo_air1_aqi_pm25, apollo_air1_aqi_pm10)
- AQI info metric with category labels (apollo_air1_aqi_info)
- Concentration truncation per EPA spec (PM2.5 to 1 decimal, PM10 to integer)
- Proper cleanup of stale AQI category labels when air quality changes

### Changed
- Restructured AQI metrics for proper Prometheus semantics
- apollo_air1_aqi now only has device/host labels (category moved to info metric)

## [0.0.7] - 2025-01-23

### Added
- Dependabot configuration for automated dependency updates
- Enhanced Cargo.toml metadata for better crates.io discoverability

### Fixed
- Musl toolchain installation in release workflow for binary builds
- GitHub release creation with proper binary artifacts

## [0.0.6] - 2025-01-22

### Added
- OCI labels to Dockerfile for GitHub Container Registry integration
- Make release target for automated release process
- Multi-platform Docker builds (linux/amd64, linux/arm64, linux/arm/v7)

### Changed
- Standardized user naming in Docker container to 'exporter'
- Updated Docker build to use musl-based Alpine for better portability
- Improved release workflow to commit Cargo.lock automatically

### Fixed
- Cargo.lock commit issues in release pipeline
- Docker architecture mismatch in builds

## [0.0.5] - 2025-01-15

### Added
- Initial Prometheus exporter for Apollo AIR-1 air quality monitors
- AQI (Air Quality Index) calculations based on EPA standards
- Support for multiple device monitoring
- Health check endpoint
- Docker support with multi-stage builds
- GitHub Actions CI/CD pipeline

### Features
- Real-time air quality monitoring
- PM2.5, PM10, and overall AQI metrics
- Device status tracking
- Configurable polling intervals
- TLS-enabled HTTP client