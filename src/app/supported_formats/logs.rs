use log_if::prelude::*;
use parse_info::ParseInfo;
use serde::{Deserialize, Serialize};
use skytem_logs::{
    generator::GeneratorLog,
    mbed_motor_control::{pid::pidlog::PidLog, status::statuslog::StatusLog},
};

pub(crate) mod parse_info;

/// Represents a supported log format, which can be any of the supported log format types.
///
/// This simply serves to encapsulate all the supported log format in a single type
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SupportedLog {
    MbedPid(PidLog, ParseInfo),
    MbedStatus(StatusLog, ParseInfo),
    Generator(GeneratorLog, ParseInfo),
}

impl SupportedLog {
    pub(crate) fn parse_info(&self) -> ParseInfo {
        match self {
            Self::MbedPid(_, parse_info)
            | Self::MbedStatus(_, parse_info)
            | Self::Generator(_, parse_info) => *parse_info,
        }
    }
}

impl From<(PidLog, ParseInfo)> for SupportedLog {
    fn from(value: (PidLog, ParseInfo)) -> Self {
        Self::MbedPid(value.0, value.1)
    }
}

impl From<(StatusLog, ParseInfo)> for SupportedLog {
    fn from(value: (StatusLog, ParseInfo)) -> Self {
        Self::MbedStatus(value.0, value.1)
    }
}

impl From<(GeneratorLog, ParseInfo)> for SupportedLog {
    fn from(value: (GeneratorLog, ParseInfo)) -> Self {
        Self::Generator(value.0, value.1)
    }
}

impl Plotable for SupportedLog {
    fn raw_plots(&self) -> &[RawPlot] {
        match self {
            Self::MbedPid(l, _) => l.raw_plots(),
            Self::MbedStatus(l, _) => l.raw_plots(),
            Self::Generator(l, _) => l.raw_plots(),
        }
    }

    fn first_timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::MbedPid(l, _) => l.first_timestamp(),
            Self::MbedStatus(l, _) => l.first_timestamp(),
            Self::Generator(l, _) => l.first_timestamp(),
        }
    }

    fn descriptive_name(&self) -> &str {
        match self {
            Self::MbedPid(l, _) => l.descriptive_name(),
            Self::MbedStatus(l, _) => l.descriptive_name(),
            Self::Generator(l, _) => l.descriptive_name(),
        }
    }

    fn labels(&self) -> Option<&[PlotLabels]> {
        match self {
            Self::MbedPid(l, _) => l.labels(),
            Self::MbedStatus(l, _) => l.labels(),
            Self::Generator(l, _) => l.labels(),
        }
    }

    fn metadata(&self) -> Option<Vec<(String, String)>> {
        match self {
            Self::MbedPid(l, _) => l.metadata(),
            Self::MbedStatus(l, _) => l.metadata(),
            Self::Generator(l, _) => l.metadata(),
        }
    }
}
