use crate::prelude::*;
use std::io;

/// Take a reader and parse [`LogEntry`]s from it until it returns an error,
/// then return a vector of all [`LogEntry`]s and the total number of bytes read from the reader.
pub fn parse_to_vec<T: LogEntry>(reader: &mut impl io::BufRead) -> (Vec<T>, usize) {
    let mut entries = Vec::new();
    let mut total_bytes_read = 0;

    loop {
        match T::from_reader(reader) {
            Ok((entry, bytes_read)) => {
                entries.push(entry);
                total_bytes_read += bytes_read;
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    break;
                } else {
                    log::warn!("Failed parsing log entry: {e}");
                    break;
                }
            }
        }
    }

    (entries, total_bytes_read)
}

/// Utility function for converting a slice of [`LogEntry`] to plot points by supplying extractor functions
/// detailing how to extract the timestamp (X) and the data (Y) from [`LogEntry`]s.
pub fn plot_points_from_log_entry<XF, YF, L: LogEntry>(
    log: &[L],
    x_extractor: XF,
    y_extractor: YF,
) -> Vec<[f64; 2]>
where
    XF: Fn(&L) -> f64,
    YF: Fn(&L) -> f64,
{
    log.iter()
        .map(|e| [x_extractor(e), y_extractor(e)])
        .collect()
}
