use chrono::{DateTime, NaiveDateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{fmt, io};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct NavSysSpsHeader {
    version: u32,
    first_timestamp: DateTime<Utc>,
    software_rev: String,
    tilt_sensor_id: String,
    calibration_data_1: CalibrationData,
    calibration_data_2: CalibrationData,
}

impl NavSysSpsHeader {
    pub(crate) fn version(&self) -> u32 {
        self.version
    }
    pub(crate) fn first_timestamp(&self) -> DateTime<Utc> {
        self.first_timestamp
    }
    pub(crate) fn software_revision(&self) -> &str {
        &self.software_rev
    }
    pub(crate) fn tilt_sensor_id(&self) -> &str {
        &self.tilt_sensor_id
    }
}

impl fmt::Display for NavSysSpsHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Timestamp: {}", self.first_timestamp)?;
        writeln!(f, "Software Revision: {}", self.software_rev)?;
        writeln!(f, "Tilt sensor ID: {}", self.tilt_sensor_id)?;
        writeln!(f, "#1 Calibration Data: {}", self.calibration_data_1)?;
        writeln!(f, "#2 Calibration Data: {}", self.calibration_data_2)?;
        Ok(())
    }
}

#[derive(Debug, Display, Clone, PartialEq, Deserialize, Serialize)]
#[display(
    "Angle offset Y: {angle_offset_y}
Angle Y: {angle_y}
Angle offset X: {angle_offset_x}
Angle X: {angle_x}"
)]
struct CalibrationData {
    angle_offset_y: f64,
    angle_y: f64,
    angle_offset_x: f64,
    angle_x: f64,
}

impl CalibrationData {
    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)> {
        let mut bytes_read = 0;
        let mut line = String::new();

        let mut parse_value = |prefix: &str| -> io::Result<f64> {
            line.clear();
            bytes_read += reader.read_line(&mut line)?;
            line.split(prefix)
                .nth(1)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("Missing {prefix}"))
                })?
                .trim()
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse float"))
        };
        let angle_offset_y = parse_value("OffsetY: ")?;
        let angle_y = parse_value("Y: ")?;
        let angle_offset_x = parse_value("OffsetX: ")?;
        let angle_x = parse_value("X: ")?;

        Ok((
            CalibrationData {
                angle_offset_y,
                angle_y,
                angle_offset_x,
                angle_x,
            },
            bytes_read,
        ))
    }
}

impl NavSysSpsHeader {
    /// Attempts to deserialize a [`NavSysSpsHeader`] from a reader
    ///
    /// On success, returns a [`NavSysSpsHeader`] and how many bytes were read from the reader
    pub fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)> {
        let mut bytes_read = 0;
        let mut line = String::new();

        // Parse version
        bytes_read += reader.read_line(&mut line)?;
        let version: u32 = line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid version format"))?
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse version"))?;

        // Parse first timestamp (format: MRK YYYY MM DD HH MM SS SSS)
        line.clear();
        bytes_read += reader.read_line(&mut line)?;
        let timestamp_str = line
            .split_whitespace()
            .skip(1)
            .take(7)
            .collect::<Vec<_>>()
            .join(" ");
        let first_timestamp =
            NaiveDateTime::parse_from_str(&timestamp_str, "%Y %m %d %H %M %S %3f")
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid timestamp format")
                })?
                .and_utc();

        // Parse software revision from the same string as above
        let software_rev = line
            .split("Navsys software rev: ")
            .nth(1)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid software revision format",
                )
            })?
            .trim()
            .to_string();

        // Parse tilt sensor ID
        line.clear();
        bytes_read += reader.read_line(&mut line)?;
        let tilt_sensor_id = line
            .split("TiltSensorID : ")
            .nth(1)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid tilt sensor ID format")
            })?
            .trim()
            .to_string();

        // Parse both calibration data sets
        let (calibration_data_1, bytes_cal_1) = CalibrationData::from_reader(reader)?;
        let (calibration_data_2, bytes_cal_2) = CalibrationData::from_reader(reader)?;
        bytes_read += bytes_cal_1 + bytes_cal_2;

        Ok((
            NavSysSpsHeader {
                version,
                first_timestamp,
                software_rev,
                tilt_sensor_id,
                calibration_data_1,
                calibration_data_2,
            },
            bytes_read,
        ))
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    const TEST_HEADER_STR: &str = "VER 3
MRK 2024 10 03 12 52 42 401 Navsys software rev: Build: 2.0.0.6
MRK 2024 10 03 12 52 42 417 TiltSensorID : 1459_1458
MRK 2024 10 03 12 52 42 417 CalAng 1 OffsetY: 0.4950
MRK 2024 10 03 12 52 42 417 CalAng 1 Y: 3.26488
MRK 2024 10 03 12 52 42 432 CalAng 1 OffsetX: 0.5099
MRK 2024 10 03 12 52 42 432 CalAng 1 X: 3.28745
MRK 2024 10 03 12 52 42 432 CalAng 2 OffsetY: 0.4947
MRK 2024 10 03 12 52 42 432 CalAng 2 Y: 3.34307
MRK 2024 10 03 12 52 42 432 CalAng 2 OffsetX: 0.5047
MRK 2024 10 03 12 52 42 432 CalAng 2 X: 3.30737";

    #[test]
    fn test_parse_header() -> TestResult {
        let mut reader = TEST_HEADER_STR.as_bytes();
        let (header, bytes_read) = NavSysSpsHeader::from_reader(&mut reader)?;

        assert_eq!(bytes_read, 526);

        assert_eq!(header.version, 3);
        assert_eq!(
            header.first_timestamp,
            NaiveDateTime::parse_from_str("2024-10-03 12:52:42.401", "%Y-%m-%d %H:%M:%S.%3f")?
                .and_utc()
        );
        assert_eq!(header.software_rev, "Build: 2.0.0.6");
        assert_eq!(header.tilt_sensor_id, "1459_1458");

        // Check calibration data 1
        assert_eq!(header.calibration_data_1.angle_offset_y, 0.4950);
        assert_eq!(header.calibration_data_1.angle_y, 3.26488);
        assert_eq!(header.calibration_data_1.angle_offset_x, 0.5099);
        assert_eq!(header.calibration_data_1.angle_x, 3.28745);

        // Check calibration data 2
        assert_eq!(header.calibration_data_2.angle_offset_y, 0.4947);
        assert_eq!(header.calibration_data_2.angle_y, 3.34307);
        assert_eq!(header.calibration_data_2.angle_offset_x, 0.5047);
        assert_eq!(header.calibration_data_2.angle_x, 3.30737);

        assert_eq!(bytes_read, TEST_HEADER_STR.len());
        Ok(())
    }

    #[test]
    fn test_display_header() -> TestResult {
        let (header, _) = NavSysSpsHeader::from_reader(&mut TEST_HEADER_STR.as_bytes())?;
        eprintln!("{header}");

        let expected_string_format = r#"Version: 3
Timestamp: 2024-10-03 12:52:42.401 UTC
Software Revision: Build: 2.0.0.6
Tilt sensor ID: 1459_1458
#1 Calibration Data: Angle offset Y: 0.495
Angle Y: 3.26488
Angle offset X: 0.5099
Angle X: 3.28745
#2 Calibration Data: Angle offset Y: 0.4947
Angle Y: 3.34307
Angle offset X: 0.5047
Angle X: 3.30737
"#;
        assert_eq!(header.to_string(), expected_string_format);

        Ok(())
    }
}
