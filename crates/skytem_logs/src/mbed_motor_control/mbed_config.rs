use std::io::{self};

use byteorder::{LittleEndian, ReadBytesExt};
use getset::CopyGetters;
use serde::{Deserialize, Serialize};

#[derive(Debug, CopyGetters, PartialEq, Deserialize, Serialize, Clone, Copy)]
#[repr(packed)]
pub struct MbedConfig {
    #[getset(get_copy = "pub")]
    kp: f32,
    #[getset(get_copy = "pub")]
    ki: f32,
    #[getset(get_copy = "pub")]
    kd: f32,
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

impl MbedConfig {
    pub const fn size() -> usize {
        size_of::<Self>()
    }

    pub fn from_slice(slice: &[u8]) -> io::Result<Self> {
        if slice.len() < Self::size() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Slice is too short to contain a valid MbedConfig",
            ));
        }

        let mut cursor = io::Cursor::new(slice);
        Self::from_reader(&mut cursor)
    }

    pub fn from_reader(reader: &mut impl io::BufRead) -> io::Result<Self> {
        let kp = reader.read_f32::<LittleEndian>()?;
        let ki = reader.read_f32::<LittleEndian>()?;
        let kd = reader.read_f32::<LittleEndian>()?;
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
            kp,
            ki,
            kd,
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

    pub fn field_value_pairs(&self) -> Vec<(String, String)> {
        vec![
            ("Kp".to_owned(), self.kp().to_string()),
            ("Ki".to_owned(), self.ki().to_string()),
            ("Kd".to_owned(), self.kd().to_string()),
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
