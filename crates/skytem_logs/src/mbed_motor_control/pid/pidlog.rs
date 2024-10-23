use chrono::{DateTime, Utc};
use log_if::{log::LogEntry, parseable::Parseable, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    io::{self, Read},
    path::Path,
};

use crate::{
    mbed_motor_control::{
        mbed_config::MbedConfig,
        mbed_header::{MbedMotorControlLogHeader, SIZEOF_UNIQ_DESC},
    },
    parse_unique_description,
};

use super::{entry::PidLogEntry, header::PidLogHeader};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PidLog {
    header: PidLogHeader,
    entries: Vec<PidLogEntry>,
    timestamps_ns: Vec<f64>,
    all_plots_raw: Vec<RawPlot>,
    startup_timestamp: DateTime<Utc>,
}

impl PidLog {
    /// Checks if the file at the given path is a valid [`PidLog`] file
    pub fn file_is_valid(path: &Path) -> bool {
        let Ok(mut file) = fs::File::open(path) else {
            return false;
        };

        let mut buffer = vec![0u8; SIZEOF_UNIQ_DESC + 2];
        match file.read_exact(&mut buffer) {
            Ok(_) => Self::is_buf_valid(&buffer),
            Err(_) => false, // Return false if we can't read enough bytes
        }
    }
}

impl SkytemLog for PidLog {
    type Entry = PidLogEntry;

    fn entries(&self) -> &[Self::Entry] {
        &self.entries
    }
}

impl Parseable for PidLog {
    const DESCRIPTIVE_NAME: &str = "Mbed PID log";

    /// Probes the buffer and check if it starts with [`super::UNIQUE_DESCRIPTION`] and therefor contains a valid [`PidLog`]
    fn is_buf_valid(content: &[u8]) -> bool {
        if content.len() < SIZEOF_UNIQ_DESC + 2 {
            return false;
        }

        let unique_description = &content[..SIZEOF_UNIQ_DESC];
        parse_unique_description(unique_description) == super::UNIQUE_DESCRIPTION
    }

    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)> {
        let mut total_bytes_read: usize = 0;
        let (header, bytes_read) = PidLogHeader::from_reader(reader)?;
        total_bytes_read += bytes_read;
        let startup_timestamp = match &header {
            PidLogHeader::V1(h) => h.startup_timestamp(),
            PidLogHeader::V2(h) => h.startup_timestamp(),
            PidLogHeader::V3(h) => h.startup_timestamp(),
        }
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .and_utc();
        let startup_timestamp_ns = startup_timestamp
            .timestamp_nanos_opt()
            .expect("timestamp as nanoseconds out of range")
            as f64;
        let (vec_of_entries, entries_bytes_read): (Vec<PidLogEntry>, usize) = parse_to_vec(reader);
        let timestamps_ns: Vec<f64> = vec_of_entries
            .iter()
            .map(|e| startup_timestamp_ns + e.timestamp_ns())
            .collect();
        total_bytes_read += entries_bytes_read;

        let rpm_plot_raw = plot_points_from_log_entry(
            &vec_of_entries,
            |e| e.timestamp_ns() + startup_timestamp_ns,
            |e| e.rpm as f64,
        );
        let pid_err_plot_raw = plot_points_from_log_entry(
            &vec_of_entries,
            |e| e.timestamp_ns() + startup_timestamp_ns,
            |e| e.pid_output as f64,
        );
        let servo_duty_cycle_plot_raw = plot_points_from_log_entry(
            &vec_of_entries,
            |e| e.timestamp_ns() + startup_timestamp_ns,
            |e| e.servo_duty_cycle as f64,
        );
        let rpm_error_count_plot_raw = plot_points_from_log_entry(
            &vec_of_entries,
            |e| e.timestamp_ns() + startup_timestamp_ns,
            |e| e.rpm_error_count as f64,
        );
        let first_valid_rpm_count_plot_raw = plot_points_from_log_entry(
            &vec_of_entries,
            |e| e.timestamp_ns() + startup_timestamp_ns,
            |e| e.first_valid_rpm_count as f64,
        );

        let all_plots_raw = vec![
            RawPlot::new("RPM".into(), rpm_plot_raw, ExpectedPlotRange::Thousands),
            RawPlot::new(
                "PID Output".into(),
                pid_err_plot_raw,
                ExpectedPlotRange::Percentage,
            ),
            RawPlot::new(
                "Servo Duty Cycle".into(),
                servo_duty_cycle_plot_raw,
                ExpectedPlotRange::Percentage,
            ),
            RawPlot::new(
                "RPM Error Count".into(),
                rpm_error_count_plot_raw,
                ExpectedPlotRange::OneToOneHundred,
            ),
            RawPlot::new(
                "First Valid RPM Count".into(),
                first_valid_rpm_count_plot_raw,
                ExpectedPlotRange::OneToOneHundred,
            ),
        ];
        // Iterate through the plots and make sure all the first timestamps match
        if let Some(first_plot) = all_plots_raw.first() {
            if let Some([first_timestamp, ..]) = first_plot.points().first() {
                for p in &all_plots_raw {
                    if let Some([current_first_timestamp, ..]) = p.points().first() {
                        debug_assert_eq!(current_first_timestamp, first_timestamp, "First timestamp of plots are not equal, was an offset applied to some plots but not all?");
                    }
                }
            }
        }

        Ok((
            Self {
                header,
                entries: vec_of_entries,
                timestamps_ns,
                all_plots_raw,
                startup_timestamp,
            },
            total_bytes_read,
        ))
    }
}

