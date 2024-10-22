use std::{
    fmt, fs,
    io::{self, BufRead, BufReader},
    path::Path,
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NavSysErrLog {
    entries: Vec<NavSysErrLog>,
    timestamp_ns: Vec<f64>,
    all_plot_raw: Vec<RawPlot>,

}



// impl SkytemLog for NavSysErrLog {
//     type Entry = NavSysErrEntry
//     pub fn ...
// }


//Device Nummer
//ANG   =0; H1    =2; H2    =3; GPS1  =4; GPS2  =5; TX    =6; PaPc  =8; TIB   =9; TxCoilMove=10;  MA=11;
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Copy)]
pub struct NavSysErrEntry {
    pub timestamp: NaiveDateTime,
    pub device_number: i32,
    pub error_code: i32,
    pub error_string: String
}

impl FromStr for NavSysErrEntry {
    type Entry = NavSysErrEntry;

    fn from_str(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.split_whitespace().collect();

        // Parse timestamp
        let timestamp_str = format!(
            "{} {} {} {} {} {} {}",
            parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], parts[6]
        );

        let timestamp =
            NaiveDateTime::parse_from_str(&timestamp_str, "%Y %m %d %H %M %S %3f")?.and_utc();

        let device_number = parts[7];

        let error_code = parts[8];

        let error_string = parts



    }





            // Parse timestamp
            let timestamp_str = format!(
                "{} {} {} {} {} {} {}",
                parts[1], parts[2], parts[3], parts[4], parts[5], parts[6], parts[7]
            );
            let timestamp =
                NaiveDateTime::parse_from_str(&timestamp_str, "%Y %m %d %H %M %S %3f")?.and_utc();
