use std::io;

use byteorder::{LittleEndian, ReadBytesExt};
use getset::CopyGetters;
use serde::{Deserialize, Serialize};

use super::MbedConfig;

#[derive(Debug, CopyGetters, PartialEq, Deserialize, Serialize, Clone, Copy)]
#[repr(packed)]
pub(crate) struct MbedConfigV2 {
    #[getset(get_copy = "pub")]
    pid_cfg: PidConfig,
    #[getset(get_copy = "pub")]
    general_cfg: GeneralConfig,
}

impl MbedConfig for MbedConfigV2 {
    fn raw_size() -> usize {
        size_of::<Self>()
    }

    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<Self> {
        let pid_cfg = PidConfig::from_reader(reader)?;
        let general_cfg = GeneralConfig::from_reader(reader)?;

        Ok(Self {
            pid_cfg,
            general_cfg,
        })
    }

    fn field_value_pairs(&self) -> Vec<(String, String)> {
        let mut fvp = self.general_cfg.field_value_pairs();
        fvp.append(&mut self.pid_cfg.field_value_pairs());
        fvp
    }
}

#[derive(Debug, CopyGetters, PartialEq, Deserialize, Serialize, Clone, Copy)]
#[repr(packed)]
pub(crate) struct GeneralConfig {
    #[getset(get_copy = "pub")]
    t_standby: u8,
    #[getset(get_copy = "pub")]
    t_run: u8,
    #[getset(get_copy = "pub")]
    t_fan_on: u8,
    #[getset(get_copy = "pub")]
    t_fan_off: u8,
    #[getset(get_copy = "pub")]
    rpm_standby: u16,
    #[getset(get_copy = "pub")]
    rpm_running: u16,

    #[getset(get_copy = "pub")]
    time_shutdown: u16,
    #[getset(get_copy = "pub")]
    time_wait_for_cap: u16,

    #[getset(get_copy = "pub")]
    vbat_ready: f32,
    #[getset(get_copy = "pub")]
    servo_min: u16,
    #[getset(get_copy = "pub")]
    servo_max: u16,
}

impl MbedConfig for GeneralConfig {
    fn raw_size() -> usize {
        size_of::<Self>()
    }

    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<Self> {
        let t_standby = reader.read_u8()?;
        let t_run = reader.read_u8()?;
        let t_fan_on = reader.read_u8()?;
        let t_fan_off = reader.read_u8()?;
        let rpm_standby = reader.read_u16::<LittleEndian>()?;
        let rpm_running = reader.read_u16::<LittleEndian>()?;
        let time_shutdown = reader.read_u16::<LittleEndian>()?;
        let time_wait_for_cap = reader.read_u16::<LittleEndian>()?;
        let vbat_ready = reader.read_f32::<LittleEndian>()?;
        let servo_min = reader.read_u16::<LittleEndian>()?;
        let servo_max = reader.read_u16::<LittleEndian>()?;
        Ok(Self {
            t_standby,
            t_run,
            t_fan_on,
            t_fan_off,
            rpm_standby,
            rpm_running,
            time_shutdown,
            time_wait_for_cap,
            vbat_ready,
            servo_min,
            servo_max,
        })
    }

    fn field_value_pairs(&self) -> Vec<(String, String)> {
        vec![
            ("T_STANDBY".to_owned(), self.t_standby.to_string()),
            ("T_RUN".to_owned(), self.t_run.to_string()),
            ("T_FAN_On".to_owned(), self.t_fan_on.to_string()),
            ("T_FAN_Off".to_owned(), self.t_fan_off.to_string()),
            ("RPM_STANDBY".to_owned(), self.rpm_standby().to_string()),
            ("RPM_RUNNING".to_owned(), self.rpm_running().to_string()),
            ("TIME_SHUTDOWN".to_owned(), self.time_shutdown().to_string()),
            (
                "TIME_WAIT_FOR_CAP".to_owned(),
                self.time_wait_for_cap().to_string(),
            ),
            ("VBAT_READY".to_owned(), self.vbat_ready().to_string()),
            ("SERVO_MIN".to_owned(), self.servo_min().to_string()),
            ("SERVO_MAX".to_owned(), self.servo_max().to_string()),
        ]
    }
}

#[derive(Debug, CopyGetters, PartialEq, Deserialize, Serialize, Clone, Copy)]
#[repr(packed)]
pub(crate) struct PidConfig {
    #[getset(get_copy = "pub")]
    kp_initial: f32,
    #[getset(get_copy = "pub")]
    ki_initial: f32,
    #[getset(get_copy = "pub")]
    kd_initial: f32,

    #[getset(get_copy = "pub")]
    kp_idle: f32,
    #[getset(get_copy = "pub")]
    ki_idle: f32,
    #[getset(get_copy = "pub")]
    kd_idle: f32,

    #[getset(get_copy = "pub")]
    kp_standby: f32,
    #[getset(get_copy = "pub")]
    ki_standby: f32,
    #[getset(get_copy = "pub")]
    kd_standby: f32,

    #[getset(get_copy = "pub")]
    kp_running: f32,
    #[getset(get_copy = "pub")]
    ki_running: f32,
    #[getset(get_copy = "pub")]
    kd_running: f32,
}

impl MbedConfig for PidConfig {
    fn raw_size() -> usize {
        size_of::<Self>()
    }

    fn from_reader(reader: &mut impl io::BufRead) -> io::Result<Self> {
        let kp_initial = reader.read_f32::<LittleEndian>()?;
        let ki_initial = reader.read_f32::<LittleEndian>()?;
        let kd_initial = reader.read_f32::<LittleEndian>()?;

        let kp_idle = reader.read_f32::<LittleEndian>()?;
        let ki_idle = reader.read_f32::<LittleEndian>()?;
        let kd_idle = reader.read_f32::<LittleEndian>()?;

        let kp_standby = reader.read_f32::<LittleEndian>()?;
        let ki_standby = reader.read_f32::<LittleEndian>()?;
        let kd_standby = reader.read_f32::<LittleEndian>()?;

        let kp_running = reader.read_f32::<LittleEndian>()?;
        let ki_running = reader.read_f32::<LittleEndian>()?;
        let kd_running = reader.read_f32::<LittleEndian>()?;

        Ok(Self {
            kp_initial,
            ki_initial,
            kd_initial,
            kp_idle,
            ki_idle,
            kd_idle,
            kp_standby,
            ki_standby,
            kd_standby,
            kp_running,
            ki_running,
            kd_running,
        })
    }

    fn field_value_pairs(&self) -> Vec<(String, String)> {
        vec![
            ("Kp_initial".to_owned(), self.kp_initial().to_string()),
            ("Ki_initial".to_owned(), self.ki_initial().to_string()),
            ("Kd_initial".to_owned(), self.kd_initial().to_string()),
            ("Kp_idle".to_owned(), self.kp_idle().to_string()),
            ("Ki_idle".to_owned(), self.ki_idle().to_string()),
            ("Kd_idle".to_owned(), self.kd_idle().to_string()),
            ("Kp_standby".to_owned(), self.kp_standby().to_string()),
            ("Ki_standby".to_owned(), self.ki_standby().to_string()),
            ("Kd_standby".to_owned(), self.kd_standby().to_string()),
            ("Kp_running".to_owned(), self.kp_running().to_string()),
            ("Ki_running".to_owned(), self.ki_running().to_string()),
            ("Kd_running".to_owned(), self.kd_running().to_string()),
        ]
    }
}
