use std::{io, path::Path};

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};

use hdf5::Dataset;
use log_if::prelude::*;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use stream_descriptor::StreamDescriptor;

use crate::util::{read_any_attribute_to_string, read_string_attribute};

mod stream_descriptor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BifrostLoopCurrent {
    // It does not actually contain a timestamp so we just add 1. january current year to make it slightly more convenient
    starting_timestamp_utc: DateTime<Utc>,
    dataset_description: String,
    raw_plots: Vec<RawPlot>,
    metadata: Vec<(String, String)>,
}

impl BifrostLoopCurrent {
    pub const DATASET_NAME: &str = "hm_current";
    pub const DATASET_DIMENSIONS: usize = 3;
}

impl BifrostLoopCurrent {
    /// Opens the [`BifrostCurrent`] dataset and checks the validity of the [`Dataset`] structure.
    ///
    /// # Returns
    ///
    /// The [`BifrostCurrent`] dataset as a [`Dataset`].
    ///
    /// # Errors
    ///
    /// If opening the file or any validity check fails.
    pub fn open_bifrost_current_dataset<P: AsRef<Path>>(path: P) -> io::Result<Dataset> {
        let hdf5_file = hdf5::File::open(&path)?;

        let Ok(current_data_set) = hdf5_file.dataset(Self::DATASET_NAME) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "No {dataset_name} dataset in {fname}",
                    dataset_name = Self::DATASET_NAME,
                    fname = path.as_ref().display()
                ),
            ));
        };

        if current_data_set.ndim() != Self::DATASET_DIMENSIONS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Expected {ndim} dimensions in dataset {dataset_name}",
                    dataset_name = Self::DATASET_NAME,
                    ndim = Self::DATASET_DIMENSIONS
                ),
            ));
        }

        let dataset_attributes = current_data_set.attr_names()?;

        if !dataset_attributes.contains(&"description".to_owned()) {
            let comma_separated_attr_list = dataset_attributes
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "Expected 'description' among dataset attributes, but attributes do not contain 'description'. Attributes in dataset: {comma_separated_attr_list}",
            )));
        }

        Ok(current_data_set)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let current_dataset = Self::open_bifrost_current_dataset(path)?;

        let dataset_description = read_string_attribute(&current_dataset.attr("description")?)?;
        let stream_descriptor_toml_str =
            read_string_attribute(&current_dataset.attr("stream_descriptor")?)?;
        let Ok(stream_descriptor): Result<StreamDescriptor, toml::de::Error> =
            toml::from_str(&stream_descriptor_toml_str)
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Failed decoding 'stream_descriptor' string as TOML from stream_descriptor: {stream_descriptor_toml_str}"
                    )));
        };

        for a in current_dataset.attr_names()? {
            let attr = current_dataset.attr(&a)?;
            let attr_val_as_string = read_any_attribute_to_string(&attr)?;
            eprintln!("Attr: {attr_val_as_string}");
        }

        let data_3: ndarray::Array3<f32> = current_dataset.read()?;

        let (gps_timestamps, samples_per_ts, polarities) = data_3.dim();
        log::info!("Got bifrost current dataset with: GPS timestamps: {gps_timestamps}, samples per timestamp: {samples_per_ts}, polarities: {polarities}");
        let mut metadata = vec![
            ("Dataset Description".into(), dataset_description.clone()),
            ("GPS Timestamps".into(), gps_timestamps.to_string()),
            ("Samples per timestamp".into(), samples_per_ts.to_string()),
            ("Polarities".into(), polarities.to_string()),
        ];
        metadata.extend_from_slice(&stream_descriptor.to_metadata());
        // There's no timestamp (it has to be correlated with GPS data) so we just set the time to
        // 1. january current year, hopefully it will be obvious to the user that it is an auto generated timestamp and not the real one
        let now = chrono::offset::Local::now();
        let first_january_this_year =
            NaiveDate::from_ymd_opt(now.year(), 1, 1).expect("Invalid date");
        let first_january_this_year = NaiveDateTime::new(
            first_january_this_year,
            NaiveTime::from_hms_opt(0, 0, 0).expect("Invalid time"),
        )
        .and_utc();
        let starting_timestamp_ns = first_january_this_year
            .timestamp_nanos_opt()
            .expect("timestamp as nanoseconds out of range")
            .to_f64()
            .expect("Failed converting timestamp to f64");

        let mut polarity0_currents: Vec<[f64; 2]> = Vec::new();
        let mut polarity1_currents: Vec<[f64; 2]> = Vec::new();

        let nanosec_multiplier = 1_000_000_000.0;
        // Assumes one timestamp per second
        let sample_step_size_approx: f64 = 1.0 * nanosec_multiplier
            / (samples_per_ts
                .to_f64()
                .expect("Failed converting usize to f64"));
        let mut timestamp_idx = 0;
        let mut sample_idx = 0;

        for d in data_3.rows() {
            let offset_from_start = (timestamp_idx as f64) * nanosec_multiplier
                + (sample_idx as f64) * sample_step_size_approx;

            let offset_ts = starting_timestamp_ns + offset_from_start;

            sample_idx += 1;
            if sample_idx == samples_per_ts {
                sample_idx = 0;
                timestamp_idx += 1;
            }
            if d.len() != 2 {
                log::error!(
                    "Expected bifrost hm_current row length of 2, got: {}",
                    d.len()
                );
                continue;
            }

            polarity0_currents.push([offset_ts, d[0].into()]);
            polarity1_currents.push([offset_ts, d[1].into()]);
        }

        let plot_polarity0 = RawPlot::new(
            "+ Polarity [A]".to_owned(),
            polarity0_currents.clone(),
            ExpectedPlotRange::OneToOneHundred,
        );
        let plot_polarity1 = RawPlot::new(
            "- Polarity [A]".to_owned(),
            polarity1_currents.clone(),
            ExpectedPlotRange::OneToOneHundred,
        );

        Ok(Self {
            starting_timestamp_utc: first_january_this_year,
            dataset_description,
            raw_plots: vec![plot_polarity0, plot_polarity1],
            metadata,
        })
    }
}