impl GitMetadata for PidLog {
    fn project_version(&self) -> Option<String> {
        match &self.header {
            PidLogHeader::V1(h) => h.project_version(),
            PidLogHeader::V2(h) => h.project_version(),
            PidLogHeader::V3(h) => h.project_version(),
        }
    }

    fn git_short_sha(&self) -> Option<String> {
        match &self.header {
            PidLogHeader::V1(h) => h.git_short_sha(),
            PidLogHeader::V2(h) => h.git_short_sha(),
            PidLogHeader::V3(h) => h.git_short_sha(),
        }
    }

    fn git_branch(&self) -> Option<String> {
        match &self.header {
            PidLogHeader::V1(h) => h.git_branch(),
            PidLogHeader::V2(h) => h.git_branch(),
            PidLogHeader::V3(h) => h.git_branch(),
        }
    }

    fn git_repo_status(&self) -> Option<String> {
        match &self.header {
            PidLogHeader::V1(h) => h.git_repo_status(),
            PidLogHeader::V2(h) => h.git_repo_status(),
            PidLogHeader::V3(h) => h.git_repo_status(),
        }
    }
}

impl Plotable for PidLog {
    fn raw_plots(&self) -> &[RawPlot] {
        &self.all_plots_raw
    }

    fn first_timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.startup_timestamp
    }

    fn descriptive_name(&self) -> &str {
        match self.header {
            PidLogHeader::V1(_) => "Mbed PID v1",
            PidLogHeader::V2(_) => "Mbed PID v2",
            PidLogHeader::V3(_) => "Mbed PID v3",
        }
    }

    fn labels(&self) -> Option<&[PlotLabels]> {
        None
    }

    fn metadata(&self) -> Option<Vec<(String, String)>> {
        let mut metadata = vec![
            (
                "Project Version".to_owned(),
                self.project_version().unwrap_or_else(|| "N/A".to_owned()),
            ),
            (
                "Git Branch".to_owned(),
                self.git_branch().unwrap_or_else(|| "N/A".to_owned()),
            ),
            (
                "Git Repo Status".to_owned(),
                self.git_repo_status().unwrap_or_else(|| "Clean".to_owned()),
            ),
            (
                "Git Short SHA".to_owned(),
                self.git_short_sha().unwrap_or_else(|| "N/A".to_owned()),
            ),
            (
                "Startup Timestamp".to_owned(),
                self.startup_timestamp.naive_utc().to_string(),
            ),
        ];

        match self.header {
            // V1 has no more than that
            PidLogHeader::V1(_) => (),
            // V2 also has config values
            PidLogHeader::V2(h) => {
                metadata.push(("Config values".to_owned(), String::new()));
                metadata.extend_from_slice(&h.mbed_config().field_value_pairs());
            }
            PidLogHeader::V3(h) => {
                metadata.push(("Config values".to_owned(), String::new()));
                metadata.extend_from_slice(&h.mbed_config().field_value_pairs());
            }
        }

        Some(metadata)
    }
}

