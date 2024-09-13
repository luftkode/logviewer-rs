use std::{ops::RangeInclusive, time::Duration};

use crate::{
    app::PlayBackButtonEvent,
    logs::{
        generator::GeneratorLog,
        mbed_motor_control::{pid::PidLog, status::StatusLog},
    },
};
use chrono::{DateTime, Timelike};
use egui::{Color32, Response, RichText};
use egui_plot::{AxisHints, GridMark, HPlacement, Legend, Line, Plot, PlotPoint, Text, VPlacement};
use play_state::PlayState;
use util::{ExpectedPlotRange, PlotWithName};

mod play_state;
pub mod util;

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
struct AxisConfig {
    link_x: bool,
    link_cursor_x: bool,
    show_axes: bool,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            link_x: true,
            link_cursor_x: true,
            show_axes: true,
        }
    }
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LogPlot {
    config: Legend,
    line_width: f32,
    axis_config: AxisConfig,
    play_state: PlayState,
    show_percentage_plot: bool,
    show_to_hundreds_plot: bool,
    show_to_thousands_plot: bool,
}

impl Default for LogPlot {
    fn default() -> Self {
        Self {
            config: Default::default(),
            line_width: 1.5,
            axis_config: Default::default(),
            play_state: PlayState::default(),
            show_percentage_plot: true,
            show_to_hundreds_plot: true,
            show_to_thousands_plot: true,
        }
    }
}

