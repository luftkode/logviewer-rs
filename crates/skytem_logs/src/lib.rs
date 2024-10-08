use std::io;

use log_if::prelude::*;

pub mod generator;
pub mod mbed_motor_control;
pub mod util;

/// Parse the unique description string from a 128-byte array
///
/// A log header has a unique description, e.g. `MBED-MOTOR-CONTROL-STATUS-LOG`
/// represented by a 128 byte array of chars.
///
/// ### Note
///
/// This might only apply for binary formats or even only for the MBED binary log formats.
/// Hopefully that becomes apparent soon, and if it is, this function should be pushed down
/// to the `mbed_motor_control` module.
pub fn parse_unique_description(raw_uniq_desc: [u8; 128]) -> String {
    String::from_utf8_lossy(&raw_uniq_desc)
        .trim_end_matches(char::from(0))
        .to_owned()
}

/// Parse log entries and display them, optionally only display up to `limit` entries
///
/// This is a good way to verify by hand that your data is parsed as expected
///
/// Example
/// ```
/// use std::fs::File;
/// use std::io::{self, BufReader, ErrorKind};
/// use skytem_logs::{mbed_motor_control::pid::{header_v1::PidLogHeaderV1, entry::PidLogEntry}, parse_and_display_log_entries};
/// use crate::skytem_logs::mbed_motor_control::mbed_header::MbedMotorControlLogHeader;
///
/// fn main() -> std::io::Result<()> {
///     // Open the log file
///     let file = File::open("../../test_data/mbed_motor_control/v1/20240926_121708/pid_20240926_121708_00.bin")?;
///     let mut reader = BufReader::new(file);
///
///     // First, read the header
///     let header = PidLogHeaderV1::from_reader(&mut reader)?;
///     println!("Log Header: {:?}", header);
///
///     if !header.is_valid_header() {
///         return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Header"));
///     }
///
///     // Now parse and display the log entries
///     println!("Log Entries:");
///     parse_and_display_log_entries::<PidLogEntry, _>(&mut reader, Some(10));
///
///     Ok(())
/// }
/// ```
pub fn parse_and_display_log_entries<T: LogEntry, R: io::Read>(
    reader: &mut R,
    limit: Option<usize>,
) {
    let mut entry_count = 0;
    while let Ok(e) = T::from_reader(reader) {
        entry_count += 1;
        println!("{e}");
        if limit.is_some_and(|l| l == entry_count) {
            break;
        }
    }
}
