use std::{ops::RangeInclusive, time::Duration};

use crate::{
    app::PlayBackButtonEvent,
    logs::{
        generator::GeneratorLog,
        mbed_motor_control::{
            pid::{PidLog, PidLogEntry},
            status::{StatusLog, StatusLogEntry},
        },
        Log, LogEntry,
    },
};
use chrono::{DateTime, Timelike};
use egui::{Color32, Pos2, Response, RichText};
use egui_plot::{
    AxisHints, GridMark, HPlacement, Legend, Line, Plot, PlotItem, PlotPoint, PlotPoints, Points,
    Text, VPlacement,
};
use play_state::PlayState;

mod play_state;

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
    first_click: Option<[f64; 2]>,
    second_click: Option<[f64; 2]>,
    difference: Option<[f64; 2]>,
}

impl Default for LogPlot {
    fn default() -> Self {
        Self {
            config: Default::default(),
            line_width: 1.0,
            axis_config: Default::default(),
            play_state: PlayState::default(),
            first_click: None,
            second_click: None,
            difference: None,
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

    fn line_from_log_entry<XF, YF, L: LogEntry>(
        pid_logs: &[L],
        x_extractor: XF,
        y_extractor: YF,
    ) -> Line
    where
        XF: Fn(&L) -> f64,
        YF: Fn(&L) -> f64,
    {
        let points: PlotPoints = pid_logs
            .iter()
            .map(|e| [x_extractor(e), y_extractor(e)])
            .collect();
        Line::new(points)
    }

    fn pid_log_lines(pid_logs: &[PidLogEntry]) -> (Vec<Line>, Vec<Line>) {
        let zero_to_one_range = vec![
            Self::line_from_log_entry(pid_logs, |e| e.timestamp_ms() as f64, |e| e.pid_err as f64)
                .name("PID Error"),
            Self::line_from_log_entry(
                pid_logs,
                |e| e.timestamp_ms() as f64,
                |e| e.servo_duty_cycle as f64,
            )
            .name("Servo Duty Cycle"),
        ];
        let big_range = vec![Self::line_from_log_entry(
            pid_logs,
            |e| e.timestamp_ms() as f64,
            |e| e.rpm as f64,
        )
        .name("RPM")];
        (zero_to_one_range, big_range)
    }

    fn status_log_lines(status_log: &[StatusLogEntry]) -> (Vec<Line>, Vec<Line>) {
        let zero_to_one_range = vec![Self::line_from_log_entry(
            status_log,
            |e| e.timestamp_ms() as f64,
            |e| (e.fan_on as u8) as f64,
        )
        .name("Fan On")];

        let big_range = vec![
            Self::line_from_log_entry(
                status_log,
                |e| e.timestamp_ms() as f64,
                |e| e.engine_temp as f64,
            )
            .name("Engine Temp °C"),
            Self::line_from_log_entry(status_log, |e| e.timestamp_ms() as f64, |e| e.vbat.into())
                .name("Vbat"),
            Self::line_from_log_entry(
                status_log,
                |e| e.timestamp_ms() as f64,
                |e| (e.motor_state as u8) as f64,
            )
            .name("Motor State"),
            Self::line_from_log_entry(
                status_log,
                |e| e.timestamp_ms() as f64,
                |e| e.setpoint.into(),
            )
            .name("Setpoint"),
        ];
        (zero_to_one_range, big_range)
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
            first_click,
            second_click,
            difference,
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
                ui.checkbox(&mut self.axis_config.link_x, "Linked Axes");
                ui.checkbox(&mut self.axis_config.link_cursor_x, "Linked Cursors");
                ui.checkbox(&mut self.axis_config.show_axes, "Show Axes");
            });

            ui.horizontal_centered(|ui| {
                ui.label("| ");
                // Reset button
                let reset_text = RichText::new(egui_phosphor::regular::SKIP_BACK);
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
            if let Some([dx, dy]) = difference {
                ui.label(format!("Difference - X: {:.2}, Y: {:.2}", dx, dy));
            }
            ui.end_row();
        });
        if let Some(e) = playback_button_event {
            play_state.handle_playback_button_press(e);
        };
        let is_reset_pressed = matches!(playback_button_event, Some(PlayBackButtonEvent::Reset));
        let timer = play_state.time_since_update();

        let link_group_id = ui.id().with("linked_plots");
        ui.vertical(|ui| {
            // Determining plot count really needs a refactor
            // the generator and pid logs count as 2 if any of them is there, just because they both have values that fall on the 0-1 plot and the "large values"-plot
            // hopefully future plots with be compatible with this format or the format needs refactoring (it will at some point) such that many logs can be plotted on the same
            // few plots that handle different value ranges (or look into custom axes in egui_plot)
            let mut total_plot_count: u8 = 0;
            if pid_log.is_some() || status_log.is_some() {
                total_plot_count += 2;
            }
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

            let zero_to_one_range_plot = Plot::new("zero_to_one_range_plot")
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

            // Plot for values outside 0-1 range
            let large_range_plot = Plot::new("large_range_plot")
                .legend(config.clone())
                .height(plot_height)
                .show_axes(self.axis_config.show_axes)
                .y_axis_position(HPlacement::Right)
                .x_axis_formatter(move |x, _range| format_time(x.value))
                .link_axis(link_group_id, self.axis_config.link_x, false)
                .link_cursor(link_group_id, self.axis_config.link_cursor_x, false);

            let (zero_to_one_range_pid, large_range_pid) = pid_log
                .map(|log| Self::pid_log_lines(log.entries()))
                .unwrap_or((vec![], vec![]));
            let (zero_to_one_range_status, large_range_status) = status_log
                .map(|log| Self::status_log_lines(log.entries()))
                .unwrap_or((vec![], vec![]));

            let has_zero_to_one_data =
                !zero_to_one_range_pid.is_empty() || !zero_to_one_range_status.is_empty();
            let has_large_range_data = !large_range_pid.is_empty()
                || !large_range_status.is_empty()
                || status_log.is_some();

            if has_zero_to_one_data {
                zero_to_one_range_plot.show(ui, |plot_ui| {
                    if let Some(status_log) = status_log {
                        for (ts, st_change) in status_log.timestamps_with_state_changes() {
                            plot_ui.text(Text::new(
                                PlotPoint::new(*ts as f64, ((*st_change as u8) as f64) / 10.0),
                                st_change.to_string(),
                            ))
                        }
                    }
                    for lineplot in zero_to_one_range_pid {
                        plot_ui.line(lineplot.width(*line_width));
                    }
                    for lineplot in zero_to_one_range_status {
                        plot_ui.line(lineplot.width(*line_width));
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

                    if let Some(first) = *first_click {
                        plot_ui.points(
                            Points::new(vec![first])
                                .shape(egui_plot::MarkerShape::Plus)
                                .color(Color32::DARK_RED)
                                .radius(8.0)
                                .name(format!("[{:.2}, {:.3}]", first[0], first[1])),
                        );
                    }

                    if let Some(second) = *second_click {
                        plot_ui.points(
                            Points::new(vec![second])
                                .shape(egui_plot::MarkerShape::Plus)
                                .color(Color32::BLUE)
                                .radius(8.0)
                                .name(format!("[{:.2}, {:.3}]", second[0], second[1])),
                        );
                    }

                    if plot_ui.response().clicked() {
                        log::info!("Plot was clicked");
                        let pos = plot_ui.pointer_coordinate().unwrap();
                        log::info!("Clicked at: {pos:?}");
                        let position = pos.to_pos2();
                        let click_point = [position.x as f64, position.y as f64];
                        if first_click.is_none() {
                            *first_click = Some(click_point);
                            *second_click = None;
                            *difference = None;
                        } else if second_click.is_none() {
                            *second_click = Some(click_point);
                            let (first, second) = (first_click.unwrap(), click_point);
                            let dx = second[0] - first[0];
                            let dy = second[1] - first[1];
                            *difference = Some([dx, dy]);
                        } else {
                            *first_click = None;
                            *second_click = None;
                            *difference = None;
                        }
                    }
                });
            }

            if has_large_range_data {
                large_range_plot.show(ui, |plot_ui| {
                    for lineplot in large_range_pid {
                        plot_ui.line(lineplot.width(*line_width));
                    }
                    if let Some(log) = status_log {
                        for (ts, st_change) in log.timestamps_with_state_changes() {
                            plot_ui.text(Text::new(
                                PlotPoint::new(*ts as f64, (*st_change as u8) as f64),
                                st_change.to_string(),
                            ))
                        }
                        for lineplot in large_range_status {
                            plot_ui.line(lineplot.width(*line_width));
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

                    if let Some(first) = *first_click {
                        plot_ui.points(
                            Points::new(vec![first])
                                .shape(egui_plot::MarkerShape::Cross)
                                .color(Color32::RED)
                                .radius(25.0)
                                .name("First Click"),
                        );
                    }

                    if let Some(second) = *second_click {
                        plot_ui.points(
                            Points::new(vec![second])
                                .shape(egui_plot::MarkerShape::Plus)
                                .color(Color32::BLUE)
                                .radius(25.0)
                                .name("Second Click"),
                        );
                    }

                    if plot_ui.response().clicked() {
                        log::info!("Plot was clicked");
                        let pos = plot_ui.pointer_coordinate().unwrap();
                        log::info!("Clicked at: {pos:?}");
                        let position = pos.to_pos2();
                        let click_point = [position.x as f64, position.y as f64];
                        if first_click.is_none() {
                            *first_click = Some(click_point);
                            *second_click = None;
                            *difference = None;
                        } else if second_click.is_none() {
                            *second_click = Some(click_point);
                            let (first, second) = (first_click.unwrap(), click_point);
                            let dx = second[0] - first[0];
                            let dy = second[1] - first[1];
                            *difference = Some([dx, dy]);
                        } else {
                            *first_click = Some(click_point);
                            *second_click = None;
                            *difference = None;
                        }
                    }
                });
            }
        })
        .response
    }
}
