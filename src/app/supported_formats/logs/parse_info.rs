use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ParsedBytes(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct TotalBytes(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct ParseInfo {
    parsed_bytes: ParsedBytes,
    total_bytes: TotalBytes,
}

impl ParseInfo {
    pub fn new(parsed_bytes: ParsedBytes, total_bytes: TotalBytes) -> Self {
        let parsed = parsed_bytes.0;
        let total = total_bytes.0;

        debug_assert!(
            parsed <= total,
            "Unsound condition, parsed more than the total bytes! Parsed: {parsed}, total: {total}"
        );
        Self {
            parsed_bytes,
            total_bytes,
        }
    }

    pub fn parsed_bytes(&self) -> usize {
        self.parsed_bytes.0
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes.0
    }

    pub fn remainder_bytes(&self) -> usize {
        self.total_bytes.0 - self.parsed_bytes.0
    }
}
