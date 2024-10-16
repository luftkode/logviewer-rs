use std::{fmt::Display, io};

use crate::plotable::Plotable;

/// A given log should implement this trait
pub trait SkytemLog: Plotable + Clone + Display + Send + Sync + Sized + GitMetadata {
    type Entry: LogEntry;
    /// Create a [`SkytemLog`] instance from a reader and return the log along with the number of bytes read
    fn from_reader(reader: &mut impl io::Read) -> io::Result<(Self, usize)>;

    /// Return a borrowed slice (list) of log entries
    fn entries(&self) -> &[Self::Entry];
}

/// A given log entry should implement this trait
pub trait LogEntry: Sized + Display + Send + Sync {
    /// Create a [`LogEntry`] instance from a reader
    ///
    /// Returns a tuple containing:
    /// - The created `LogEntry` instance
    /// - The number of bytes consumed from the reader
    fn from_reader(reader: &mut impl io::Read) -> io::Result<(Self, usize)>;

    /// Timestamp in nanoseconds
    fn timestamp_ns(&self) -> f64;
}

/// A given log header should implement this
///
/// If it does not, it returns [`None`] but it really should!
pub trait GitMetadata {
    fn project_version(&self) -> Option<String>;
    fn git_short_sha(&self) -> Option<String>;
    fn git_branch(&self) -> Option<String>;
    fn git_repo_status(&self) -> Option<String>;
}
