use egui::{Key, RichText};
use egui_phosphor::regular;

use super::{axis_config::AxisConfig, plot_settings::PlotSettings};

// filter settings should be refactored out to be a standalone thing, maybe together with loaded_logs_ui
pub fn show_settings_grid(
    ui: &mut egui::Ui,
    line_width: &mut f32,
    axis_cfg: &mut AxisConfig,
    plot_settings: &mut PlotSettings,
) {
    egui::Grid::new("settings").show(ui, |ui| {
        ui.horizontal_top(|ui| {
            plot_settings.show(ui);
            ui.label("|");
            let axis_cfg_str = RichText::new(format!("{} Axis config", regular::GEAR));
            if ui.button(axis_cfg_str.clone()).clicked() {
                axis_cfg.ui_visible = !axis_cfg.ui_visible;
            }
            if axis_cfg.ui_visible {
                let mut open: bool = axis_cfg.ui_visible;
                egui::Window::new(axis_cfg_str)
                    .open(&mut open)
                    .show(ui.ctx(), |ui| {
                        axis_cfg.toggle_axis_cfg_ui(ui);
                    });
                axis_cfg.ui_visible = open;
            }
            if ui.ctx().input(|i| i.key_pressed(Key::Escape)) {
                axis_cfg.ui_visible = false;
            }
            ui.label("Line width");
            ui.add(
                egui::DragValue::new(line_width)
                    .speed(0.02)
                    .range(0.5..=20.0),
            );
        });
    });
}
