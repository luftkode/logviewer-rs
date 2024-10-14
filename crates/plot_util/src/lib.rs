pub mod mipmap;

use egui::{Color32, Stroke};
use egui_plot::{Line, PlotBounds, PlotPoint, PlotPoints, Polygon};
use log_if::prelude::*;

pub mod plots;

pub use plots::{
    plot_data::{PlotData, PlotValues, StoredPlotLabels},
    Plots,
};

pub fn line_from_log_entry<XF, YF, L: LogEntry>(log: &[L], x_extractor: XF, y_extractor: YF) -> Line
where
    XF: Fn(&L) -> f64,
    YF: Fn(&L) -> f64,
{
    let points: PlotPoints = log
        .iter()
        .map(|e| [x_extractor(e), y_extractor(e)])
        .collect();
    Line::new(points)
}

/// An instance of a `MipMap` configuration for a given frame
#[derive(Debug, Clone, Copy)]
pub enum MipMapConfiguration {
    Enabled(Option<usize>),
    Disabled,
}

pub fn plot_lines(
    plot_ui: &mut egui_plot::PlotUi,
    plots: &mut [PlotValues],
    name_filter: &[&str],
    id_filter: &[usize],
    line_width: f32,
    mipmap_cfg: MipMapConfiguration,
    plots_width_pixels: usize,
) {
    let (x_lower, x_higher) = extended_x_plot_bound(plot_ui.plot_bounds(), 0.1);
    for plot_vals in plots
        .iter_mut()
        .filter(|p| !name_filter.contains(&p.name()) && !id_filter.contains(&p.log_id()))
    {
        match mipmap_cfg {
            MipMapConfiguration::Disabled => plot_raw(plot_ui, plot_vals, (x_lower, x_higher)),
            MipMapConfiguration::Enabled(level_option) => {
                let (level, idx_range) = match level_option {
                    Some(lvl) => (lvl, None),
                    None => plot_vals.get_scaled_mipmap_levels(
                        plots_width_pixels,
                        (x_lower as usize, x_higher as usize),
                    ),
                };

                if level == 0 {
                    plot_raw(plot_ui, plot_vals, (x_lower, x_higher));
                    continue;
                }

                let (plot_points_min, plot_points_max) = plot_vals.get_level_or_max(level);
                if plot_points_min.is_empty() {
                    continue;
                }

                let (plot_points_min, plot_points_max) = match idx_range {
                    Some((start, end)) => {
                        extract_range_points(plot_points_min, plot_points_max, start, end)
                    }
                    None => (
                        filter_plot_points(plot_points_min, (x_lower, x_higher)),
                        filter_plot_points(plot_points_max, (x_lower, x_higher)),
                    ),
                };

                plot_min_max_lines(
                    plot_ui,
                    plot_vals.label(),
                    (plot_points_min, plot_points_max),
                    line_width,
                    plot_vals.get_color(),
                );
            }
        }
    }
}

#[inline(always)]
fn extract_range_points(
    points_min: &[[f64; 2]],
    points_max: &[[f64; 2]],
    start: usize,
    end: usize,
) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
    let element_count = end - start + 2;
    let mut min_points = Vec::with_capacity(element_count);
    let mut max_points = Vec::with_capacity(element_count);

    min_points.push(points_min[0]);
    max_points.push(points_max[0]);

    min_points.extend_from_slice(&points_min[start..end]);
    max_points.extend_from_slice(&points_max[start..end]);

    if let Some(last_point) = points_min.last() {
        if min_points.last().is_some_and(|lp| lp != last_point) {
            min_points.push(*last_point);
        }
    }
    if let Some(last_point) = points_max.last() {
        if max_points.last().is_some_and(|lp| lp != last_point) {
            max_points.push(*last_point);
        }
    }

    (min_points, max_points)
}

#[inline]
fn plot_min_max_lines(
    plot_ui: &mut egui_plot::PlotUi,
    base_label: &str,
    (points_min, points_max): (Vec<[f64; 2]>, Vec<[f64; 2]>),
    line_width: f32,
    color: Color32,
) {
    let mut label_min = base_label.to_owned();
    label_min.push_str(" (min)");
    let mut label_max = base_label.to_owned();
    label_max.push_str(" (max)");

    let shaded_area = create_filled_segments_trapezoids(&points_max, &points_min, color);
    let line_min = Line::new(points_min).name(label_min).color(color);
    let line_max = Line::new(points_max).name(label_max).color(color);

    for polygon in shaded_area {
        plot_ui.add(polygon);
    }
    plot_ui.line(line_min.width(line_width));
    plot_ui.line(line_max.width(line_width));
}

fn create_filled_segments_trapezoids(
    top_line: &[[f64; 2]],
    bot_line: &[[f64; 2]],
    color: Color32,
) -> Vec<Polygon> {
    let transparent_color = color.additive();
    top_line
        .windows(2)
        .zip(bot_line.windows(2))
        .map(|(top_window, bot_window)| {
            let points = vec![top_window[0], top_window[1], bot_window[1], bot_window[0]];
            Polygon::new(PlotPoints::new(points))
                .fill_color(transparent_color)
                .allow_hover(false)
                .stroke(Stroke::new(0.0, transparent_color))
        })
        .collect()
}

