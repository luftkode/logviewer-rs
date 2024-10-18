use chrono::{DateTime, NaiveDateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Display, PartialEq, Deserialize, Serialize)]
#[display("MA{id} {timestamp}: {field_nanotesla}")]
pub struct MagSensor {
    pub id: u8,
    timestamp: DateTime<Utc>,
    field_nanotesla: f64,
}

impl MagSensor {
    pub(crate) fn timestamp_ns(&self) -> f64 {
        self.timestamp
            .timestamp_nanos_opt()
            .expect("timestamp as nanoseconds out of range") as f64
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum MagSensorParseError {
    #[error("Invalid format")]
    InvalidFormat,
    #[error("Invalid ID")]
    InvalidId,
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(#[from] chrono::ParseError),
    #[error("Invalid field strength")]
    InvalidFieldStrength,
}

impl FromStr for MagSensor {
    type Err = MagSensorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 9 {
            return Err(MagSensorParseError::InvalidFormat);
        }

        // Parse ID (format: "MA1" -> 1)
        let id = parts[0]
            .strip_prefix("MA")
            .and_then(|id| id.parse().ok())
            .ok_or(MagSensorParseError::InvalidId)?;

        // Parse timestamp
        let timestamp_str = format!(
            "{} {} {} {} {} {} {}",
            parts[1], parts[2], parts[3], parts[4], parts[5], parts[6], parts[7]
        );
        let naive_dt = NaiveDateTime::parse_from_str(&timestamp_str, "%Y %m %d %H %M %S %3f")?;
        let timestamp = DateTime::from_naive_utc_and_offset(naive_dt, Utc);

        // Parse field strength
        let field_nanotesla = parts[8]
            .parse()
            .map_err(|_| MagSensorParseError::InvalidFieldStrength)?;

        Ok(MagSensor {
            id,
            timestamp,
            field_nanotesla,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testresult::TestResult;

    const TEST_ENTRY_MA1: &str = "MA1 2024 10 03 15 00 18 491 49750.1573";
    const TEST_ENTRY_MA2: &str = "MA2 2024 10 03 15 00 18 592 49751.2684";
    const TEST_TWO_LINES_BOTH: &str = "MA1 2024 10 03 15 00 18 491 49750.1573
MA2 2024 10 03 15 00 18 592 49751.2684
";

    #[test]
    fn test_parse_magsensor_line() -> TestResult {
        // Test parsing MA1
        let ma1 = MagSensor::from_str(TEST_ENTRY_MA1)?;
        assert_eq!(ma1.id, 1);
        assert_eq!(ma1.field_nanotesla, 49750.1573);
        assert_eq!(
            ma1.timestamp.naive_utc(),
            NaiveDateTime::parse_from_str("2024-10-03 15:00:18.491", "%Y-%m-%d %H:%M:%S.%3f")?
        );

        // Test parsing MA2
        let ma2 = MagSensor::from_str(TEST_ENTRY_MA2)?;
        assert_eq!(ma2.id, 2);
        assert_eq!(ma2.field_nanotesla, 49751.2684);
        assert_eq!(
            ma2.timestamp.naive_utc(),
            NaiveDateTime::parse_from_str("2024-10-03 15:00:18.592", "%Y-%m-%d %H:%M:%S.%3f")?
        );

        Ok(())
    }

    #[test]
    fn test_misc() -> TestResult {
        // Test parsing multiple lines fails appropriately
        assert!(MagSensor::from_str(TEST_TWO_LINES_BOTH).is_err());

        // Test error cases
        assert!(matches!(
            MagSensor::from_str("invalid").unwrap_err(),
            MagSensorParseError::InvalidFormat
        ));
        assert!(matches!(
            MagSensor::from_str("MAX 2024 10 03 15 00 18 491 49750.1573").unwrap_err(),
            MagSensorParseError::InvalidId
        ));
        assert!(matches!(
            MagSensor::from_str("MA1 2024 10 03 15 00 18 491 invalid").unwrap_err(),
            MagSensorParseError::InvalidFieldStrength
        ));
        assert!(matches!(
            MagSensor::from_str("MA1 2024 13 03 15 00 18 491 49750.1573").unwrap_err(),
            MagSensorParseError::InvalidTimestamp(_)
        ));

        // Test Display formatting
        let ma1 = MagSensor::from_str(TEST_ENTRY_MA1)?;
        assert_eq!(
            ma1.to_string(),
            "MA1 2024-10-03 15:00:18.491 UTC: 49750.1573"
        );
        Ok(())
    }

    #[test]
    fn test_roundtrip_parsing() -> TestResult {
        let original = MagSensor {
            id: 1,
            timestamp: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str("2024-10-03 15:00:18.491", "%Y-%m-%d %H:%M:%S.%3f")?,
                Utc,
            ),
            field_nanotesla: 49750.1573,
        };

        let parsed = MagSensor::from_str(TEST_ENTRY_MA1)?;
        assert_eq!(original, parsed);

        Ok(())
    }
}
