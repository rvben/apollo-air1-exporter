# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] - 2025-01-17

### Added
- Initial release of Apollo Air-1 Prometheus Exporter
- Support for multiple Apollo Air-1 devices
- Auto-discovery of available sensors
- Metrics for CO2, temperature, humidity, and other air quality indicators
- Docker support with multi-architecture builds (amd64, arm64, armv7)
- Configurable polling interval and timeout
- Health check endpoint
- Comprehensive logging with configurable levels

### Features
- Graceful handling of offline devices
- Custom device naming support
- Environment variable configuration
- Prometheus-compatible metrics endpoint

[Unreleased]: https://github.com/rvben/apollo-air1-exporter/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/rvben/apollo-air1-exporter/releases/tag/v0.0.1