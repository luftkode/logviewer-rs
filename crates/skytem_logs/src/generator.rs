use std::{
    fmt, fs,
    io::{self, BufRead, BufReader},
    path::Path,
    str::FromStr,
};

use chrono::NaiveDateTime;
use log_if::{log::LogEntry, parseable::Parseable, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GeneratorLog {
    entries: Vec<GeneratorLogEntry>,
    power: Vec<f64>, // Calculated from Vout * Vin
    /// timestamps in nanoseconds since the epoch
    timestamps_ns: Vec<f64>,
    all_plots_raw: Vec<RawPlot>,
}

impl GeneratorLog {
    pub fn file_is_generator_log(fpath: &Path) -> io::Result<bool> {
        let file = fs::File::open(fpath)?;
        let mut buf_reader = BufReader::new(file);
        let mut first_line = String::new();
        _ = buf_reader.read_line(&mut first_line)?;
        let is_first_line_gen_log_entry =
            GeneratorLogEntry::is_line_valid_generator_log_entry(&first_line);

        Ok(is_first_line_gen_log_entry)
    }
}

impl SkytemLog for GeneratorLog {
    type Entry = GeneratorLogEntry;

    fn entries(&self) -> &[Self::Entry] {
        &self.entries
    }
}

impl Parseable for GeneratorLog {
    const DESCRIPTIVE_NAME: &str = "Legacy Generator Log";
    fn is_buf_valid(buf: &[u8]) -> bool {
        let mut bufreader = BufReader::new(buf);
        let mut line = String::new();
        if bufreader.read_line(&mut line).is_err() {
            return false;
        }
        GeneratorLogEntry::is_line_valid_generator_log_entry(&line)
    }

    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)> {
        let mut entries = Vec::new();
        let mut total_bytes_read = 0;

        // Read the buffer in chunks and handle the line parsing
        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line)?;

            // If we didn't read any bytes, we're done
            if bytes_read == 0 {
                break;
            }

            // Create a Cursor from the line string to pass to from_reader
            let line_bytes = line.as_bytes();
            match GeneratorLogEntry::from_reader(&mut io::Cursor::new(line_bytes)) {
                Ok((entry, _)) => {
                    total_bytes_read += bytes_read;
                    entries.push(entry);
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => log::warn!("Failed parsing generator log entry: {e}... Continuing"),
            }
        }

        let mut power_vals: Vec<f64> = Vec::with_capacity(entries.len());
        for e in &entries {
            let power = (e.vout as f64) * (e.i_in as f64);
            power_vals.push(power);
        }

        let mut timestamps_ns: Vec<f64> = Vec::with_capacity(entries.len());
        for entry in &entries {
            timestamps_ns.push(entry.timestamp_ns());
        }

        let all_plots_raw = build_all_plots(&entries);
        Ok((
            Self {
                entries,
                power: power_vals,
                all_plots_raw,
                timestamps_ns,
            },
            total_bytes_read,
        ))
    }
}

impl Plotable for GeneratorLog {
    fn raw_plots(&self) -> &[RawPlot] {
        &self.all_plots_raw
    }

    fn first_timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.entries()
            .first()
            .expect("No entries")
            .timestamp
            .and_utc()
    }

    fn descriptive_name(&self) -> &str {
        "Legacy Generator"
    }

    fn labels(&self) -> Option<&[PlotLabels]> {
        None
    }

    fn metadata(&self) -> Option<Vec<(String, String)>> {
        None
    }
}

impl GitMetadata for GeneratorLog {
    fn project_version(&self) -> Option<String> {
        None
    }

    fn git_short_sha(&self) -> Option<String> {
        None
    }

    fn git_branch(&self) -> Option<String> {
        None
    }

    fn git_repo_status(&self) -> Option<String> {
        None
    }
}