impl LogPlot {
    pub fn formatted_playback_time(&self) -> String {
        self.play_state.formatted_time()
    }
    pub fn is_playing(&self) -> bool {
        self.play_state.is_playing()
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        pid_log: Option<&PidLog>,
        status_log: Option<&StatusLog>,
        generator_log: Option<&GeneratorLog>,
    ) -> Response {
        let Self {
            config,
            line_width,
            axis_config: _,
            play_state,
            show_percentage_plot,
            show_to_hundreds_plot,
            show_to_thousands_plot,
        } = self;

        let mut playback_button_event = None;

        egui::Grid::new("settings").show(ui, |ui| {
            ui.label("Line width");
            ui.add(
                egui::DragValue::new(line_width)
                    .speed(0.02)
                    .range(0.5..=20.0),
            );
            ui.horizontal_top(|ui| {
                ui.toggle_value(&mut self.axis_config.link_x, "Linked Axes");
                ui.toggle_value(&mut self.axis_config.link_cursor_x, "Linked Cursors");
                ui.toggle_value(&mut self.axis_config.show_axes, "Show Axes");
                ui.label("|");
                ui.toggle_value(show_percentage_plot, "Show % plot");
                ui.toggle_value(show_to_hundreds_plot, "Show 0-100 plot");
                ui.toggle_value(show_to_thousands_plot, "Show 0-1000 plot");
            });

            ui.horizontal_centered(|ui| {
                ui.label("| ");
                // Reset button
                let reset_text = RichText::new(egui_phosphor::regular::REWIND);
                if ui.button(reset_text).clicked() {
                    playback_button_event = Some(PlayBackButtonEvent::Reset);
                }
                let playpause_text = if play_state.is_playing() {
                    RichText::new(egui_phosphor::regular::PAUSE).color(Color32::YELLOW)
                } else {
                    RichText::new(egui_phosphor::regular::PLAY).color(Color32::GREEN)
                };
                if ui.button(playpause_text).clicked() {
                    playback_button_event = Some(PlayBackButtonEvent::PlayPause);
                }

                ui.label(RichText::new(play_state.formatted_time()));
                ui.label(" |");
            });

            ui.end_row();
        });
        if let Some(e) = playback_button_event {
            play_state.handle_playback_button_press(e);
        };
        let is_reset_pressed = matches!(playback_button_event, Some(PlayBackButtonEvent::Reset));
        let timer = play_state.time_since_update();

        let link_group_id = ui.id().with("linked_plots");

        ui.vertical(|ui| {
            let mut percentage_plots: Vec<PlotWithName> = vec![];
            let mut to_hundred_plots: Vec<PlotWithName> = vec![];
            let mut thousands_plots: Vec<PlotWithName> = vec![];

            if let Some(pid_log) = pid_log {
                for (points, name, range) in pid_log.all_plots_raw().iter() {
                    match range {
                        ExpectedPlotRange::Percentage => {
                            percentage_plots.push(PlotWithName::new(points.clone(), name.clone()))
                        }
                        ExpectedPlotRange::OneToOneHundred => {
                            to_hundred_plots.push(PlotWithName::new(points.clone(), name.clone()))
                        }
                        ExpectedPlotRange::Thousands => {
                            thousands_plots.push(PlotWithName::new(points.clone(), name.clone()))
                        }
                    }
                }
            }
            if let Some(status_log) = status_log {
                for (points, name, range) in status_log.all_plots_raw().iter() {
                    match range {
                        ExpectedPlotRange::Percentage => {
                            percentage_plots.push(PlotWithName::new(points.clone(), name.clone()))
                        }
                        ExpectedPlotRange::OneToOneHundred => {
                            to_hundred_plots.push(PlotWithName::new(points.clone(), name.clone()))
                        }
                        ExpectedPlotRange::Thousands => {
                            thousands_plots.push(PlotWithName::new(points.clone(), name.clone()))
                        }
                    }
                }
            }
            let mut total_plot_count: u8 = 0;
            let display_percentage_plot = !percentage_plots.is_empty() && *show_percentage_plot;
            total_plot_count += display_percentage_plot as u8;
            let display_to_hundred_plot = !to_hundred_plots.is_empty() && *show_to_hundreds_plot;
            total_plot_count += display_to_hundred_plot as u8;
            let display_to_thousands_plot = !thousands_plots.is_empty() && *show_to_thousands_plot;
            total_plot_count += display_to_thousands_plot as u8;

            if generator_log.is_some() {
                total_plot_count += 1;
            }

            let plot_height = ui.available_height() / (total_plot_count as f32);

            // Function to format milliseconds into HH:MM.ms
            let format_time = |x: f64| {
                let duration = Duration::from_millis(x as u64);
                let hours = duration.as_secs() / 3600;
                let minutes = (duration.as_secs() % 3600) / 60;
                let seconds = duration.as_secs() % 60;

                format!("{:1}:{:02}:{:02}.{x:03}", hours, minutes, seconds)
            };

            let percentage_plot = Plot::new("percentage_plot")
                .legend(config.clone())
                .height(plot_height)
                .show_axes(self.axis_config.show_axes)
                .x_axis_position(VPlacement::Top)
                .y_axis_position(HPlacement::Right)
                .include_y(0.0) // Force Y-axis to include 0%
                .include_y(1.0) // Force Y-axis to include 100%
                .y_axis_formatter(|y, _range| format!("{:.0}%", y.value * 100.0))
                .x_axis_formatter(move |x, _range| format_time(x.value))
                .link_axis(link_group_id, self.axis_config.link_x, false)
                .link_cursor(link_group_id, self.axis_config.link_cursor_x, false);

            let to_hundred = Plot::new("to_hundreds")
                .legend(config.clone())
                .height(plot_height)
                .include_y(0.0) // Force Y-axis to include 0
                .show_axes(self.axis_config.show_axes)
                .y_axis_position(HPlacement::Right)
                .x_axis_formatter(move |x, _range| format_time(x.value))
                .link_axis(link_group_id, self.axis_config.link_x, false)
                .link_cursor(link_group_id, self.axis_config.link_cursor_x, false);

            let thousands = Plot::new("thousands")
                .legend(config.clone())
                .height(plot_height)
                .show_axes(self.axis_config.show_axes)
                .y_axis_position(HPlacement::Right)
                .include_y(0.0) // Force Y-axis to include 0
                .x_axis_formatter(move |x, _range| format_time(x.value))
                .link_axis(link_group_id, self.axis_config.link_x, false)
                .link_cursor(link_group_id, self.axis_config.link_cursor_x, false);

            if display_percentage_plot {
                percentage_plot.show(ui, |plot_ui| {
                    if let Some(status_log) = status_log {
                        for (ts, st_change) in status_log.timestamps_with_state_changes() {
                            plot_ui.text(Text::new(
                                PlotPoint::new(*ts as f64, ((*st_change as u8) as f64) / 10.0),
                                st_change.to_string(),
                            ))
                        }
                    }
                    for plot_with_name in percentage_plots {
                        let line =
                            Line::new(plot_with_name.raw_plot.to_vec()).name(plot_with_name.name);
                        plot_ui.line(line.width(*line_width));
                    }

                    if let Some(t) = timer {
                        let mut bounds = plot_ui.plot_bounds();
                        bounds.translate_x(t * 1000.0); // multiply by 1000 to get milliseconds
                        plot_ui.set_plot_bounds(bounds);
                    }
                    if is_reset_pressed {
                        let mut bounds = plot_ui.plot_bounds();
                        bounds.translate_x(-bounds.min()[0]);
                        plot_ui.set_plot_bounds(bounds);
                    }
                });
            }

            if display_to_hundred_plot {
                ui.separator();
                to_hundred.show(ui, |plot_ui| {
                    for plot_with_name in to_hundred_plots {
                        let line =
                            Line::new(plot_with_name.raw_plot.to_vec()).name(plot_with_name.name);
                        plot_ui.line(line.width(*line_width));
                    }
                    if let Some(t) = timer {
                        let mut bounds = plot_ui.plot_bounds();
                        bounds.translate_x(t * 1000.0); // multiply by 1000 to get milliseconds
                        plot_ui.set_plot_bounds(bounds);
                    }
                    if is_reset_pressed {
                        let mut bounds = plot_ui.plot_bounds();
                        bounds.translate_x(-bounds.min()[0]);
                        plot_ui.set_plot_bounds(bounds);
                    }
                });
            }

            if display_to_thousands_plot {
                ui.separator();
                thousands.show(ui, |plot_ui| {
                    for plot_with_name in thousands_plots {
                        let line =
                            Line::new(plot_with_name.raw_plot.to_vec()).name(plot_with_name.name);
                        plot_ui.line(line.width(*line_width));
                    }

                    if let Some(log) = status_log {
                        for (ts, st_change) in log.timestamps_with_state_changes() {
                            plot_ui.text(Text::new(
                                PlotPoint::new(*ts as f64, (*st_change as u8) as f64),
                                st_change.to_string(),
                            ))
                        }
                    }
                    if let Some(t) = timer {
                        let mut bounds = plot_ui.plot_bounds();
                        bounds.translate_x(t * 1000.0); // multiply by 1000 to get milliseconds
                        plot_ui.set_plot_bounds(bounds);
                    }
                    if is_reset_pressed {
                        let mut bounds = plot_ui.plot_bounds();
                        bounds.translate_x(-bounds.min()[0]);
                        plot_ui.set_plot_bounds(bounds);
                    }
                });
            }

            if let Some(gen_log) = generator_log {
                ui.separator();
                let time_formatter = |mark: GridMark, _range: &RangeInclusive<f64>| {
                    let sec = mark.value;
                    let dt = DateTime::from_timestamp(sec as i64, 0).unwrap();
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                };
                let x_axes = vec![AxisHints::new_x().label("Time").formatter(time_formatter)];
                let label_fmt = |_s: &str, val: &PlotPoint| {
                    let dt = DateTime::from_timestamp(val.x as i64, 0).unwrap();
                    format!(
                        "{h:02}:{m:02}:{s:02}",
                        h = dt.hour(),
                        m = dt.minute(),
                        s = dt.second()
                    )
                };

                let gen_log_plot = Plot::new("generator_log_plot")
                    .legend(config.clone())
                    .height(plot_height)
                    .show_axes(self.axis_config.show_axes)
                    .x_axis_position(VPlacement::Top)
                    .y_axis_position(HPlacement::Right)
                    .custom_x_axes(x_axes)
                    .label_formatter(label_fmt)
                    .include_y(0.0);

                gen_log_plot.show(ui, |plot_ui| {
                    for line_plot in gen_log.all_plots() {
                        plot_ui.line(line_plot.width(*line_width));
                    }
                    if let Some(t) = timer {
                        let mut bounds = plot_ui.plot_bounds();
                        bounds.translate_x(t); // Divide by 1000 because this plot is in seconds but timer is in ms
                        plot_ui.set_plot_bounds(bounds);
                    }
                    if is_reset_pressed {
                        let mut bounds = plot_ui.plot_bounds();
                        let first_timestamp = gen_log.first_timestamp().unwrap_or(0.0);

                        // Translate X to start from the first data point timestamp
                        bounds.translate_x(-bounds.min()[0] + first_timestamp);
                        plot_ui.set_plot_bounds(bounds);
                    }
                });
            }
        })
        .response
    }
}
