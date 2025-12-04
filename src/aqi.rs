/// Air Quality Index (AQI) calculation module
///
/// Based on US EPA standards for PM2.5 and PM10.
/// PM2.5 breakpoints updated to 2024 EPA revision (effective May 6, 2024).
///
/// References:
/// - EPA AQI Breakpoints: https://aqs.epa.gov/aqsweb/documents/codetables/aqi_breakpoints.html
/// - Federal Register Final Rule: https://www.federalregister.gov/documents/2024/03/06/2024-02637/

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
    /// Overall AQI (max of all pollutants)
    pub aqi: f64,
    /// Category based on overall AQI
    pub category: AqiCategory,
    /// Pollutant with highest AQI
    pub primary_pollutant: String,
    /// Individual PM2.5 sub-AQI (if available)
    pub pm25_aqi: Option<f64>,
    /// Individual PM10 sub-AQI (if available)
    pub pm10_aqi: Option<f64>,
}

/// PM2.5 breakpoints (24-hour average, µg/m³)
/// Updated to 2024 EPA revision (effective May 6, 2024)
/// Source: https://aqs.epa.gov/aqsweb/documents/codetables/aqi_breakpoints.html
const PM25_BREAKPOINTS: [(f64, f64, u16, u16); 7] = [
    (0.0, 9.0, 0, 50),        // Good
    (9.1, 35.4, 51, 100),     // Moderate
    (35.5, 55.4, 101, 150),   // Unhealthy for Sensitive Groups
    (55.5, 125.4, 151, 200),  // Unhealthy
    (125.5, 225.4, 201, 300), // Very Unhealthy
    (225.5, 325.4, 301, 500), // Hazardous
    (325.5, 999.9, 501, 999), // Beyond AQI scale
];

/// PM10 breakpoints (24-hour average, µg/m³)
/// Source: https://aqs.epa.gov/aqsweb/documents/codetables/aqi_breakpoints.html
const PM10_BREAKPOINTS: [(f64, f64, u16, u16); 7] = [
    (0.0, 54.0, 0, 50),       // Good
    (55.0, 154.0, 51, 100),   // Moderate
    (155.0, 254.0, 101, 150), // Unhealthy for Sensitive Groups
    (255.0, 354.0, 151, 200), // Unhealthy
    (355.0, 424.0, 201, 300), // Very Unhealthy
    (425.0, 604.0, 301, 500), // Hazardous
    (605.0, 999.0, 501, 999), // Beyond AQI scale
];

/// Truncate PM2.5 concentration to 1 decimal place per EPA specification
fn truncate_pm25(value: f64) -> f64 {
    (value * 10.0).floor() / 10.0
}

/// Truncate PM10 concentration to integer per EPA specification
fn truncate_pm10(value: f64) -> f64 {
    value.floor()
}

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
///
/// Concentrations are truncated per EPA specification before calculation:
/// - PM2.5: truncated to 1 decimal place
/// - PM10: truncated to integer
pub fn calculate_aqi(pm25_ugm3: Option<f64>, pm10_ugm3: Option<f64>) -> Option<AqiResult> {
    let mut max_aqi = 0.0;
    let mut primary_pollutant = String::new();

    // Calculate PM2.5 AQI (truncate to 1 decimal per EPA spec)
    let pm25_aqi =
        pm25_ugm3.and_then(|pm25| calculate_pollutant_aqi(truncate_pm25(pm25), &PM25_BREAKPOINTS));
    if let Some(aqi) = pm25_aqi
        && aqi > max_aqi
    {
        max_aqi = aqi;
        primary_pollutant = "PM2.5".to_string();
    }

    // Calculate PM10 AQI (truncate to integer per EPA spec)
    let pm10_aqi =
        pm10_ugm3.and_then(|pm10| calculate_pollutant_aqi(truncate_pm10(pm10), &PM10_BREAKPOINTS));
    if let Some(aqi) = pm10_aqi
        && aqi > max_aqi
    {
        max_aqi = aqi;
        primary_pollutant = "PM10".to_string();
    }

    // Return None if no valid pollutant data
    if primary_pollutant.is_empty() {
        return None;
    }

    Some(AqiResult {
        aqi: max_aqi,
        category: AqiCategory::from_aqi(max_aqi),
        primary_pollutant,
        pm25_aqi,
        pm10_aqi,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pm25_aqi_calculation() {
        // Good range (0.0-9.0 µg/m³ → AQI 0-50) - 2024 EPA breakpoints
        assert_eq!(calculate_pollutant_aqi(5.0, &PM25_BREAKPOINTS), Some(28.0));
        assert_eq!(calculate_pollutant_aqi(9.0, &PM25_BREAKPOINTS), Some(50.0));

        // Moderate range (9.1-35.4 µg/m³ → AQI 51-100)
        assert_eq!(calculate_pollutant_aqi(12.0, &PM25_BREAKPOINTS), Some(56.0));
        assert_eq!(calculate_pollutant_aqi(20.0, &PM25_BREAKPOINTS), Some(71.0));
        assert_eq!(
            calculate_pollutant_aqi(35.4, &PM25_BREAKPOINTS),
            Some(100.0)
        );
    }

    #[test]
    fn test_truncation() {
        // PM2.5 truncation to 1 decimal
        assert_eq!(truncate_pm25(12.34), 12.3);
        assert_eq!(truncate_pm25(12.39), 12.3);
        assert_eq!(truncate_pm25(12.0), 12.0);

        // PM10 truncation to integer
        assert_eq!(truncate_pm10(54.9), 54.0);
        assert_eq!(truncate_pm10(55.0), 55.0);
        assert_eq!(truncate_pm10(100.7), 100.0);
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
        // PM2.5 higher than PM10 (2024 breakpoints)
        let result = calculate_aqi(Some(20.0), Some(30.0)).unwrap();
        assert_eq!(result.aqi, 71.0);
        assert_eq!(result.category, AqiCategory::Moderate);
        assert_eq!(result.primary_pollutant, "PM2.5");
        assert_eq!(result.pm25_aqi, Some(71.0));
        assert_eq!(result.pm10_aqi, Some(28.0));

        // PM10 higher than PM2.5
        let result = calculate_aqi(Some(5.0), Some(100.0)).unwrap();
        assert_eq!(result.aqi, 73.0);
        assert_eq!(result.category, AqiCategory::Moderate);
        assert_eq!(result.primary_pollutant, "PM10");
        assert_eq!(result.pm25_aqi, Some(28.0));
        assert_eq!(result.pm10_aqi, Some(73.0));

        // Only PM2.5 available
        let result = calculate_aqi(Some(15.0), None).unwrap();
        assert_eq!(result.aqi, 62.0);
        assert_eq!(result.primary_pollutant, "PM2.5");
        assert_eq!(result.pm25_aqi, Some(62.0));
        assert_eq!(result.pm10_aqi, None);

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
