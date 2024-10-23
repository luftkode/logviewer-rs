use std::io::{self};

pub(crate) mod v1;
pub(crate) use v1::MbedConfigV1;
pub(crate) mod v2;
pub(crate) use v2::MbedConfigV2;

pub(crate) trait MbedConfig: Sized {
    /// As long as implementors are packed structs, this can just return `size_of::<Self>()`, otherwise it should return
    /// the combined memory footprint of all members as if they were packed
    fn raw_size() -> usize;

    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<Self>;

    fn field_value_pairs(&self) -> Vec<(String, String)>;
}
