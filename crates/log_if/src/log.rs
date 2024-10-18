use std::{fmt::Display, io};

use crate::{parseable::Parseable, plotable::Plotable};

/// A given log should implement this trait
pub trait SkytemLog:
    Plotable + Parseable + GitMetadata + Clone + Display + Send + Sync + Sized
{
    type Entry: LogEntry;

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
    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)>;

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
