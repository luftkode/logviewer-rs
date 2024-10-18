use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Display, PartialEq, Deserialize, Serialize)]
#[display("HE{id} {timestamp}: {altitude_m}")]
pub struct AltimeterEntry {
    pub id: u8,
    timestamp: DateTime<Utc>,
    altitude_m: f64,
}

impl AltimeterEntry {
    pub(crate) fn timestamp_ns(&self) -> f64 {
        self.timestamp
            .timestamp_nanos_opt()
            .expect("timestamp as nanoseconds out of range") as f64
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum AltimeterParseError {
    #[error("Invalid format")]
    InvalidFormat,
    #[error("Invalid ID")]
    InvalidId,
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(#[from] chrono::ParseError),
    #[error("Invalid altitude")]
    InvalidAltitude,
}

impl FromStr for AltimeterEntry {
    type Err = AltimeterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 9 {
            return Err(AltimeterParseError::InvalidFormat);
        }

        // Parse ID (format: "HE1" -> 1)
        let id = parts[0]
            .strip_prefix("HE")
            .and_then(|id| id.parse().ok())
            .ok_or(AltimeterParseError::InvalidId)?;

        // Parse timestamp
        let timestamp_str = format!(
            "{} {} {} {} {} {} {}",
            parts[1], parts[2], parts[3], parts[4], parts[5], parts[6], parts[7]
        );
        let naive_dt = NaiveDateTime::parse_from_str(&timestamp_str, "%Y %m %d %H %M %S %3f")?;
        let timestamp = DateTime::from_naive_utc_and_offset(naive_dt, Utc);

        // Parse altitude
        let altitude = parts[8]
            .parse()
            .map_err(|_| AltimeterParseError::InvalidAltitude)?;

        Ok(AltimeterEntry {
            id,
            timestamp,
            altitude_m: altitude,
        })
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    const TEST_ENTRY_HE1: &str = "HE1 2024 10 03 12 52 42 448 99999.99";
    const TEST_ENTRY_HE2: &str = "HE2 2024 10 03 12 52 42 557 99999.99";
    const TEST_TWO_LINES_BOTH: &str = "HE1 2024 10 03 12 52 42 448 99999.99
HE2 2024 10 03 12 52 42 557 99999.99
";

    #[test]
    fn test_parse_navsys_he_entry() -> TestResult {
        // Test parsing HE1
        let he1 = AltimeterEntry::from_str(TEST_ENTRY_HE1)?;
        assert_eq!(he1.id, 1);
        assert_eq!(he1.altitude_m, 99999.99);
        assert_eq!(
            he1.timestamp.naive_utc(),
            NaiveDateTime::parse_from_str("2024-10-03 12:52:42.448", "%Y-%m-%d %H:%M:%S.%3f")?
        );

        // Test parsing HE2
        let he2 = AltimeterEntry::from_str(TEST_ENTRY_HE2)?;
        assert_eq!(he2.id, 2);
        assert_eq!(he2.altitude_m, 99999.99);
        assert_eq!(
            he2.timestamp.naive_utc(),
            NaiveDateTime::parse_from_str("2024-10-03 12:52:42.557", "%Y-%m-%d %H:%M:%S.%3f")?
        );

        Ok(())
    }

    #[test]
    fn test_misc() -> TestResult {
        // Test parsing multiple lines fails appropriately
        assert!(AltimeterEntry::from_str(TEST_TWO_LINES_BOTH).is_err());

        // Test error cases
        assert!(matches!(
            AltimeterEntry::from_str("invalid").unwrap_err(),
            AltimeterParseError::InvalidFormat
        ));
        assert!(matches!(
            AltimeterEntry::from_str("HEA 2024 10 03 12 52 42 448 99999.99").unwrap_err(),
            AltimeterParseError::InvalidId
        ));
        assert!(matches!(
            AltimeterEntry::from_str("HE1 2024 10 03 12 52 42 448 invalid").unwrap_err(),
            AltimeterParseError::InvalidAltitude
        ));
        assert!(matches!(
            AltimeterEntry::from_str("HE1 2024 13 03 12 52 42 448 99999.99").unwrap_err(),
            AltimeterParseError::InvalidTimestamp(_)
        ));

        // Test Display formatting
        let he1 = AltimeterEntry::from_str(TEST_ENTRY_HE1)?;
        assert_eq!(he1.to_string(), "HE1 2024-10-03 12:52:42.448 UTC: 99999.99");
        Ok(())
    }

    #[test]
    fn test_roundtrip_parsing() -> TestResult {
        let original = AltimeterEntry {
            id: 1,
            timestamp: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str("2024-10-03 12:52:42.448", "%Y-%m-%d %H:%M:%S.%3f")?,
                Utc,
            ),
            altitude_m: 99999.99,
        };

        let parsed = AltimeterEntry::from_str(TEST_ENTRY_HE1)?;
        assert_eq!(original, parsed);

        Ok(())
    }
}
