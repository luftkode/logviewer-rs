use egui::DroppedFile;
use log_if::prelude::*;
use logs::{
    parse_info::{ParseInfo, ParsedBytes, TotalBytes},
    SupportedLog,
};
use serde::{Deserialize, Serialize};
use skytem_logs::{
    generator::GeneratorLog,
    mbed_motor_control::{pid::pidlog::PidLog, status::statuslog::StatusLog},
};
use std::{
    fs,
    io::{self, BufReader},
    path::Path,
};

#[cfg(feature = "hdf")]
#[cfg(not(target_arch = "wasm32"))]
mod hdf;
pub(crate) mod logs;
mod util;

/// Represents a supported format, which can be any of the supported format types.
///
/// This simply serves to encapsulate all the supported format in a single type
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(
    clippy::large_enum_variant,
    reason = "This enum is only created once when parsing an added file for the first time so optimizing memory for an instance of this enum is a waste of effort"
)]
pub enum SupportedFormat {
    Log(SupportedLog),
    #[cfg(feature = "hdf")]
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::upper_case_acronyms, reason = "The format is called HDF...")]
    HDF(hdf::SupportedHdfFormat),
}

impl From<(PidLog, ParseInfo)> for SupportedFormat {
    fn from(value: (PidLog, ParseInfo)) -> Self {
        Self::Log(SupportedLog::from(value))
    }
}

impl From<(StatusLog, ParseInfo)> for SupportedFormat {
    fn from(value: (StatusLog, ParseInfo)) -> Self {
        Self::Log(SupportedLog::from(value))
    }
}

impl From<(GeneratorLog, ParseInfo)> for SupportedFormat {
    fn from(value: (GeneratorLog, ParseInfo)) -> Self {
        Self::Log(SupportedLog::from(value))
    }
}

impl SupportedFormat {
    /// Attempts to parse a log from raw content.
    ///
    /// This is how content is made available in a browser.
    fn parse_from_content(content: &[u8]) -> io::Result<Self> {
        let total_bytes = content.len();
        log::debug!("Parsing content of length: {total_bytes}");
        let log: Self = if let Ok((pidlog, read_bytes)) = PidLog::try_from_buf(content) {
            log::debug!("Read: {read_bytes} bytes");
            (
                pidlog,
                ParseInfo::new(ParsedBytes(read_bytes), TotalBytes(total_bytes)),
            )
                .into()
        } else if let Ok((statuslog, read_bytes)) = StatusLog::try_from_buf(content) {
            (
                statuslog,
                ParseInfo::new(ParsedBytes(read_bytes), TotalBytes(read_bytes)),
            )
                .into()
        } else if let Ok((genlog, read_bytes)) = GeneratorLog::try_from_buf(content) {
            (
                genlog,
                ParseInfo::new(ParsedBytes(read_bytes), TotalBytes(total_bytes)),
            )
                .into()
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unrecognized log type",
            ));
        };
        log::debug!("Got: {}", log.descriptive_name());
        Ok(log)
    }

    /// Attempts to parse a log from a file path.
    ///
    /// This is how it is made available on native.
    fn parse_from_path(path: &Path) -> io::Result<Self> {
        let file = fs::File::open(path)?;
        let total_bytes = file.metadata()?.len() as usize;
        log::debug!("Parsing content of length: {total_bytes}");

        let mut reader = BufReader::new(file);
        let log: Self = if util::path_has_hdf_extension(path) {
            Self::parse_hdf_from_path(path)?
        } else if PidLog::file_is_valid(path) {
            let (log, parsed_bytes) = PidLog::from_reader(&mut reader)?;
            log::debug!("Read: {parsed_bytes} bytes");
            (
                log,
                ParseInfo::new(ParsedBytes(parsed_bytes), TotalBytes(total_bytes)),
            )
                .into()
        } else if StatusLog::file_is_valid(path) {
            let (log, parsed_bytes) = StatusLog::from_reader(&mut reader)?;
            (
                log,
                ParseInfo::new(ParsedBytes(parsed_bytes), TotalBytes(total_bytes)),
            )
                .into()
        } else if GeneratorLog::file_is_generator_log(path).unwrap_or(false) {
            let (log, parsed_bytes) = GeneratorLog::from_reader(&mut reader)?;
            (
                log,
                ParseInfo::new(ParsedBytes(parsed_bytes), TotalBytes(total_bytes)),
            )
                .into()
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unrecognized log type",
            ));
        };
        log::debug!("Got: {}", log.descriptive_name());
        Ok(log)
    }

    #[cfg(feature = "hdf")]
    #[cfg(not(target_arch = "wasm32"))]
    fn parse_hdf_from_path(path: &Path) -> io::Result<Self> {
        use skytem_hdf::bifrost::BifrostLoopCurrent;
        // Attempt to parse it has an hdf file
        if let Ok(bifrost_loop_current) = BifrostLoopCurrent::from_path(path) {
            Ok(Self::HDF(bifrost_loop_current.into()))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unrecognized HDF file",
            ))
        }
    }

    #[cfg(not(feature = "hdf"))]
    #[cfg(not(target_arch = "wasm32"))]
    fn parse_hdf_from_path(path: &Path) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Recognized '{}' as an HDF file. But the HDF feature is turned off.",
                path.display()
            ),
        ))
    }

    #[cfg(target_arch = "wasm32")]
    fn parse_hdf_from_path(path: &Path) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Recognized '{}' as an HDF file. HDF files are only supported on the native version", path.display()),
        ))
    }

    /// Returns [`None`] if there's no meaningful parsing information such as with HDF5 files.
    #[allow(
        clippy::unnecessary_wraps,
        reason = "HDF files are not supported on web (yet?) and the lint is triggered when compiling for web since then only logs are supported which always have parse info"
    )]
    pub fn parse_info(&self) -> Option<ParseInfo> {
        match self {
            Self::Log(l) => Some(l.parse_info()),
            #[cfg(feature = "hdf")]
            #[cfg(not(target_arch = "wasm32"))]
            Self::HDF(_) => None,
        }
    }
}

