use std::io::{self, BufReader};

/// The interface that a supported format has to implement to be able to load it from a file or any buffer.
pub trait Parseable: Sized {
    /// A descriptive name of what the implementer of [`Parseable`] is. e.g. an "Mbed Pid Log".
    /// Used for adding context to error messages if parsing fails
    const DESCRIPTIVE_NAME: &str;

    /// Attempt to serialize a [`Parseable`] from a buffer. When a user loads a file, this function is called
    /// on the input file contents for each supported format until either:
    ///
    /// - A supported format successfully parses the buffer
    /// - All supported formats fail to parse the buffer
    fn try_from_buf(buf: &[u8]) -> io::Result<(Self, usize)> {
        if !Self::is_buf_valid(buf) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Buffer is not a valid '{}'", Self::DESCRIPTIVE_NAME),
            ));
        }
        let mut reader = BufReader::new(buf);
        Self::from_reader(&mut reader)
    }

    /// Create an instance of a [`Parseable`] from a reader ([`io::BufRead`]) and return
    /// the instance along with the number of bytes read
    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)>;

    /// Returns whether or not the buffer is a valid instance of [`Self`]
    ///
    /// Implementers should read and verify the buffer until there's no doubt that this is
    /// a buffer of [`Self`] content, e.g. an `Mbed PID Log`. If several supported formats
    /// returns true for a given buffer, there's no way to know how to parse it.
    ///
    /// Performance is not a concern here as we are only doing this check once per file
    /// and only when the file is loaded, it is much more important that the check is
    /// extremely rigorous.
    fn is_buf_valid(buf: &[u8]) -> bool;
}
