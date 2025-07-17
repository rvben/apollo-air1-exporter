use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Comma-separated list of Apollo Air-1 device URLs (e.g., http://192.168.1.100,http://192.168.1.101)
    #[arg(long, env = "APOLLO_HOSTS", value_delimiter = ',', required = true)]
    pub hosts: Vec<String>,

    /// Optional comma-separated list of device names (same order as hosts)
    #[arg(long, env = "APOLLO_NAMES", value_delimiter = ',')]
    pub names: Option<Vec<String>>,

    /// Port to expose metrics on
    #[arg(short, long, env = "APOLLO_EXPORTER_PORT", default_value = "9926")]
    pub port: u16,

    /// Bind address for metrics server
    #[arg(long, env = "APOLLO_EXPORTER_BIND", default_value = "0.0.0.0")]
    pub bind: String,

    /// Poll interval in seconds
    #[arg(long, env = "APOLLO_POLL_INTERVAL", default_value = "30")]
    pub poll_interval: u64,

    /// HTTP timeout in seconds
    #[arg(long, env = "APOLLO_HTTP_TIMEOUT", default_value = "10")]
    pub http_timeout: u64,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "APOLLO_LOG_LEVEL", default_value = "info")]
    pub log_level: String,
}

impl Config {
    pub fn metrics_bind_address(&self) -> String {
        format!("{}:{}", self.bind, self.port)
    }

    pub fn poll_interval_duration(&self) -> Duration {
        Duration::from_secs(self.poll_interval)
    }

    pub fn http_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.http_timeout)
    }

    pub fn get_device_names(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();

        for (idx, host) in self.hosts.iter().enumerate() {
            let name = if let Some(names) = &self.names {
                names.get(idx).cloned().unwrap_or_else(|| {
                    // Extract IP or hostname from URL
                    extract_device_name(host)
                })
            } else {
                // Extract IP or hostname from URL
                extract_device_name(host)
            };

            result.push((host.clone(), name));
        }

        result
    }
}

fn extract_device_name(url: &str) -> String {
    url.trim_start_matches("http://")
        .trim_start_matches("https://")
        .split(':')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_bind_address() {
        let config = Config {
            hosts: vec!["http://192.168.1.100".to_string()],
            names: None,
            port: 9926,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
        };

        assert_eq!(config.metrics_bind_address(), "0.0.0.0:9926");
    }

    #[test]
    fn test_durations() {
        let config = Config {
            hosts: vec!["http://192.168.1.100".to_string()],
            names: None,
            port: 9926,
            bind: "0.0.0.0".to_string(),
            poll_interval: 45,
            http_timeout: 15,
            log_level: "info".to_string(),
        };

        assert_eq!(config.poll_interval_duration(), Duration::from_secs(45));
        assert_eq!(config.http_timeout_duration(), Duration::from_secs(15));
    }

    #[test]
    fn test_get_device_names() {
        let config_with_names = Config {
            hosts: vec![
                "http://192.168.1.100".to_string(),
                "http://192.168.1.101:8080".to_string(),
            ],
            names: Some(vec!["Living Room".to_string(), "Bedroom".to_string()]),
            port: 9926,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
        };

        let names = config_with_names.get_device_names();
        assert_eq!(names.len(), 2);
        assert_eq!(
            names[0],
            (
                "http://192.168.1.100".to_string(),
                "Living Room".to_string()
            )
        );
        assert_eq!(
            names[1],
            (
                "http://192.168.1.101:8080".to_string(),
                "Bedroom".to_string()
            )
        );

        let config_without_names = Config {
            hosts: vec![
                "http://192.168.1.100".to_string(),
                "https://apollo.local".to_string(),
            ],
            names: None,
            port: 9926,
            bind: "0.0.0.0".to_string(),
            poll_interval: 30,
            http_timeout: 10,
            log_level: "info".to_string(),
        };

        let names = config_without_names.get_device_names();
        assert_eq!(names.len(), 2);
        assert_eq!(
            names[0],
            (
                "http://192.168.1.100".to_string(),
                "192.168.1.100".to_string()
            )
        );
        assert_eq!(
            names[1],
            (
                "https://apollo.local".to_string(),
                "apollo.local".to_string()
            )
        );
    }

    #[test]
    fn test_extract_device_name() {
        assert_eq!(extract_device_name("http://192.168.1.100"), "192.168.1.100");
        assert_eq!(
            extract_device_name("http://192.168.1.100:8080"),
            "192.168.1.100"
        );
        assert_eq!(extract_device_name("https://apollo.local"), "apollo.local");
        assert_eq!(extract_device_name("apollo.local"), "apollo.local");
    }
}