fn create_filled_segments_triangles(
    top_line: &[[f64; 2]],
    bot_line: &[[f64; 2]],
    color: Color32,
) -> Vec<Polygon> {
    let transparent_color = color.additive();
    let mut triangles = Vec::new();

    for (i, (&top, &bot)) in top_line.iter().zip(bot_line.iter()).enumerate() {
        if i < top_line.len() - 2 {
            // First triangle
            let points1 = vec![top, top_line[i + 1], bot];
            triangles.push(
                Polygon::new(PlotPoints::new(points1))
                    .fill_color(color)
                    .allow_hover(false)
                    .stroke(Stroke::new(0.0, transparent_color)),
            );

            // Second triangle
            let points2 = vec![bot, top_line[i + 1], bot_line[i + 1]];
            triangles.push(
                Polygon::new(PlotPoints::new(points2))
                    .fill_color(color)
                    .allow_hover(false)
                    .stroke(Stroke::new(0.0, transparent_color)),
            );
        }
    }

    triangles
}

pub fn plot_labels(plot_ui: &mut egui_plot::PlotUi, plot_data: &PlotData, id_filter: &[usize]) {
    for plot_labels in plot_data
        .plot_labels()
        .iter()
        .filter(|pl| !id_filter.contains(&pl.log_id))
    {
        for label in plot_labels.labels() {
            let point = PlotPoint::new(label.point()[0], label.point()[1]);
            let txt = egui::RichText::new(label.text()).size(10.0);
            let txt = egui_plot::Text::new(point, txt);
            plot_ui.text(txt);
        }
    }
}

fn plot_raw(plot_ui: &mut egui_plot::PlotUi, plot_vals: &PlotValues, x_min_max_ext: (f64, f64)) {
    let plot_points = plot_vals.get_raw();
    let filtered_points = filter_plot_points(plot_points, x_min_max_ext);
    let line = Line::new(filtered_points)
        .name(plot_vals.label())
        .color(plot_vals.get_color());
    plot_ui.line(line);
}

#[inline(always)]
fn x_plot_bound(bounds: PlotBounds) -> (f64, f64) {
    let range = bounds.range_x();
    (*range.start(), *range.end())
}

/// Extends the x plot bounds by a specified percentage in both directions
#[inline]
pub fn extended_x_plot_bound(bounds: PlotBounds, extension_percentage: f64) -> (f64, f64) {
    let (x_bound_min, x_bound_max) = x_plot_bound(bounds);

    // Calculate the extension values based on the magnitude of the bounds
    let x_extension = (x_bound_max - x_bound_min).abs() * extension_percentage;

    // Extend the bounds
    let extended_x_bound_min = x_bound_min - x_extension;
    let extended_x_bound_max = x_bound_max + x_extension;

    (extended_x_bound_min, extended_x_bound_max)
}

/// Filter plot points based on the x plot bounds. Always includes the first and last plot point
/// such that resetting zooms works well even when the plot bounds are outside the data range.
pub fn filter_plot_points(points: &[[f64; 2]], x_range: (f64, f64)) -> Vec<[f64; 2]> {
    let points_len = points.len();
    // Don't bother filtering if there's less than 1024 points
    if points_len < 1024 {
        return points.to_vec();
    }

    let start_idx = points.partition_point(|point| point[0] < x_range.0);
    let end_idx = points.partition_point(|point| point[0] < x_range.1);

    let points_within = end_idx - start_idx;
    // If all the points are within the bound, return all the points
    if points_within == points_len {
        return points.to_vec();
    }
    // In this case none of the points are within the bounds so just return the first and last
    if start_idx == end_idx {
        return vec![points[0], points[points_len - 1]];
    }

    // allocate enough for the points within + 2 for the first and last points.
    // we might not end up including the first and last points if they are included in the points within
    // but this way we are sure to only allocate once
    let mut filtered = Vec::with_capacity(points_within + 2);

    // add the first points if it is not within the points that are within the bounds
    if start_idx != 0 {
        filtered.push(points[0]);
    }
    // Add all the points within the bounds
    filtered.extend_from_slice(&points[start_idx..end_idx]);

    // add the last points if it is not included in the points that are within the bounds
    if end_idx != points_len {
        filtered.push(points[points_len - 1]);
    }

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_less_than_1024_points_no_filtering() {
        let points: Vec<[f64; 2]> = (0..500).map(|i| [i as f64, i as f64 + 1.0]).collect();
        let x_range = (100.0, 300.0);

        // Since points are less than 1024, no filtering should be done
        let result = filter_plot_points(&points, x_range);

        // Result should be identical to input
        assert_eq!(result, points);
    }

    #[test]
    fn test_more_than_1024_points_with_filtering() {
        let points: Vec<[f64; 2]> = (0..1500).map(|i| [i as f64, i as f64 + 1.0]).collect();
        let x_range = (100.0, 500.0);

        // Since the points are more than 1024, filtering should happen
        let result = filter_plot_points(&points, x_range);

        // First point, range of points between start and end range, last point should be included
        let mut expected: Vec<[f64; 2]> = vec![
            // First point
            [0.0, 1.0],
        ];
        // Points within the range (100..500)
        expected.extend_from_slice(&points[100..500]);
        // Last point
        expected.push([1499.0, 1500.0]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_range_outside_bounds_with_large_data() {
        let points: Vec<[f64; 2]> = (0..1500).map(|i| [i as f64, i as f64 + 1.0]).collect();
        let x_range = (2000.0, 3000.0);

        // Since range is outside the data points, we should get first and last points
        let result = filter_plot_points(&points, x_range);

        let expected = vec![[0.0, 1.0], [1499.0, 1500.0]];

        assert_eq!(result, expected);
    }
}
