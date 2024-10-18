use crate::mbed_motor_control::mbed_header::BuildMbedLogHeaderV1;
use crate::mbed_motor_control::mbed_header::{
    GitBranchData, GitRepoStatusData, GitShortShaData, MbedMotorControlLogHeader,
    ProjectVersionData, StartupTimestamp, UniqueDescriptionData, SIZEOF_GIT_BRANCH,
    SIZEOF_GIT_REPO_STATUS, SIZEOF_GIT_SHORT_SHA, SIZEOF_PROJECT_VERSION, SIZEOF_STARTUP_TIMESTAMP,
    SIZEOF_UNIQ_DESC,
};

use log_if::prelude::*;
use serde_big_array::BigArray;
use std::{
    fmt,
    io::{self, Read},
};

#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct StatusLogHeaderV1 {
    #[serde(with = "BigArray")]
    unique_description: UniqueDescriptionData,
    version: u16,
    project_version: ProjectVersionData,
    git_short_sha: GitShortShaData,
    #[serde(with = "BigArray")]
    git_branch: GitBranchData,
    git_repo_status: GitRepoStatusData,
    startup_timestamp: StartupTimestamp,
}

impl GitMetadata for StatusLogHeaderV1 {
    fn project_version(&self) -> Option<String> {
        Some(
            String::from_utf8_lossy(self.project_version_raw())
                .trim_end_matches(char::from(0))
                .to_owned(),
        )
    }
    fn git_branch(&self) -> Option<String> {
        let git_branch_info = String::from_utf8_lossy(self.git_branch_raw())
            .trim_end_matches(char::from(0))
            .to_owned();
        if git_branch_info.is_empty() {
            None
        } else {
            Some(git_branch_info)
        }
    }

    fn git_repo_status(&self) -> Option<String> {
        let repo_status = String::from_utf8_lossy(self.git_repo_status_raw())
            .trim_end_matches(char::from(0))
            .to_owned();
        if repo_status.is_empty() {
            None
        } else {
            Some(repo_status)
        }
    }

    fn git_short_sha(&self) -> Option<String> {
        let short_sha = String::from_utf8_lossy(self.git_short_sha_raw())
            .trim_end_matches(char::from(0))
            .to_owned();
        if short_sha.is_empty() {
            None
        } else {
            Some(short_sha)
        }
    }
}

impl BuildMbedLogHeaderV1 for StatusLogHeaderV1 {
    fn new(
        unique_description: UniqueDescriptionData,
        version: u16,
        project_version: ProjectVersionData,
        git_short_sha: GitShortShaData,
        git_branch: GitBranchData,
        git_repo_status: GitRepoStatusData,
        startup_timestamp: StartupTimestamp,
    ) -> Self {
        Self {
            unique_description,
            version,
            project_version,
            git_short_sha,
            git_branch,
            git_repo_status,
            startup_timestamp,
        }
    }
}

impl MbedMotorControlLogHeader for StatusLogHeaderV1 {
    const RAW_SIZE: usize = SIZEOF_UNIQ_DESC
        + SIZEOF_PROJECT_VERSION
        + SIZEOF_GIT_SHORT_SHA
        + SIZEOF_GIT_BRANCH
        + SIZEOF_GIT_REPO_STATUS
        + SIZEOF_STARTUP_TIMESTAMP;
    const VERSION: u16 = 1;

    fn unique_description_bytes(&self) -> &[u8; 128] {
        &self.unique_description
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn project_version_raw(&self) -> &ProjectVersionData {
        &self.project_version
    }

    fn git_short_sha_raw(&self) -> &GitShortShaData {
        &self.git_short_sha
    }

    fn git_branch_raw(&self) -> &GitBranchData {
        &self.git_branch
    }

    fn git_repo_status_raw(&self) -> &GitRepoStatusData {
        &self.git_repo_status
    }

    fn startup_timestamp_raw(&self) -> &StartupTimestamp {
        &self.startup_timestamp
    }

    /// Deserialize a header for a `reader` that implements [Read]
    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<(Self, usize)> {
        Self::build_from_reader(reader)
    }

    fn from_reader_with_uniq_descr_version(
        reader: &mut impl Read,
        unique_description: UniqueDescriptionData,
        version: u16,
    ) -> io::Result<(Self, usize)> {
        Self::build_from_reader_with_uniq_descr_version(reader, unique_description, version)
    }
}

impl fmt::Display for StatusLogHeaderV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}-v{}", self.unique_description(), self.version)?;
        writeln!(
            f,
            "Project Version: {}",
            self.project_version()
                .unwrap_or_else(|| "<Missing>".to_owned())
        )?;
        if let Some(git_branch) = self.git_branch() {
            writeln!(f, "Branch: {git_branch}")?;
        }
        if let Some(git_short_sha) = self.git_short_sha() {
            writeln!(f, "SHA: {git_short_sha}")?;
        }
        if self.git_repo_status().is_some() {
            writeln!(f, "Repo status: dirty")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self};
    use testresult::TestResult;

    const TEST_DATA: &str =
        "../../test_data/mbed_motor_control/v1/20240926_121708/status_20240926_121708_00.bin";

    #[test]
    fn test_deserialize() -> TestResult {
        let data = fs::read(TEST_DATA)?;
        let (status_log_header, bytes_read) = StatusLogHeaderV1::from_reader(&mut data.as_slice())?;
        eprintln!("{status_log_header}");
        assert_eq!(bytes_read, 261);
        assert_eq!(
            status_log_header.unique_description(),
            crate::mbed_motor_control::status::UNIQUE_DESCRIPTION
        );
        assert_eq!(status_log_header.version, 1);
        assert_eq!(status_log_header.project_version().unwrap(), "1.3.0");
        assert_eq!(
            status_log_header.git_branch().unwrap(),
            "fix-23-pid-loop-in-standby"
        );
        assert_eq!(status_log_header.git_short_sha().unwrap(), "fe6e412");
        Ok(())
    }
}
