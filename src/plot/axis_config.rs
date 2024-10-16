use axis_lock::YAxisLock;
use egui::{Color32, RichText};
use egui_phosphor::regular;

use super::PlotType;

mod axis_lock;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AxisConfig {
    link_x: bool,
    link_cursor_x: bool,
    show_axes: bool,
    show_grid: bool,
    y_axis_lock: YAxisLock,
    pub ui_visible: bool,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            link_x: true,
            link_cursor_x: true,
            show_axes: true,
            show_grid: true,
            y_axis_lock: YAxisLock::default(),
            ui_visible: false,
        }
    }
}

impl AxisConfig {
    pub fn show_axes(&self) -> bool {
        self.show_axes
    }

    pub fn link_x(&self) -> bool {
        self.link_x
    }

    pub fn link_cursor_x(&self) -> bool {
        self.link_cursor_x
    }

    pub fn show_grid(&self) -> bool {
        self.show_grid
    }

    pub fn handle_y_axis_lock<F>(
        &mut self,
        plot_ui: &mut egui_plot::PlotUi,
        plot_type: PlotType,
        between_bounds_update_fn: F,
    ) where
        F: FnOnce(&mut egui_plot::PlotUi),
    {
        self.y_axis_lock
            .handle(plot_ui, plot_type, between_bounds_update_fn);
    }

    pub fn toggle_axis_cfg_ui(&mut self, ui: &mut egui::Ui) {
        let linked_x_axis_text = format!(
            "{} Linked Axes",
            if self.link_x {
                regular::LINK_SIMPLE
            } else {
                regular::LINK_BREAK
            }
        );
        ui.toggle_value(&mut self.link_x, linked_x_axis_text);
        let linked_x_cursor_text = format!(
            "{} Linked Cursors",
            if self.link_cursor_x {
                regular::LINK_SIMPLE
            } else {
                regular::LINK_BREAK
            }
        );
        ui.toggle_value(&mut self.link_cursor_x, linked_x_cursor_text);
        let show_axes_text = format!(
            "{} Axes",
            if self.show_axes {
                regular::EYE
            } else {
                regular::EYE_SLASH
            }
        );
        ui.toggle_value(&mut self.show_axes, show_axes_text);
        let is_y_axis_locked = self.y_axis_lock.lock_y_axis;
        let lock_y_axis_text = RichText::new(format!(
            "{} Lock Y-axis",
            if is_y_axis_locked {
                regular::LOCK_LAMINATED
            } else {
                regular::LOCK_SIMPLE_OPEN
            }
        ));
        let show_grid_text = format!(
            "{} Grid",
            if self.show_axes {
                regular::EYE
            } else {
                regular::EYE_SLASH
            }
        );
        ui.toggle_value(&mut self.show_grid, show_grid_text);
        let lock_y_axis_text = if is_y_axis_locked {
            lock_y_axis_text.color(Color32::RED)
        } else {
            lock_y_axis_text
        };

        ui.toggle_value(&mut self.y_axis_lock.lock_y_axis, lock_y_axis_text);
    }
}
