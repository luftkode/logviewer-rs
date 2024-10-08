use egui::DroppedFile;
use log_if::prelude::*;
use serde::{Deserialize, Serialize};
use skytem_logs::{
    generator::{GeneratorLog, GeneratorLogEntry},
    mbed_motor_control::{
        mbed_header::MbedMotorControlLogHeader,
        pid::{
            header_v1::PidLogHeaderV1, header_v2::PidLogHeaderV2, pidlog_v1::PidLogV1,
            pidlog_v2::PidLogV2,
        },
        status::{
            header_v1::StatusLogHeaderV1, header_v2::StatusLogHeaderV2, statuslog_v1::StatusLogV1,
            statuslog_v2::StatusLogV2,
        },
    },
};
use std::{
    fs,
    io::{self, BufReader},
    path::{self, Path},
};

/// In the ideal future, this explicit list of supported logs is instead just a vector of log interfaces (traits)
/// that would require the log interface to also support a common way for plotting logs
#[derive(Default, Deserialize, Serialize)]
pub struct SupportedLogs {
    pid_log_v1: Vec<PidLogV1>,
    pid_log_v2: Vec<PidLogV2>,
    status_log_v1: Vec<StatusLogV1>,
    status_log_v2: Vec<StatusLogV2>,
    generator_log: Vec<GeneratorLog>,
}

impl SupportedLogs {
    /// Return a vector of immutable references to all logs
    pub fn logs(&self) -> Vec<&dyn Plotable> {
        let mut all_logs: Vec<&dyn Plotable> = Vec::new();
        for pl in &self.pid_log_v1 {
            all_logs.push(pl);
        }
        for pl in &self.pid_log_v2 {
            all_logs.push(pl);
        }
        for sl in &self.status_log_v1 {
            all_logs.push(sl);
        }
        for sl in &self.status_log_v2 {
            all_logs.push(sl);
        }
        for gl in &self.generator_log {
            all_logs.push(gl);
        }
        all_logs
    }

    /// Take all the logs currently store in [`SupportedLogs`] and return them as a list
    pub fn take_logs(&mut self) -> Vec<Box<dyn Plotable>> {
        let mut all_logs: Vec<Box<dyn Plotable>> = Vec::new();
        all_logs.extend(self.pid_log_v1.drain(..).map(|log| log.into()));
        all_logs.extend(self.pid_log_v2.drain(..).map(|log| log.into()));
        all_logs.extend(self.status_log_v1.drain(..).map(|log| log.into()));
        all_logs.extend(self.status_log_v2.drain(..).map(|log| log.into()));
        all_logs.extend(self.generator_log.drain(..).map(|log| log.into()));

        all_logs
    }

    /// Parse dropped files to supported logs.
    ///
    /// ### Note to developers who are not seasoned Rust devs :)
    /// This cannot take `&mut self` as that breaks ownership rules when looping over dropped files
    /// meaning you would be forced to make a copy which isn't actually needed, but required for it to compile.
    pub fn parse_dropped_files(&mut self, dropped_files: &[DroppedFile]) -> io::Result<()> {
        for file in dropped_files {
            log::debug!("Parsing dropped file: {file:?}");
            self.parse_file(file)?;
        }
        Ok(())
    }

    fn parse_file(&mut self, file: &DroppedFile) -> io::Result<()> {
        if let Some(content) = file.bytes.as_ref() {
            // This is how content is made accessible via drag-n-drop in a browser
            self.parse_content(content)?;
        } else if let Some(path) = &file.path {
            // This is how content is accessible via drag-n-drop when the app is running natively
            log::debug!("path: {path:?}");
            if path.is_dir() {
                self.parse_directory(path)?;
            } else if is_zip_file(path) {
                #[cfg(not(target_arch = "wasm32"))]
                self.parse_zip_file(path)?;
            } else {
                self.parse_path(path)?;
            }
        } else {
            unreachable!("What is this content??")
        }
        Ok(())
    }

