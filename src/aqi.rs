/// Air Quality Index (AQI) calculation module
/// Based on US EPA standards for PM2.5 and PM10

#[derive(Debug, Clone, PartialEq)]
pub enum AqiCategory {
    Good,
    Moderate,
    UnhealthyForSensitiveGroups,
    Unhealthy,
    VeryUnhealthy,
    Hazardous,
}

impl AqiCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AqiCategory::Good => "Good",
            AqiCategory::Moderate => "Moderate",
            AqiCategory::UnhealthyForSensitiveGroups => "Unhealthy for Sensitive Groups",
            AqiCategory::Unhealthy => "Unhealthy",
            AqiCategory::VeryUnhealthy => "Very Unhealthy",
            AqiCategory::Hazardous => "Hazardous",
        }
    }

    fn from_aqi(aqi: f64) -> Self {
        match aqi as u16 {
            0..=50 => AqiCategory::Good,
            51..=100 => AqiCategory::Moderate,
            101..=150 => AqiCategory::UnhealthyForSensitiveGroups,
            151..=200 => AqiCategory::Unhealthy,
            201..=300 => AqiCategory::VeryUnhealthy,
            _ => AqiCategory::Hazardous,
        }
    }
}

#[derive(Debug)]
pub struct AqiResult {
    pub aqi: f64,
    pub category: AqiCategory,
    pub primary_pollutant: String,
}

/// PM2.5 breakpoints (24-hour average, µg/m³)
const PM25_BREAKPOINTS: [(f64, f64, u16, u16); 7] = [
    (0.0, 12.0, 0, 50),       // Good
    (12.1, 35.4, 51, 100),    // Moderate
    (35.5, 55.4, 101, 150),   // Unhealthy for Sensitive Groups
    (55.5, 150.4, 151, 200),  // Unhealthy
    (150.5, 250.4, 201, 300), // Very Unhealthy
    (250.5, 350.4, 301, 400), // Hazardous
    (350.5, 500.4, 401, 500), // Hazardous
];

/// PM10 breakpoints (24-hour average, µg/m³)
const PM10_BREAKPOINTS: [(f64, f64, u16, u16); 7] = [
    (0.0, 54.0, 0, 50),       // Good
    (55.0, 154.0, 51, 100),   // Moderate
    (155.0, 254.0, 101, 150), // Unhealthy for Sensitive Groups
    (255.0, 354.0, 151, 200), // Unhealthy
    (355.0, 424.0, 201, 300), // Very Unhealthy
    (425.0, 504.0, 301, 400), // Hazardous
    (505.0, 604.0, 401, 500), // Hazardous
];

/// Calculate AQI for a pollutant using EPA formula
/// AQI = [(IHi - ILo)/(BPHi - BPLo)] × (Cp - BPLo) + ILo
fn calculate_pollutant_aqi(
    concentration: f64,
    breakpoints: &[(f64, f64, u16, u16)],
) -> Option<f64> {
    for &(bp_lo, bp_hi, i_lo, i_hi) in breakpoints {
        if concentration >= bp_lo && concentration <= bp_hi {
            let aqi = ((i_hi as f64 - i_lo as f64) / (bp_hi - bp_lo)) * (concentration - bp_lo)
                + i_lo as f64;
            return Some(aqi.round());
        }
    }

    // If concentration exceeds all breakpoints, return max AQI
    if concentration > breakpoints.last().unwrap().1 {
        return Some(500.0);
    }

    None
}

