use std::time::Duration;

/// Parse a timestamp in milliseconds to a timestamp string on the form `HH:MM:SS.zzz` where `zzz` is the millisecond fraction.
pub fn parse_timestamp(timestamp_ms: u32) -> String {
    let duration = Duration::from_millis(timestamp_ms as u64);
    let hours = (duration.as_secs() % 86400) / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    let milliseconds = duration.subsec_millis();

    format!("{hours:02}:{minutes:02}:{seconds:02}.{milliseconds:03}")
}