impl fmt::Display for PidLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Header: {}", self.header)?;
        for e in &self.entries {
            writeln!(f, "{e}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_and_display_log_entries;
    use std::fs::{self, File};
    use testresult::TestResult;

    const TEST_DATA_V1: &str =
        "../../test_data/mbed_motor_control/v1/20240926_121708/pid_20240926_121708_00.bin";
    const TEST_DATA_V2: &str =
        "../../test_data/mbed_motor_control/v2/20241014_080729/pid_20241014_080729_00.bin";

    #[test]
    fn test_deserialize_v1() -> TestResult {
        let data = fs::read(TEST_DATA_V1)?;
        let full_data_len = data.len();
        let (pidlog, bytes_read) = PidLog::from_reader(&mut data.as_slice())?;

        assert!(bytes_read <= full_data_len);
        assert_eq!(bytes_read, 873981);
        let first_entry = pidlog.entries.first().expect("Empty entries");
        assert_eq!(first_entry.rpm, 0.0);
        assert_eq!(first_entry.pid_output, 0.17777778);
        assert_eq!(first_entry.servo_duty_cycle, 0.04185);
        assert_eq!(first_entry.rpm_error_count, 0);
        assert_eq!(first_entry.first_valid_rpm_count, 0);

        let second_entry = &pidlog.entries[1];
        assert_eq!(second_entry.rpm, 0.0);
        assert_eq!(second_entry.pid_output, 0.17777778);
        assert_eq!(second_entry.servo_duty_cycle, 0.04185);
        assert_eq!(second_entry.rpm_error_count, 0);
        assert_eq!(second_entry.first_valid_rpm_count, 0);
        //eprintln!("{pidlog}");
        Ok(())
    }

    #[test]
    fn test_parse_and_display_v1() -> TestResult {
        let file = File::open(TEST_DATA_V1)?;
        let mut reader = io::BufReader::new(file);
        let (header, bytes_read) = PidLogHeader::from_reader(&mut reader)?;
        assert_eq!(bytes_read, 261);
        println!("{header}");
        parse_and_display_log_entries::<PidLogEntry>(&mut reader, Some(10));
        Ok(())
    }

    #[test]
    fn test_deserialize_v2() -> TestResult {
        let data = fs::read(TEST_DATA_V2)?;
        let full_data_len = data.len();
        let (pidlog, bytes_read) = PidLog::from_reader(&mut data.as_slice())?;
        assert!(bytes_read <= full_data_len);
        assert_eq!(bytes_read, 722429);
        let first_entry = pidlog.entries.first().expect("Empty entries");
        assert_eq!(first_entry.rpm, 0.0);
        assert_eq!(first_entry.pid_output, 0.0);
        assert_eq!(first_entry.servo_duty_cycle, 0.03825);
        assert_eq!(first_entry.rpm_error_count, 0);
        assert_eq!(first_entry.first_valid_rpm_count, 0);

        let second_entry = &pidlog.entries[1];
        assert_eq!(second_entry.rpm, 0.0);
        assert_eq!(second_entry.pid_output, 0.0);
        assert_eq!(second_entry.servo_duty_cycle, 0.03825);
        assert_eq!(second_entry.rpm_error_count, 0);
        assert_eq!(second_entry.first_valid_rpm_count, 0);
        //eprintln!("{pidlog}");
        Ok(())
    }

    #[test]
    fn test_parse_and_display_v2() -> TestResult {
        let file = File::open(TEST_DATA_V2)?;
        let mut reader = io::BufReader::new(file);
        let (header, bytes_read) = PidLogHeader::from_reader(&mut reader)?;
        assert_eq!(bytes_read, 293);
        println!("{header}");
        parse_and_display_log_entries::<PidLogEntry>(&mut reader, Some(10));
        Ok(())
    }
}
