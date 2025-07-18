use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct ApolloClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SensorData {
    pub id: String,
    pub value: f64,
    pub state: String,
}

#[derive(Debug, Clone)]
pub struct ApolloStatus {
    pub sensors: HashMap<String, SensorValue>,
    pub device_name: String,
}

#[derive(Debug, Clone)]
pub struct SensorValue {
    pub value: f64,
    #[allow(dead_code)]
    pub unit: String,
    #[allow(dead_code)]
    pub name: String,
}

// Known Apollo Air-1 sensors - using ESPHome sensor names
const KNOWN_SENSORS: &[(&str, &str)] = &[
    ("co2", "CO2"),
    ("sen55_temperature", "Temperature"),
    ("sen55_humidity", "Humidity"),
    ("pm__1_m_weight_concentration", "PM1.0"),
    ("pm__2_5_m_weight_concentration", "PM2.5"),
    ("pm__10_m_weight_concentration", "PM10"),
    ("sen55_voc", "VOC"),
    ("sen55_nox", "NOx"),
    ("dps310_pressure", "Pressure"),
    ("illuminance", "Illuminance"),
    ("esp_temperature", "ESP Temperature"),
    ("rssi", "WiFi RSSI"),
];

impl ApolloClient {
    pub fn new(base_url: String, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, base_url })
    }

    pub async fn get_status(&self, device_name: &str) -> Result<ApolloStatus> {
        debug!("Fetching status from Apollo Air-1 at {}", self.base_url);

        let mut sensors = HashMap::new();

        // Try to fetch each known sensor
        for (sensor_id, sensor_name) in KNOWN_SENSORS {
            match self.get_sensor(sensor_id).await {
                Ok(data) => {
                    let unit = extract_unit(&data.state, data.value);
                    sensors.insert(
                        sensor_id.to_string(),
                        SensorValue {
                            value: data.value,
                            unit,
                            name: sensor_name.to_string(),
                        },
                    );
                    debug!("Got {}: {} {}", sensor_name, data.value, data.state);
                }
                Err(e) => {
                    debug!("Sensor {} not available: {}", sensor_id, e);
                }
            }
        }

        if sensors.is_empty() {
            return Err(anyhow!("No sensors found on device"));
        }

        info!("Retrieved {} sensors from {}", sensors.len(), device_name);

        Ok(ApolloStatus {
            sensors,
            device_name: device_name.to_string(),
        })
    }

    async fn get_sensor(&self, sensor_id: &str) -> Result<SensorData> {
        let url = format!("{}/sensor/{}", self.base_url, sensor_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch sensor {}: {}", sensor_id, e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch sensor {}: HTTP {}",
                sensor_id,
                response.status()
            ));
        }

        let data = response
            .json::<SensorData>()
            .await
            .map_err(|e| anyhow!("Failed to parse sensor {} data: {}", sensor_id, e))?;

        Ok(data)
    }

    pub async fn test_connection(&self) -> Result<bool> {
        // Try to fetch CO2 sensor as a connection test
        match self.get_sensor("co2").await {
            Ok(_) => Ok(true),
            Err(_) => {
                // Try ESP temperature as fallback
                match self.get_sensor("esp_temperature").await {
                    Ok(_) => Ok(true),
                    Err(_) => {
                        // Try uptime as last resort
                        match self.get_sensor("uptime").await {
                            Ok(_) => Ok(true),
                            Err(e) => {
                                warn!("Connection test failed: {}", e);
                                Ok(false)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Extract unit from state string
fn extract_unit(state: &str, value: f64) -> String {
    // Try to extract unit from state string
    // Format is usually "value unit" e.g. "25.5 °C"
    let value_str = format!("{value}");
    let value_str_formatted = format!("{value:.1}");

    if let Some(pos) = state.find(&value_str) {
        state[pos + value_str.len()..].trim().to_string()
    } else if let Some(pos) = state.find(&value_str_formatted) {
        state[pos + value_str_formatted.len()..].trim().to_string()
    } else {
        // Try to find common units
        if state.contains("°C") {
            "°C".to_string()
        } else if state.contains("°F") {
            "°F".to_string()
        } else if state.contains("%") {
            "%".to_string()
        } else if state.contains("ppm") {
            "ppm".to_string()
        } else if state.contains("µg/m³") {
            "µg/m³".to_string()
        } else if state.contains("hPa") {
            "hPa".to_string()
        } else if state.contains("lx") {
            "lx".to_string()
        } else if state.contains("dBm") {
            "dBm".to_string()
        } else if state.contains(" s") {
            "s".to_string()
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn test_get_sensor() {
        let mock_server = MockServer::start().await;

        let sensor_response = r#"{
            "id": "sensor-co2",
            "value": 450.0,
            "state": "450 ppm"
        }"#;

        Mock::given(method("GET"))
            .and(path("/sensor/co2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(sensor_response))
            .mount(&mock_server)
            .await;

        let client = ApolloClient::new(mock_server.uri(), Duration::from_secs(5)).unwrap();

        let data = client.get_sensor("co2").await.unwrap();
        assert_eq!(data.value, 450.0);
        assert_eq!(data.state, "450 ppm");
    }

    #[tokio::test]
    async fn test_get_status() {
        let mock_server = MockServer::start().await;

        // Mock CO2 sensor
        Mock::given(method("GET"))
            .and(path("/sensor/co2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{
                "id": "sensor-co2",
                "value": 520.0,
                "state": "520 ppm"
            }"#,
            ))
            .mount(&mock_server)
            .await;

        // Mock temperature sensor
        Mock::given(method("GET"))
            .and(path("/sensor/sen55_temperature"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{
                "id": "sensor-sen55_temperature",
                "value": 22.5,
                "state": "22.5 °C"
            }"#,
            ))
            .mount(&mock_server)
            .await;

        // Mock other sensors as not found
        for (sensor, _) in KNOWN_SENSORS.iter().skip(2) {
            Mock::given(method("GET"))
                .and(path(format!("/sensor/{}", sensor)))
                .respond_with(ResponseTemplate::new(404))
                .mount(&mock_server)
                .await;
        }

        let client = ApolloClient::new(mock_server.uri(), Duration::from_secs(5)).unwrap();

        let status = client.get_status("Test Device").await.unwrap();
        assert_eq!(status.device_name, "Test Device");
        assert_eq!(status.sensors.len(), 2);

        let co2 = status.sensors.get("co2").unwrap();
        assert_eq!(co2.value, 520.0);
        assert_eq!(co2.unit, "ppm");
        assert_eq!(co2.name, "CO2");

        let temp = status.sensors.get("sen55_temperature").unwrap();
        assert_eq!(temp.value, 22.5);
        assert_eq!(temp.unit, "°C");
        assert_eq!(temp.name, "Temperature");
    }

    #[test]
    fn test_extract_unit() {
        assert_eq!(extract_unit("450 ppm", 450.0), "ppm");
        assert_eq!(extract_unit("22.5 °C", 22.5), "°C");
        assert_eq!(extract_unit("65 %", 65.0), "%");
        assert_eq!(extract_unit("1013.25 hPa", 1013.25), "hPa");
        assert_eq!(extract_unit("-62 dBm", -62.0), "dBm");
        assert_eq!(extract_unit("2.5 µg/m³", 2.5), "µg/m³");
    }
}