impl Plotable for BifrostLoopCurrent {
    fn raw_plots(&self) -> &[RawPlot] {
        &self.raw_plots
    }

    fn first_timestamp(&self) -> DateTime<Utc> {
        self.starting_timestamp_utc
    }

    fn descriptive_name(&self) -> &str {
        &self.dataset_description
    }

    fn labels(&self) -> Option<&[PlotLabels]> {
        None
    }

    fn metadata(&self) -> Option<Vec<(String, String)>> {
        Some(self.metadata.clone())
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    const TEST_DATA: &str = "../../test_data/hdf5/bifrost_current/20240930_100137_bifrost.h5";

    #[test]
    fn test_read_bifrost_current() -> TestResult {
        let bifrost_currents = BifrostLoopCurrent::from_path(TEST_DATA)?;
        assert_eq!(&bifrost_currents.dataset_description, "TX Loop Current");

        let expected_metadata = [
            ("Dataset Description".into(), "TX Loop Current".into()),
            ("GPS Timestamps".into(), "50".into()),
            ("Samples per timestamp".into(), "303".into()),
            ("Polarities".into(), "2".into()),
        ];

        for (metadata_kv, expected_metadata_kv) in bifrost_currents
            .metadata()
            .expect("Expected metadata but contained none")
            .iter()
            .zip(expected_metadata.iter())
        {
            assert_eq!(metadata_kv, expected_metadata_kv);
        }

        let plot_polarity1 = bifrost_currents.raw_plots()[1].points();

        let expected_first_value = 4.434181213378906;
        assert_eq!(plot_polarity1.first().unwrap()[1], expected_first_value);

        let expected_last_value = 17.78993797302246;
        assert_eq!(plot_polarity1.last().unwrap()[1], expected_last_value);

        Ok(())
    }
}
