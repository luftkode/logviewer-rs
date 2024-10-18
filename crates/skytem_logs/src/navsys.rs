use std::{
    fmt,
    io::{self, BufReader},
};

use chrono::{DateTime, Utc};
use entries::NavSysSpsEntry;
use header::NavSysSpsHeader;
use log_if::{parseable::Parseable, prelude::*};
use serde::{Deserialize, Serialize};

mod entries;
mod header;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NavSysSps {
    header: NavSysSpsHeader,
    entries: Vec<NavSysSpsEntry>,
}

impl fmt::Display for NavSysSps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.header)?;
        for e in &self.entries {
            writeln!(f, "{e}")?;
        }
        Ok(())
    }
}

impl Plotable for NavSysSps {
    fn raw_plots(&self) -> &[RawPlot] {
        todo!()
    }

    fn first_timestamp(&self) -> DateTime<Utc> {
        self.header.first_timestamp()
    }

    fn descriptive_name(&self) -> &str {
        Self::DESCRIPTIVE_NAME
    }

    fn labels(&self) -> Option<&[PlotLabels]> {
        None
    }

    fn metadata(&self) -> Option<Vec<(String, String)>> {
        let mut metadata: Vec<(String, String)> = vec![
            ("Version".into(), self.header.version().to_string()),
            (
                "NavSys Software rev.".into(),
                self.header.software_revision().to_owned(),
            ),
            (
                "TiltSensor ID".into(),
                self.header.tilt_sensor_id().to_owned(),
            ),
        ];

        todo!()
    }
}

impl SkytemLog for NavSysSps {
    type Entry = NavSysSpsEntry;

    fn entries(&self) -> &[Self::Entry] {
        &self.entries
    }
}

impl Parseable for NavSysSps {
    const DESCRIPTIVE_NAME: &str = "NavSys Sps";

    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)> {
        let mut total_bytes_read = 0;
        let (header, bytes_read) = NavSysSpsHeader::from_reader(reader)?;
        total_bytes_read += bytes_read;

        let (entries, bytes_read) = parse_to_vec(reader);
        total_bytes_read += bytes_read;

        Ok((Self { header, entries }, total_bytes_read))
    }

    fn is_buf_valid(buf: &[u8]) -> bool {
        let mut reader = BufReader::new(buf);
        NavSysSpsHeader::from_reader(&mut reader).is_ok()
    }
}

impl GitMetadata for NavSysSps {
    fn project_version(&self) -> Option<String> {
        Some(self.header.software_revision().to_owned())
    }

    fn git_short_sha(&self) -> Option<String> {
        None
    }

    fn git_branch(&self) -> Option<String> {
        None
    }

    fn git_repo_status(&self) -> Option<String> {
        None
    }
}
