# Apollo Air-1 Prometheus Exporter

[![CI](https://github.com/rvben/apollo-air1-exporter/actions/workflows/ci.yml/badge.svg)](https://github.com/rvben/apollo-air1-exporter/actions/workflows/ci.yml)
[![Docker Pulls](https://img.shields.io/docker/pulls/rvben/apollo-air1-exporter)](https://hub.docker.com/r/rvben/apollo-air1-exporter)
[![Docker Image Size](https://img.shields.io/docker/image-size/rvben/apollo-air1-exporter)](https://hub.docker.com/r/rvben/apollo-air1-exporter)
[![License](https://img.shields.io/github/license/rvben/apollo-air1-exporter)](LICENSE)

A Prometheus exporter for Apollo Air-1 air quality monitors (ESPHome-based devices).

## Features

- Exports air quality metrics from Apollo Air-1 devices
- Supports multiple devices with configurable names
- Auto-discovery of available sensors
- Graceful handling of offline devices

## Metrics

The exporter provides the following metrics (when available on the device):

- `apollo_air1_device_up` - Device availability (1 = up, 0 = down)
- `apollo_air1_co2_ppm` - CO2 concentration in parts per million
- `apollo_air1_pm1_0_ugm3` - PM1.0 particulate matter in µg/m³
- `apollo_air1_pm2_5_ugm3` - PM2.5 particulate matter in µg/m³
- `apollo_air1_pm10_0_ugm3` - PM10 particulate matter in µg/m³
- `apollo_air1_voc_index` - Volatile Organic Compounds index
- `apollo_air1_nox_index` - Nitrogen Oxides index
- `apollo_air1_temperature_celsius` - Temperature in degrees Celsius
- `apollo_air1_humidity_percent` - Relative humidity percentage
- `apollo_air1_pressure_hpa` - Atmospheric pressure in hectopascals
- `apollo_air1_illuminance_lux` - Light level in lux
- `apollo_air1_esp_temperature_celsius` - ESP32 internal temperature
- `apollo_air1_wifi_rssi_dbm` - WiFi signal strength in dBm

All metrics include `device` and `host` labels for identification.

## Configuration

The exporter is configured via environment variables:

- `APOLLO_HOSTS` (required) - Comma-separated list of device URLs (e.g., `http://192.168.1.100,http://192.168.1.101`)
- `APOLLO_NAMES` (optional) - Comma-separated list of device names (same order as hosts)
- `APOLLO_EXPORTER_PORT` (default: 9926) - Port to expose metrics on
- `APOLLO_EXPORTER_BIND` (default: 0.0.0.0) - Bind address for metrics server
- `APOLLO_POLL_INTERVAL` (default: 30) - Poll interval in seconds
- `APOLLO_HTTP_TIMEOUT` (default: 10) - HTTP timeout in seconds
- `APOLLO_LOG_LEVEL` (default: info) - Log level (trace, debug, info, warn, error)

## Installation

### Docker (Recommended)

Pull the pre-built multi-architecture image from Docker Hub:

```bash
docker pull rvben/apollo-air1-exporter:latest
```

Run with Docker:

```bash
docker run -d \
  --name apollo-air1-exporter \
  -p 9926:9926 \
  -e APOLLO_HOSTS="http://192.168.1.100,http://192.168.1.101" \
  -e APOLLO_NAMES="Living Room,Bedroom" \
  rvben/apollo-air1-exporter:latest
```

### Building from source

```bash
git clone https://github.com/rvben/apollo-air1-exporter.git
cd apollo-air1-exporter
cargo build --release
```

### Running locally

```bash
# Single device
APOLLO_HOSTS="http://192.168.1.100" ./target/release/apollo-air1-exporter

# Multiple devices with names
APOLLO_HOSTS="http://192.168.1.100,http://192.168.1.101" \
APOLLO_NAMES="Living Room,Bedroom" \
./target/release/apollo-air1-exporter
```

### Docker Compose

```yaml
version: '3'
services:
  apollo-air1-exporter:
    build: .
    ports:
      - "9926:9926"
    environment:
      - APOLLO_HOSTS=http://192.168.1.100,http://192.168.1.101
      - APOLLO_NAMES=Living Room,Bedroom
      - APOLLO_POLL_INTERVAL=30
      - APOLLO_LOG_LEVEL=info
    restart: unless-stopped
```

## Endpoints

- `/metrics` - Prometheus metrics
- `/health` - Health check endpoint
- `/` - Welcome page

## Building from source

```bash
cargo build --release
```

## Prometheus Configuration

Add the following to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'apollo_air1'
    static_configs:
      - targets: ['localhost:9926']
```

## License

Same as the other exporters in this repository.