/// Calculate overall AQI from PM2.5 and PM10 concentrations
pub fn calculate_aqi(pm25_ugm3: Option<f64>, pm10_ugm3: Option<f64>) -> Option<AqiResult> {
    let mut max_aqi = 0.0;
    let mut primary_pollutant = String::new();

    // Calculate PM2.5 AQI
    if let Some(pm25) = pm25_ugm3 {
        if let Some(pm25_aqi) = calculate_pollutant_aqi(pm25, &PM25_BREAKPOINTS) {
            if pm25_aqi > max_aqi {
                max_aqi = pm25_aqi;
                primary_pollutant = "PM2.5".to_string();
            }
        }
    }

    // Calculate PM10 AQI
    if let Some(pm10) = pm10_ugm3 {
        if let Some(pm10_aqi) = calculate_pollutant_aqi(pm10, &PM10_BREAKPOINTS) {
            if pm10_aqi > max_aqi {
                max_aqi = pm10_aqi;
                primary_pollutant = "PM10".to_string();
            }
        }
    }

    // Return None if no valid pollutant data
    if primary_pollutant.is_empty() {
        return None;
    }

    Some(AqiResult {
        aqi: max_aqi,
        category: AqiCategory::from_aqi(max_aqi),
        primary_pollutant,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pm25_aqi_calculation() {
        // Good range
        assert_eq!(calculate_pollutant_aqi(5.0, &PM25_BREAKPOINTS), Some(21.0));
        assert_eq!(calculate_pollutant_aqi(12.0, &PM25_BREAKPOINTS), Some(50.0));

        // Moderate range
        assert_eq!(calculate_pollutant_aqi(20.0, &PM25_BREAKPOINTS), Some(68.0));
        assert_eq!(
            calculate_pollutant_aqi(35.4, &PM25_BREAKPOINTS),
            Some(100.0)
        );
    }

    #[test]
    fn test_pm10_aqi_calculation() {
        // Good range
        assert_eq!(calculate_pollutant_aqi(25.0, &PM10_BREAKPOINTS), Some(23.0));
        assert_eq!(calculate_pollutant_aqi(54.0, &PM10_BREAKPOINTS), Some(50.0));

        // Moderate range
        assert_eq!(
            calculate_pollutant_aqi(100.0, &PM10_BREAKPOINTS),
            Some(73.0)
        );
    }

    #[test]
    fn test_overall_aqi_calculation() {
        // PM2.5 higher than PM10
        let result = calculate_aqi(Some(20.0), Some(30.0)).unwrap();
        assert_eq!(result.aqi, 68.0);
        assert_eq!(result.category, AqiCategory::Moderate);
        assert_eq!(result.primary_pollutant, "PM2.5");

        // PM10 higher than PM2.5
        let result = calculate_aqi(Some(5.0), Some(100.0)).unwrap();
        assert_eq!(result.aqi, 73.0);
        assert_eq!(result.category, AqiCategory::Moderate);
        assert_eq!(result.primary_pollutant, "PM10");

        // Only PM2.5 available
        let result = calculate_aqi(Some(15.0), None).unwrap();
        assert_eq!(result.aqi, 57.0);
        assert_eq!(result.primary_pollutant, "PM2.5");

        // No data available
        assert!(calculate_aqi(None, None).is_none());
    }

    #[test]
    fn test_aqi_categories() {
        assert_eq!(AqiCategory::from_aqi(25.0), AqiCategory::Good);
        assert_eq!(AqiCategory::from_aqi(75.0), AqiCategory::Moderate);
        assert_eq!(
            AqiCategory::from_aqi(125.0),
            AqiCategory::UnhealthyForSensitiveGroups
        );
        assert_eq!(AqiCategory::from_aqi(175.0), AqiCategory::Unhealthy);
        assert_eq!(AqiCategory::from_aqi(250.0), AqiCategory::VeryUnhealthy);
        assert_eq!(AqiCategory::from_aqi(450.0), AqiCategory::Hazardous);
    }

    #[test]
    fn test_category_strings() {
        assert_eq!(AqiCategory::Good.as_str(), "Good");
        assert_eq!(AqiCategory::Moderate.as_str(), "Moderate");
        assert_eq!(
            AqiCategory::UnhealthyForSensitiveGroups.as_str(),
            "Unhealthy for Sensitive Groups"
        );
    }
}