// Helper function to keep all the boiler plate of building each plot
fn build_all_plots(entries: &[GeneratorLogEntry]) -> Vec<RawPlot> {
    vec![
        RawPlot::new(
            "Rotor [R]".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.r_rotor.into()),
            ExpectedPlotRange::OneToOneHundred,
        ),
        RawPlot::new(
            "RPM".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.rpm.into()),
            ExpectedPlotRange::Thousands,
        ),
        RawPlot::new(
            "Power [W]".into(),
            plot_points_from_log_entry(
                entries,
                |e| e.timestamp_ns(),
                |e| f64::from(e.vout) * f64::from(e.i_in),
            ),
            ExpectedPlotRange::Thousands,
        ),
        RawPlot::new(
            "PWM [%]".into(),
            // Load is percentage but in the log it is represented as 0-100 so we divide by 100 to normalize to [0.0,1.0]
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| (e.pwm / 100.0).into()),
            ExpectedPlotRange::Percentage,
        ),
        RawPlot::new(
            "Load [%]".into(),
            // Load is percentage but in the log it is represented as 0-100 so we divide by 100 to normalize to [0.0,1.0]
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| (e.load / 100.0).into()),
            ExpectedPlotRange::Percentage,
        ),
        RawPlot::new(
            "Rotor [I]".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.i_rotor.into()),
            ExpectedPlotRange::OneToOneHundred,
        ),
        RawPlot::new(
            "Temp1 °C".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.temp1.into()),
            ExpectedPlotRange::OneToOneHundred,
        ),
        RawPlot::new(
            "Temp2 °C".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.temp2.into()),
            ExpectedPlotRange::OneToOneHundred,
        ),
        RawPlot::new(
            "I_in".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.i_in.into()),
            ExpectedPlotRange::OneToOneHundred,
        ),
        RawPlot::new(
            "Iout".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.i_out.into()),
            ExpectedPlotRange::OneToOneHundred,
        ),
        RawPlot::new(
            "Vbat [V]".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.vbat.into()),
            ExpectedPlotRange::OneToOneHundred,
        ),
        RawPlot::new(
            "Vout [V]".into(),
            plot_points_from_log_entry(entries, |e| e.timestamp_ns(), |e| e.vout.into()),
            ExpectedPlotRange::OneToOneHundred,
        ),
    ]
}

impl fmt::Display for GeneratorLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Generator Log:")?;
        for (index, entry) in self.entries.iter().enumerate() {
            writeln!(f, "Entry {}: {}", index + 1, entry)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub struct GeneratorLogEntry {
    pub timestamp: NaiveDateTime,
    pub vout: f32,
    pub vbat: f32,
    pub i_out: f32,
    pub rpm: u32,
    pub load: f32,
    pub pwm: f32,
    pub temp1: f32,
    pub temp2: f32,
    pub i_in: f32,
    pub i_rotor: f32,
    pub r_rotor: f32,
}

impl GeneratorLogEntry {
    pub fn is_line_valid_generator_log_entry(line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if *parts.get(1).unwrap_or(&"") != "Vout:" {
            return false;
        }
        if *parts.get(3).unwrap_or(&"") != "Vbat:" {
            return false;
        }
        if *parts.get(5).unwrap_or(&"") != "Iout:" {
            return false;
        }
        if *parts.get(7).unwrap_or(&"") != "RPM:" {
            return false;
        }
        if *parts.get(9).unwrap_or(&"") != "Load:" {
            return false;
        }
        if *parts.get(11).unwrap_or(&"") != "PWM:" {
            return false;
        }
        if *parts.get(13).unwrap_or(&"") != "Temp1" {
            return false;
        }
        if *parts.get(15).unwrap_or(&"") != "Temp2" {
            return false;
        }
        if *parts.get(17).unwrap_or(&"") != "IIn:" {
            return false;
        }
        if *parts.get(19).unwrap_or(&"") != "Irotor:" {
            return false;
        }
        if *parts.get(21).unwrap_or(&"") != "Rrotor:" {
            return false;
        }
        true
    }
}

impl LogEntry for GeneratorLogEntry {
    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)> {
        let mut line = String::new();

        // Read the line and track the number of bytes read
        let bytes_read = reader.read_line(&mut line)?;

        let gen_log_entry = Self::from_str(&line)?;

        Ok((gen_log_entry, bytes_read))
    }

    /// Timestamp in nanoseconds since the epoch
    fn timestamp_ns(&self) -> f64 {
        self.timestamp
            .and_utc()
            .timestamp_nanos_opt()
            .expect("timestamp as nanoseconds out of range") as f64
    }
}

