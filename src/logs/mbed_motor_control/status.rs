use std::{fmt, io};

use entry::{MotorState, StatusLogEntry};
use header::StatusLogHeader;

use crate::logs::{parse_to_vec, Log};

use super::MbedMotorControlLogHeader;

pub mod entry;
pub mod header;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct StatusLog {
    header: StatusLogHeader,
    entries: Vec<StatusLogEntry>,
    timestamps_with_state_changes: Vec<(u32, MotorState)>, // for memoization
}

impl Log for StatusLog {
    type Entry = StatusLogEntry;

    fn from_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let header = StatusLogHeader::from_reader(reader)?;
        let vec_of_entries: Vec<StatusLogEntry> = parse_to_vec(reader);
        let timestamps_with_state_changes = parse_timestamps_with_state_changes(&vec_of_entries);
        Ok(Self {
            header,
            entries: vec_of_entries,
            timestamps_with_state_changes,
        })
    }

    fn entries(&self) -> &[Self::Entry] {
        &self.entries
    }
}

impl StatusLog {
    pub fn timestamps_with_state_changes(&self) -> &[(u32, MotorState)] {
        &self.timestamps_with_state_changes
    }
}

impl fmt::Display for StatusLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Header: {}", self.header)?;
        for e in &self.entries {
            writeln!(f, "{e}")?;
        }
        Ok(())
    }
}

fn parse_timestamps_with_state_changes(entries: &[StatusLogEntry]) -> Vec<(u32, MotorState)> {
    let mut result = Vec::new();
    let mut last_state = None;

    for entry in entries.iter() {
        // Check if the current state is different from the last recorded state
        if last_state != Some(entry.motor_state) {
            result.push((entry.timestamp_ms, entry.motor_state));
            last_state = Some(entry.motor_state);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use testresult::TestResult;

    const TEST_DATA: &str =
        "test_data/mbed_motor_control/old_rpm_algo/status_20240912_122203_00.bin";

    use crate::logs::{
        mbed_motor_control::MbedMotorControlLogHeader, parse_and_display_log_entries,
    };

    use super::*;

    #[test]
    fn test_deserialize() -> TestResult {
        let data = fs::read(TEST_DATA)?;
        let status_log = StatusLog::from_reader(&mut data.as_slice())?;
        eprintln!("{}", status_log.header);

        let first_entry = status_log.entries.first().unwrap();
        assert_eq!(first_entry.engine_temp, 4.770642);
        assert!(!first_entry.fan_on);
        assert_eq!(first_entry.vbat, 4.211966);
        assert_eq!(first_entry.setpoint, 2500.0);
        assert_eq!(first_entry.motor_state, MotorState::POWER_HOLD);
        let second_entry = status_log.entries.get(1).unwrap();
        assert_eq!(second_entry.engine_temp, 4.770642);
        assert!(!second_entry.fan_on);
        assert_eq!(second_entry.vbat, 4.219487);
        assert_eq!(second_entry.setpoint, 2500.0);
        assert_eq!(second_entry.motor_state, MotorState::POWER_HOLD);

        let last_entry = status_log.entries().last().unwrap();
        assert_eq!(last_entry.timestamp_ms(), 17492);
        assert_eq!(last_entry.engine_temp, 4.770642);
        assert!(!last_entry.fan_on);
        assert_eq!(last_entry.vbat, 4.219487);
        assert_eq!(last_entry.setpoint, 0.0);
        assert_eq!(last_entry.motor_state, MotorState::WAIT_TIME_SHUTDOWN);
        //eprintln!("{status_log}");
        Ok(())
    }

    #[test]
    fn test_parse_and_display() -> TestResult {
        let file = File::open(TEST_DATA)?;
        let mut reader = io::BufReader::new(file);
        let header = StatusLogHeader::from_reader(&mut reader)?;
        println!("{header}");
        parse_and_display_log_entries::<StatusLogEntry, _>(&mut reader, Some(10));
        Ok(())
    }
}
