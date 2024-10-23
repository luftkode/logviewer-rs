use crate::mbed_motor_control::{
    mbed_config::MbedConfigV2,
    mbed_header::{
        BuildMbedLogHeaderV2, GitBranchData, GitRepoStatusData, GitShortShaData,
        MbedMotorControlLogHeader, ProjectVersionData, StartupTimestamp, UniqueDescriptionData,
    },
};

use log_if::prelude::*;
use serde_big_array::BigArray;
use std::{fmt, io};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct StatusLogHeaderV3 {
    #[serde(with = "BigArray")]
    unique_description: UniqueDescriptionData,
    version: u16,
    project_version: ProjectVersionData,
    git_short_sha: GitShortShaData,
    #[serde(with = "BigArray")]
    git_branch: GitBranchData,
    git_repo_status: GitRepoStatusData,
    startup_timestamp: StartupTimestamp,
    mbed_config: MbedConfigV2,
}

impl StatusLogHeaderV3 {
    pub(crate) fn mbed_config(&self) -> &MbedConfigV2 {
        &self.mbed_config
    }
}

impl BuildMbedLogHeaderV2<MbedConfigV2> for StatusLogHeaderV3 {
    fn new(
        unique_description: UniqueDescriptionData,
        version: u16,
        project_version: ProjectVersionData,
        git_short_sha: GitShortShaData,
        git_branch: GitBranchData,
        git_repo_status: GitRepoStatusData,
        startup_timestamp: StartupTimestamp,
        mbed_config: MbedConfigV2,
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

impl GitMetadata for StatusLogHeaderV3 {
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

impl MbedMotorControlLogHeader for StatusLogHeaderV3 {
    const VERSION: u16 = 2;

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

impl fmt::Display for StatusLogHeaderV3 {
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
    use crate::mbed_motor_control::mbed_config::MbedConfig;

    use super::*;
    use std::fs::{self};
    use testresult::TestResult;

    const TEST_DATA: &str = "../../test_data/mbed_motor_control/v3/status_20241023_143859_00.bin";

    #[test]
    fn test_deserialize() -> TestResult {
        let data = fs::read(TEST_DATA)?;
        let (status_log_header, bytes_read) = StatusLogHeaderV3::from_reader(&mut data.as_slice())?;
        eprintln!("{status_log_header}");
        assert_eq!(bytes_read, 329);
        assert_eq!(
            status_log_header.unique_description(),
            crate::mbed_motor_control::status::UNIQUE_DESCRIPTION
        );
        assert_eq!(status_log_header.version, 3);
        assert_eq!(status_log_header.project_version().unwrap(), "3.0.0");
        assert_eq!(
            status_log_header.git_branch(),
            Some("fix-23-pid-loop-in-standby".into())
        );
        assert_eq!(status_log_header.git_short_sha(), Some("9057619".into()));
        assert_eq!(
            status_log_header.mbed_config().general_cfg().t_standby(),
            50
        );
        assert_eq!(status_log_header.mbed_config().general_cfg().t_run(), 65);
        assert_eq!(status_log_header.mbed_config().general_cfg().t_fan_on(), 81);
        assert_eq!(
            status_log_header.mbed_config().general_cfg().t_fan_off(),
            80
        );
        assert_eq!(
            status_log_header.mbed_config().general_cfg().rpm_standby(),
            3600
        );
        assert_eq!(
            status_log_header.mbed_config().general_cfg().rpm_running(),
            6000
        );
        assert_eq!(
            status_log_header
                .mbed_config()
                .general_cfg()
                .time_shutdown(),
            60
        );
        assert_eq!(
            status_log_header
                .mbed_config()
                .general_cfg()
                .time_wait_for_cap(),
            300
        );
        assert_eq!(
            status_log_header.mbed_config().general_cfg().vbat_ready(),
            12.8
        );
        assert_eq!(
            status_log_header.mbed_config().general_cfg().servo_max(),
            1500
        );
        assert_eq!(
            status_log_header.mbed_config().general_cfg().servo_min(),
            800
        );
        assert_eq!(status_log_header.mbed_config().pid_cfg().kp_initial(), 3.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().ki_initial(), 0.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().kd_initial(), 0.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().kp_idle(), 0.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().ki_idle(), 0.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().kd_idle(), 0.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().kp_standby(), 1.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().ki_standby(), 1.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().kd_standby(), 0.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().kp_running(), 3.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().ki_running(), 1.0);
        assert_eq!(status_log_header.mbed_config().pid_cfg().kd_running(), 0.0);

        eprintln!("{:?}", status_log_header.mbed_config().field_value_pairs());
        Ok(())
    }
}
