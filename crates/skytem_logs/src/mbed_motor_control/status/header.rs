use std::{fmt, io};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use v1::StatusLogHeaderV1;
use v2::StatusLogHeaderV2;
use v3::StatusLogHeaderV3;

use crate::{
    mbed_motor_control::mbed_header::{
        MbedMotorControlLogHeader, UniqueDescriptionData, SIZEOF_UNIQ_DESC,
    },
    parse_unique_description,
};

mod v1;
mod v2;
mod v3;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum StatusLogHeader {
    V1(StatusLogHeaderV1),
    V2(StatusLogHeaderV2),
    V3(StatusLogHeaderV3),
}

impl fmt::Display for StatusLogHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V1(h) => write!(f, "{h}"),
            Self::V2(h) => write!(f, "{h}"),
            Self::V3(h) => write!(f, "{h}"),
        }
    }
}

impl StatusLogHeader {
    pub(super) fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)> {
        let mut total_bytes_read = 0;

        // Read unique description
        let mut unique_description: UniqueDescriptionData = [0u8; SIZEOF_UNIQ_DESC];
        reader.read_exact(&mut unique_description)?;
        total_bytes_read += SIZEOF_UNIQ_DESC;

        // Validate unique description
        if parse_unique_description(&unique_description) != super::UNIQUE_DESCRIPTION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not an Mbed PidLog",
            ));
        }

        // Read version
        let version = reader.read_u16::<LittleEndian>()?;
        total_bytes_read += std::mem::size_of::<u16>();

        // Match the version and read the appropriate header
        let header = match version {
            1 => {
                let (header, bytes_read) = StatusLogHeaderV1::from_reader_with_uniq_descr_version(
                    reader,
                    unique_description,
                    version,
                )?;
                total_bytes_read += bytes_read;
                Self::V1(header)
            }
            2 => {
                let (header, bytes_read) = StatusLogHeaderV2::from_reader_with_uniq_descr_version(
                    reader,
                    unique_description,
                    version,
                )?;
                total_bytes_read += bytes_read;
                Self::V2(header)
            }
            3 => {
                let (header, bytes_read) = StatusLogHeaderV3::from_reader_with_uniq_descr_version(
                    reader,
                    unique_description,
                    version,
                )?;
                total_bytes_read += bytes_read;
                Self::V3(header)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unsupported version",
                ));
            }
        };

        // Return the header and the total number of bytes read
        Ok((header, total_bytes_read))
    }
}
