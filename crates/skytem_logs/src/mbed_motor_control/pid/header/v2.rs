use std::{fmt, io};

use crate::mbed_motor_control::{
    mbed_config::MbedConfig,
    mbed_header::{
        BuildMbedLogHeaderV2, GitBranchData, GitRepoStatusData, GitShortShaData,
        MbedMotorControlLogHeader, ProjectVersionData, StartupTimestamp, UniqueDescriptionData,
        SIZEOF_GIT_BRANCH, SIZEOF_GIT_REPO_STATUS, SIZEOF_GIT_SHORT_SHA, SIZEOF_PROJECT_VERSION,
        SIZEOF_STARTUP_TIMESTAMP, SIZEOF_UNIQ_DESC,
    },
};

use log_if::log::GitMetadata;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub(crate) struct PidLogHeaderV2 {
    #[serde(with = "BigArray")]
    unique_description: UniqueDescriptionData,
    version: u16,
    project_version: ProjectVersionData,
    git_short_sha: GitShortShaData,
    #[serde(with = "BigArray")]
    git_branch: GitBranchData,
    git_repo_status: GitRepoStatusData,
    startup_timestamp: StartupTimestamp,
    mbed_config: MbedConfig,
}

impl PidLogHeaderV2 {
    pub fn mbed_config(&self) -> &MbedConfig {
        &self.mbed_config
    }
}

impl BuildMbedLogHeaderV2 for PidLogHeaderV2 {
    fn new(
        unique_description: UniqueDescriptionData,
        version: u16,
        project_version: ProjectVersionData,
        git_short_sha: GitShortShaData,
        git_branch: GitBranchData,
        git_repo_status: GitRepoStatusData,
        startup_timestamp: StartupTimestamp,
        mbed_config: MbedConfig,
    ) -> Self {
        Self {
            unique_description,
            version,
            project_version,
            git_short_sha,
            git_branch,
            git_repo_status,
            startup_timestamp,
            mbed_config,
        }
    }
}

impl GitMetadata for PidLogHeaderV2 {
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

impl MbedMotorControlLogHeader for PidLogHeaderV2 {
    const VERSION: u16 = 2;
    /// Size of the header type in bytes if represented in raw binary
    const RAW_SIZE: usize = SIZEOF_UNIQ_DESC
        + SIZEOF_PROJECT_VERSION
        + SIZEOF_GIT_SHORT_SHA
        + SIZEOF_GIT_BRANCH
        + SIZEOF_GIT_REPO_STATUS
        + SIZEOF_STARTUP_TIMESTAMP
        + MbedConfig::size();

    fn unique_description_bytes(&self) -> &UniqueDescriptionData {
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
        reader: &mut impl io::BufRead,
        unique_description: UniqueDescriptionData,
        version: u16,
    ) -> io::Result<(Self, usize)> {
        Self::build_from_reader_with_uniq_descr_version(reader, unique_description, version)
    }
}

impl fmt::Display for PidLogHeaderV2 {
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
    use std::fs;

    const TEST_DATA: &str =
        "../../test_data/mbed_motor_control/v2/20241014_080729/pid_20241014_080729_00.bin";

    use io::Read;
    use testresult::TestResult;

    use super::*;

    #[test]
    fn test_deserialize() -> TestResult {
        let mut file = fs::File::open(TEST_DATA)?;
        let mut buffer = vec![0; 500];
        file.read_exact(&mut buffer)?;
        let (pid_log_header, bytes_read) = PidLogHeaderV2::from_reader(&mut buffer.as_slice())?;
        eprintln!("{pid_log_header}");
        assert_eq!(bytes_read, 293);
        assert_eq!(
            pid_log_header.unique_description(),
            crate::mbed_motor_control::pid::UNIQUE_DESCRIPTION
        );
        assert_eq!(pid_log_header.version, 2);
        assert_eq!(pid_log_header.project_version().unwrap(), "2.3.2");
        assert_eq!(pid_log_header.git_branch(), None);
        assert_eq!(pid_log_header.git_short_sha(), None);
        assert_eq!(pid_log_header.mbed_config().kp(), 3.0);
        assert_eq!(pid_log_header.mbed_config().ki(), 1.0);
        assert_eq!(pid_log_header.mbed_config().kd(), 0.0);
        assert_eq!(pid_log_header.mbed_config().t_standby(), 50);
        assert_eq!(pid_log_header.mbed_config().t_run(), 65);
        assert_eq!(pid_log_header.mbed_config().t_fan_on(), 81);
        assert_eq!(pid_log_header.mbed_config().t_fan_off(), 80);
        assert_eq!(pid_log_header.mbed_config().rpm_standby(), 3600);
        assert_eq!(pid_log_header.mbed_config().rpm_running(), 6000);
        assert_eq!(pid_log_header.mbed_config().time_shutdown(), 60);
        assert_eq!(pid_log_header.mbed_config().time_wait_for_cap(), 300);
        assert_eq!(pid_log_header.mbed_config().vbat_ready(), 12.8);
        assert_eq!(pid_log_header.mbed_config().servo_max(), 1583);
        assert_eq!(pid_log_header.mbed_config().servo_min(), 765);
        Ok(())
    }
}