impl Plotable for SupportedFormat {
    fn raw_plots(&self) -> &[RawPlot] {
        match self {
            Self::Log(l) => l.raw_plots(),

            #[cfg(feature = "hdf")]
            #[cfg(not(target_arch = "wasm32"))]
            Self::HDF(hdf) => hdf.raw_plots(),
        }
    }

    fn first_timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::Log(l) => l.first_timestamp(),
            #[cfg(feature = "hdf")]
            #[cfg(not(target_arch = "wasm32"))]
            Self::HDF(hdf) => hdf.first_timestamp(),
        }
    }

    fn descriptive_name(&self) -> &str {
        match self {
            Self::Log(l) => l.descriptive_name(),
            #[cfg(feature = "hdf")]
            #[cfg(not(target_arch = "wasm32"))]
            Self::HDF(hdf) => hdf.descriptive_name(),
        }
    }

    fn labels(&self) -> Option<&[PlotLabels]> {
        match self {
            Self::Log(l) => l.labels(),
            #[cfg(feature = "hdf")]
            #[cfg(not(target_arch = "wasm32"))]
            Self::HDF(hdf) => hdf.labels(),
        }
    }

    fn metadata(&self) -> Option<Vec<(String, String)>> {
        match self {
            Self::Log(l) => l.metadata(),
            #[cfg(feature = "hdf")]
            #[cfg(not(target_arch = "wasm32"))]
            Self::HDF(hdf) => hdf.metadata(),
        }
    }
}

/// Contains all supported logs in a single vector.
#[derive(Default, Deserialize, Serialize)]
pub struct SupportedLogs {
    logs: Vec<SupportedFormat>,
}

impl SupportedLogs {
    /// Return a vector of immutable references to all logs
    pub fn logs(&self) -> &[SupportedFormat] {
        &self.logs
    }

    /// Take all the logs currently stored in [`SupportedLogs`] and return them as a list
    pub fn take_logs(&mut self) -> Vec<SupportedFormat> {
        self.logs.drain(..).collect()
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
            self.logs
                .push(SupportedFormat::parse_from_content(content)?);
        } else if let Some(path) = &file.path {
            if path.is_dir() {
                self.parse_directory(path)?;
            } else if is_zip_file(path) {
                #[cfg(not(target_arch = "wasm32"))]
                self.parse_zip_file(path)?;
            } else {
                self.logs.push(SupportedFormat::parse_from_path(path)?);
            }
        }
        Ok(())
    }

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
            } else {
                match SupportedFormat::parse_from_path(&path) {
                    Ok(l) => self.logs.push(l),
                    Err(e) => log::warn!("{e}"),
                }
            }
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn parse_zip_file(&mut self, path: &Path) -> io::Result<()> {
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.is_file() {
                let mut contents = Vec::new();
                io::Read::read_to_end(&mut file, &mut contents)?;
                if let Ok(log) = SupportedFormat::parse_from_content(&contents) {
                    self.logs.push(log);
                }
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
        let (status_log, _status_log_bytes_read) =
            StatusLog::from_reader(&mut data.as_slice()).unwrap();

        let data = fs::read(TEST_DATA_PID).unwrap();
        let (pidlog, _pid_log_bytes_read) = PidLog::from_reader(&mut data.as_slice()).unwrap();

        let v: Vec<Box<dyn Plotable>> = vec![Box::new(status_log), Box::new(pidlog)];
        assert_eq!(v.len(), 2);
    }
}
