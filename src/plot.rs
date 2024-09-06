use crate::logs::{
    pid::{PidLog, PidLogEntry},
    status::{StatusLog, StatusLogEntry},
    LogEntry,
};
use egui::Response;
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints};

#[derive(Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LogPlot {
    config: Legend,
    line_width: f32,
}

impl LogPlot {
    fn line_from_log_entry<F, L: LogEntry>(pid_logs: &[L], y_extractor: F) -> Line
    where
        F: Fn(&L) -> f64,
    {
        let points: PlotPoints = pid_logs
            .iter()
            .map(|e| {
                let x = e.timestamp_ms() as f64;
                let y = y_extractor(e);
                [x, y]
            })
            .collect();
        Line::new(points)
    }

    fn pid_log_lines(pid_logs: &[PidLogEntry]) -> Vec<Line> {
        vec![
            Self::line_from_log_entry(pid_logs, |e| e.rpm as f64).name("RPM"),
            Self::line_from_log_entry(pid_logs, |e| e.pid_err as f64).name("PID Error"),
            Self::line_from_log_entry(pid_logs, |e| e.servo_duty_cycle as f64)
                .name("Servo Duty Cycle"),
        ]
    }

    fn status_log_lines(status_log: &[StatusLogEntry]) -> Vec<Line> {
        vec![
            Self::line_from_log_entry(status_log, |e| e.engine_temp as f64).name("Engine Temp °C"),
            Self::line_from_log_entry(status_log, |e| (e.fan_on as u8) as f64).name("Fan On"),
            Self::line_from_log_entry(status_log, |e| e.vbat.into()).name("Vbat"),
            Self::line_from_log_entry(status_log, |e| e.setpoint.into()).name("Setpoint"),
            Self::line_from_log_entry(status_log, |e| e.motor_state.into()).name("Motor State"),
        ]
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        pid_log: Option<&PidLog>,
        status_log: Option<&StatusLog>,
    ) -> Response {
        let Self { config, line_width } = self;

        egui::Grid::new("settings").show(ui, |ui| {
            ui.label("Text style:");
            ui.horizontal(|ui| {
                let all_text_styles = ui.style().text_styles();
                for style in all_text_styles {
                    ui.selectable_value(&mut config.text_style, style.clone(), style.to_string());
                }
            });

            ui.end_row();

            ui.label("Position:");
            ui.horizontal(|ui| {
                Corner::all().for_each(|position| {
                    ui.selectable_value(&mut config.position, position, format!("{position:?}"));
                });
            });
            ui.end_row();

            ui.label("Opacity:");
            ui.add(
                egui::DragValue::new(&mut config.background_alpha)
                    .speed(0.02)
                    .range(0.0..=1.0),
            );
            ui.label("Line width");
            ui.add(
                egui::DragValue::new(line_width)
                    .speed(0.02)
                    .range(0.5..=20.0),
            );
            ui.end_row();
        });
        let legend_plot = Plot::new("plots").legend(config.clone());
        legend_plot
            .show(ui, |plot_ui| {
                if let Some(log) = pid_log {
                    for lineplot in Self::pid_log_lines(log.entries()) {
                        plot_ui.line(lineplot.width(*line_width));
                    }
                }
                if let Some(log) = status_log {
                    for lineplot in Self::status_log_lines(log.entries()) {
                        plot_ui.line(lineplot.width(*line_width));
                    }
                }
            })
            .response
    }
}