    // Parsing directory on native
    fn parse_directory(&mut self, path: &Path) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Err(e) = self.parse_directory(&path) {
                    log::warn!("{e}");
                }
            } else if is_zip_file(&path) {
                #[cfg(not(target_arch = "wasm32"))]
                self.parse_zip_file(&path)?;
            } else if let Err(e) = self.parse_path(&path) {
                log::warn!("{e}");
            }
        }
        Ok(())
    }

    // Parsing dropped content on web
    fn parse_content(&mut self, mut content: &[u8]) -> io::Result<()> {
        if PidLogHeaderV1::is_buf_header(content).unwrap_or(false) {
            let log = PidLogV1::from_reader(&mut content)?;
            log::debug!("Got: {}", log.descriptive_name());
            self.pid_log_v1.push(log);
        } else if StatusLogHeaderV1::is_buf_header(content).unwrap_or(false) {
            let log = StatusLogV1::from_reader(&mut content)?;
            log::debug!("Got: {}", log.descriptive_name());
            self.status_log_v1.push(log);
        } else if PidLogHeaderV2::is_buf_header(content).unwrap_or(false) {
            let log = PidLogV2::from_reader(&mut content)?;
            log::debug!("Got: {}", log.descriptive_name());
            self.pid_log_v2.push(log);
        } else if StatusLogHeaderV2::is_buf_header(content).unwrap_or(false) {
            let log = StatusLogV2::from_reader(&mut content)?;
            log::debug!("Got: {}", log.descriptive_name());
            self.status_log_v2.push(log);
        } else if GeneratorLogEntry::is_bytes_valid_generator_log_entry(content) {
            let log = GeneratorLog::from_reader(&mut content)?;
            log::debug!("Got: {}", log.descriptive_name());
            self.generator_log.push(log);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unrecognized file",
            ));
        }
        Ok(())
    }

    // Parse file on native
    fn parse_path(&mut self, path: &path::Path) -> io::Result<()> {
        if PidLogHeaderV1::file_starts_with_header(path).unwrap_or(false) {
            let f = fs::File::open(path)?;
            let log = PidLogV1::from_reader(&mut BufReader::new(f))?;
            log::debug!("Got: {}", log.descriptive_name());
            self.pid_log_v1.push(log);
        } else if StatusLogHeaderV1::file_starts_with_header(path).unwrap_or(false) {
            let f = fs::File::open(path)?;
            let log = StatusLogV1::from_reader(&mut BufReader::new(f))?;
            log::debug!("Got: {}", log.descriptive_name());
            self.status_log_v1.push(log);
        } else if PidLogHeaderV2::file_starts_with_header(path).unwrap_or(false) {
            let f = fs::File::open(path)?;
            let log = PidLogV2::from_reader(&mut BufReader::new(f))?;
            log::debug!("Got: {}", log.descriptive_name());
            self.pid_log_v2.push(log);
        } else if StatusLogHeaderV2::file_starts_with_header(path).unwrap_or(false) {
            let f = fs::File::open(path)?;
            let log = StatusLogV2::from_reader(&mut BufReader::new(f))?;
            log::debug!("Got: {}", log.descriptive_name());
            self.status_log_v2.push(log);
        } else if GeneratorLog::file_is_generator_log(path).unwrap_or(false) {
            let f = fs::File::open(path)?;
            let log = GeneratorLog::from_reader(&mut BufReader::new(f))?;
            log::debug!("Got: {}", log.descriptive_name());
            self.generator_log.push(log);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unrecognized file: {}", path.to_string_lossy()),
            ));
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn parse_zip_file(&mut self, path: &Path) -> io::Result<()> {
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            log::debug!("Parsing zipped: {}", file.name());

            if file.is_dir() {
                continue;
            }

            let mut contents = Vec::new();
            io::Read::read_to_end(&mut file, &mut contents)?;

            if let Err(e) = self.parse_content(&contents) {
                log::warn!("Failed to parse file {} in zip: {}", file.name(), e);
            }
        }
        Ok(())
    }
}

fn is_zip_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_DATA_STATUS: &str =
        "test_data/mbed_motor_control/v1/20240926_121708/status_20240926_121708_00.bin";

    const TEST_DATA_PID: &str =
        "test_data/mbed_motor_control/v1/20240926_121708/pid_20240926_121708_00.bin";

    #[test]
    fn test_supported_logs_dyn_vec() {
        let data = fs::read(TEST_DATA_STATUS).unwrap();
        let status_log = StatusLogV1::from_reader(&mut data.as_slice()).unwrap();

        let data = fs::read(TEST_DATA_PID).unwrap();
        let pidlog = PidLogV1::from_reader(&mut data.as_slice()).unwrap();

        let v: Vec<Box<dyn Plotable>> = vec![Box::new(status_log), Box::new(pidlog)];
        assert_eq!(v.len(), 2);
    }
}
