use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub trait Plotable {
    /// Returns a slice of all the plottable data.
    fn raw_plots(&self) -> &[RawPlot];
    /// Return the first timestamp, meaning the timestamp of the first entry
    fn first_timestamp(&self) -> DateTime<Utc>;
    /// A name that describes the type of plotable data to the user (e.g. "Mbed PID log")
    fn descriptive_name(&self) -> &str;
    /// Return all labels (if any) that should be shown on the plot(s)
    fn labels(&self) -> Option<&[PlotLabels]>;
    /// Returns metadata if any, as a list of key/values
    fn metadata(&self) -> Option<Vec<(String, String)>>;
}

/// Implement conversion from a type that implements [`Plotable`] to a generic dynamic [`Plotable`] type
///
/// Which allows e.g. building vectors of various types that implement [`Plotable`] and performing the type conversion by simply calling `.into()` on `T`
impl<T> From<T> for Box<dyn Plotable>
where
    T: Plotable + 'static,
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

/// Where does the plot values typically fit within, e.g. RPM measurements will probably be in the thousands, while a duty cycle will be in percentage.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Copy)]
pub enum ExpectedPlotRange {
    /// For plots where the value is 0.0-1.0 and corresponds to percentage 0-100%
    Percentage,
    OneToOneHundred,
    Thousands,
}

/// [`RawPlot`] represents some plottable data from a log, e.g. RPM measurements
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RawPlot {
    name: String,
    points: Vec<[f64; 2]>,
    expected_range: ExpectedPlotRange,
}

impl RawPlot {
    pub fn new(name: String, points: Vec<[f64; 2]>, expected_range: ExpectedPlotRange) -> Self {
        Self {
            name,
            points,
            expected_range,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn points(&self) -> &[[f64; 2]] {
        &self.points
    }
    pub fn expected_range(&self) -> ExpectedPlotRange {
        self.expected_range
    }
}

/// [`PlotLabel`] represents some text label that should be displayed in the plot
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PlotLabels {
    label_points: Vec<([f64; 2], String)>,
    expected_range: ExpectedPlotRange,
}

impl PlotLabels {
    pub fn new(label_points: Vec<([f64; 2], String)>, expected_range: ExpectedPlotRange) -> Self {
        Self {
            label_points,
            expected_range,
        }
    }

    pub fn label_points(&self) -> &[([f64; 2], String)] {
        &self.label_points
    }

    pub fn expected_range(&self) -> ExpectedPlotRange {
        self.expected_range
    }
}