impl fmt::Display for GeneratorLogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{timestamp}: {vout} {vbat} {i_out} {rpm} {load} {pwm} {temp1} {temp2} {i_in} {i_rotor} {r_rotor}",
            timestamp = self.timestamp,
            vout = self.vout,
            vbat =  self.vbat,
            i_out = self.i_out,
            rpm = self.rpm,
            load = self.load,
            pwm = self.pwm,
            temp1 = self.temp1,
            temp2 = self.temp2,
            i_in = self.i_in,
            i_rotor = self.i_rotor,
            r_rotor = self.r_rotor
        )
    }
}

impl FromStr for GeneratorLogEntry {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() != 23 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid log entry format",
            ));
        }

        Ok(Self {
            timestamp: NaiveDateTime::parse_from_str(parts[0], "%Y%m%d_%H%M%S")
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            vout: parts[2]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            vbat: parts[4]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            i_out: parts[6]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            rpm: parts[8]
                .parse::<u32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            load: parts[10]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            pwm: parts[12]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            temp1: parts[14]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            temp2: parts[16]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            i_in: parts[18]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            i_rotor: parts[20]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            r_rotor: parts[22]
                .parse::<f32>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use chrono::NaiveDate;
    use testresult::TestResult;

    const TEST_DATA: &str = "../../test_data/generator/20230124_134738_Gen.log";

    #[test]
    fn test_deserialize() -> TestResult {
        let data = fs::read(TEST_DATA)?;
        let full_data_len = data.len();
        let (log, bytes_read) = GeneratorLog::from_reader(&mut data.as_slice())?;

        assert_eq!(bytes_read, full_data_len);
        let first_entry = log.entries().first().expect("Empty entries");

        let first_ts_ns = first_entry.timestamp_ns();
        assert_eq!(
            first_ts_ns,
            NaiveDate::from_ymd_opt(2023, 1, 24)
                .expect("Invalid arguments")
                .and_hms_opt(13, 47, 45)
                .expect("Invalid arguments")
                .and_utc()
                .timestamp_nanos_opt()
                .expect("timestamp as nanoseconds out of range") as f64
        );
        assert_eq!(
            first_entry.timestamp,
            NaiveDate::from_ymd_opt(2023, 1, 24)
                .expect("Invalid arguments")
                .and_hms_opt(13, 47, 45)
                .expect("Invalid arguments")
        );
        assert_eq!(first_entry.vout, 74.3);
        assert_eq!(first_entry.vbat, 0.1);
        assert_eq!(first_entry.i_out, 0.0);
        assert_eq!(first_entry.rpm, 6075);
        assert_eq!(first_entry.load, 10.2);
        assert_eq!(first_entry.pwm, 10.2);
        assert_eq!(first_entry.temp1, 6.9);
        assert_eq!(first_entry.temp2, 8.4);
        assert_eq!(first_entry.i_in, 8.8);
        assert_eq!(first_entry.i_rotor, 0.7);
        assert_eq!(first_entry.r_rotor, 11.2);

        let last_entry = log.entries().last().expect("Empty entries");
        assert_eq!(
            last_entry.timestamp,
            NaiveDate::from_ymd_opt(2023, 1, 24)
                .expect("Invalid arguments")
                .and_hms_opt(15, 3, 30)
                .expect("Invalid arguments")
        );
        assert_eq!(last_entry.vout, 78.3);
        assert_eq!(last_entry.vbat, 0.1);
        assert_eq!(last_entry.i_out, 0.0);
        assert_eq!(last_entry.rpm, 5932);
        assert_eq!(last_entry.load, 9.7);
        assert_eq!(last_entry.pwm, 9.7);
        assert_eq!(last_entry.temp1, 6.9);
        assert_eq!(last_entry.temp2, 8.5);
        assert_eq!(last_entry.i_in, 8.3);
        assert_eq!(last_entry.i_rotor, 0.7);
        assert_eq!(last_entry.r_rotor, 11.0);

        Ok(())
    }

    #[test]
    fn test_is_valid_line_valid() {
        let valid_line = "20230124_134852 Vout: 77.8 Vbat: 0.1 Iout: 0.0 RPM: 5925 Load: 17.6 PWM: 17.5 Temp1 7.2 Temp2 9.9 IIn: 61.6 Irotor: 1.4 Rrotor: 9.7";

        assert!(GeneratorLogEntry::is_line_valid_generator_log_entry(
            valid_line
        ));
    }

    #[test]
    fn test_is_valid_line_invalid() {
        let invalid_line =
            "20230124_134852 Vout: 77.8 Vbat: 0.1 Iout: 0.0 RPM: 5925 something_else: 9.7";

        assert!(!GeneratorLogEntry::is_line_valid_generator_log_entry(
            invalid_line
        ));
    }

    #[test]
    fn test_is_bytes_valid_line_valid() {
        let valid_line_as_bytes = b"20230124_134745 Vout: 74.3 Vbat: 0.1 Iout: 0.0 RPM: 6075 Load: 10.2 PWM: 10.2 Temp1 6.9 Temp2 8.4 IIn: 8.8 Irotor: 0.7 Rrotor: 11.2
20230124_134746 Vout: 59.3 Vbat: 0.1 Iout: 0.0 RPM: 5438 Load: 81.2 PWM: 18.0 Temp1 6.9 Temp2 8.6 IIn: 35.5 Irotor: 0.9 Rrotor: 11.5";

        assert!(GeneratorLog::is_buf_valid(valid_line_as_bytes));
    }

    #[test]
    fn test_is_bytes_valid_line_invalid() {
        let invalid_bytes = b"20230124_134745 Vo 74. 0.1 Iout: 0.0 RPM: 6075 Load: 10.2 PWM:  8.4 IIn: 8.8 Irotor: 0.7 Rrotor: 11.2
20230124_134746 Vout: 59.3 Vbat: 0.1 Iout: 0.0 RPM: 5438 Load: 81.2 PWM: 18.0 Temp1 6.9 Temp2 8.6 IIn: 35.5 Irotor: 0.9 Rrotor: 11.5";

        assert!(!GeneratorLog::is_buf_valid(invalid_bytes));
    }

    #[test]
    fn test_parse_valid_then_partial_valid() -> TestResult {
        let valid_line_then_invalid_as_bytes = b"20230124_134745 Vout: 74.3 Vbat: 0.1 Iout: 0.0 RPM: 6075 Load: 10.2 PWM: 10.2 Temp1 6.9 Temp2 8.4 IIn: 8.8 Irotor: 0.7 Rrotor: 11.2
20230124_134746 Vout: 59.3 Vbat: 0.1 Iout: 0.0 RPM: 5438 Load: 81.2 PWM: 18.0 Temp1 6.9 Temp2 8.6 IIn: 35.5 Irotor: 0.9 Rro
";
        let mut readable = io::Cursor::new(valid_line_then_invalid_as_bytes);

        let (genlog, bytes_read) = GeneratorLog::from_reader(&mut readable)?;
        assert_eq!(genlog.entries.len(), 1);
        assert!(bytes_read < valid_line_then_invalid_as_bytes.len());
        Ok(())
    }
}
