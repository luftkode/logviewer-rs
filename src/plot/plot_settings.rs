use date_settings::LoadedLogSettings;
use egui::{Color32, Key, Response, RichText};
use egui_phosphor::regular;
use mipmap_settings::MipMapSettings;
use plot_filter::{PlotNameFilter, PlotNameShow};
use plot_util::{MipMapConfiguration, PlotValues, Plots};
use plot_visibility_config::PlotVisibilityConfig;
use serde::{Deserialize, Serialize};

pub mod date_settings;
mod loaded_logs;
pub mod mipmap_settings;
mod plot_filter;
mod plot_visibility_config;

#[derive(PartialEq, Deserialize, Serialize)]
struct PlotSettingsUi {
    show_loaded_logs: bool,
    show_filter_settings: bool,
    filter_settings_text: String,
}

impl Default for PlotSettingsUi {
    fn default() -> Self {
        Self {
            show_loaded_logs: Default::default(),
            show_filter_settings: Default::default(),
            filter_settings_text: format!("{} Filter", regular::FUNNEL),
        }
    }
}

impl PlotSettingsUi {
    pub fn filter_settings_text(&self) -> String {
        self.filter_settings_text.clone()
    }

    pub fn ui_toggle_show_filter(&mut self, ui: &mut egui::Ui) -> Response {
        let show_filter_text = self.filter_settings_text();
        ui.toggle_value(&mut self.show_filter_settings, show_filter_text)
    }
}

#[derive(Default, PartialEq, Deserialize, Serialize)]
pub struct PlotSettings {
    /// Used for invalidating any cached values that determines plot layout etc.
    invalidate_plot: bool,
    visibility: PlotVisibilityConfig,
    display_percentage_plot: bool,
    display_hundreds_plot: bool,
    display_thousands_plot: bool,
    display_plot_count: u8,
    // Plot names and whether or not they should be shown (painted)
    plot_name_filter: PlotNameFilter,
    ps_ui: PlotSettingsUi,
    log_start_date_settings: Vec<LoadedLogSettings>,
    mipmap_settings: MipMapSettings,
}

impl PlotSettings {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if self.log_start_date_settings.is_empty() {
            ui.label(RichText::new("No Logs Loaded").color(Color32::RED));
        } else {
            self.show_loaded_logs(ui);
            self.ui_plot_filter_settings(ui);
            self.mipmap_settings.show(ui);
        }
        self.visibility.toggle_visibility_ui(ui);
    }

    fn ui_plot_filter_settings(&mut self, ui: &mut egui::Ui) {
        self.ps_ui.ui_toggle_show_filter(ui);
        if self.ps_ui.show_filter_settings {
            egui::Window::new(self.ps_ui.filter_settings_text())
                .open(&mut self.ps_ui.show_filter_settings)
                .show(ui.ctx(), |ui| {
                    self.plot_name_filter.show(ui);
                });
            if ui.ctx().input(|i| i.key_pressed(Key::Escape)) {
                self.ps_ui.show_filter_settings = false;
            }
        }
    }

    fn show_loaded_logs(&mut self, ui: &mut egui::Ui) {
        let loaded_log_count = self.log_start_date_settings.len();
        let visibility_icon = if self.ps_ui.show_loaded_logs {
            regular::EYE
        } else {
            regular::EYE_SLASH
        };
        let show_loaded_logs_text = RichText::new(format!(
            "{visibility_icon} Loaded logs ({loaded_log_count})",
        ));
        ui.toggle_value(
            &mut self.ps_ui.show_loaded_logs,
            show_loaded_logs_text.text(),
        );
        if self.ps_ui.show_loaded_logs {
            // Only react on Escape input if no settings are currently open
            if ui.ctx().input(|i| i.key_pressed(Key::Escape))
                && !self.log_start_date_settings.iter().any(|s| s.clicked)
            {
                self.ps_ui.show_loaded_logs = false;
            }
            egui::Window::new(show_loaded_logs_text)
                .open(&mut self.ps_ui.show_loaded_logs)
                .show(ui.ctx(), |ui| {
                    egui::Grid::new("log_settings_grid").show(ui, |ui| {
                        for settings in &mut self.log_start_date_settings {
                            loaded_logs::log_date_settings_ui(ui, settings);
                            ui.end_row();
                        }
                    });
                });
        }
    }

    /// Needs to be called once (and only once!) per frame before querying for plot ui settings, such as
    /// how many plots to paint and more.
    pub fn refresh(&mut self, plots: &mut Plots) {
        self.update_plot_dates(plots);
        self.calc_plot_display_settings(plots);
        // If true then we set it to false such that it is only true for one frame
        if self.cached_plots_invalidated() {
            self.invalidate_plot = false;
        }
    }

    /// Whether or not to display the `percentage` plot area in the current frame
    pub fn display_percentage(&self) -> bool {
        self.display_percentage_plot
    }

    /// Whether or not to display the `one_to_hundred` plot area in the current frame
    pub fn display_hundreds(&self) -> bool {
        self.display_hundreds_plot
    }

    /// Whether or not to display the `thousands` plot area in the current frame
    pub fn display_thousands(&self) -> bool {
        self.display_thousands_plot
    }

    /// How many plots to paint in the current frame
    pub fn total_plot_count(&self) -> u8 {
        self.display_plot_count
    }

    /// Needs to be called once per frame before querying which plots to display
    pub fn calc_plot_display_settings(&mut self, plots: &Plots) {
        self.display_percentage_plot = self
            .visibility
            .should_display_percentage(plots.percentage().plots().is_empty());
        self.display_hundreds_plot = self
            .visibility
            .should_display_hundreds(plots.one_to_hundred().plots().is_empty());
        self.display_thousands_plot = self
            .visibility
            .should_display_thousands(plots.thousands().plots().is_empty());
        let mut total_plot_count: u8 = 0;
        total_plot_count += self.display_percentage_plot as u8;
        total_plot_count += self.display_hundreds_plot as u8;
        total_plot_count += self.display_thousands_plot as u8;
        self.display_plot_count = total_plot_count;
    }

    /// Adds a new plot name/label to the collection if it isn't already in the collection
    ///
    /// # Arguments
    /// - `plot_name` The name of the plot, i.e. the name that appears on the plot legend
    pub fn add_plot_name_if_not_exists(&mut self, plot_name: &str) {
        if !self.plot_name_filter.contains_name(plot_name) {
            self.plot_name_filter
                .add_plot(PlotNameShow::new(plot_name.to_owned(), true));
        }
    }

    pub fn apply_filters<'pv>(
        &'pv self,
        plot_vals: &'pv [PlotValues],
    ) -> impl Iterator<Item = &'pv PlotValues> {
        let id_filter = self.log_id_filter();
        self.plot_name_filter
            .filter_plot_values(plot_vals, move |id| {
                for id_inst in &id_filter {
                    if *id_inst == id {
                        return false;
                    }
                }
                true
            })
    }

    /// Get the next ID for a log, used for when a new log is loaded and added to the collection of logs and log settings
    pub fn next_log_id(&self) -> usize {
        (self.total_logs() + 1).into()
    }

    pub fn total_logs(&self) -> u16 {
        self.log_start_date_settings.len() as u16
    }

    pub fn add_log_setting(&mut self, log_settings: LoadedLogSettings) {
        self.log_start_date_settings.push(log_settings);
    }

    // The id filter specifies which plots belonging to which logs should not be painted on the plot ui.
    pub fn log_id_filter(&self) -> Vec<usize> {
        let mut log_id_filter: Vec<usize> = vec![];
        for settings in &self.log_start_date_settings {
            if !settings.show_log() {
                log_id_filter.push(settings.log_id());
            }
        }
        log_id_filter
    }

    fn update_plot_dates(&mut self, plots: &mut Plots) {
        for settings in &mut self.log_start_date_settings {
            date_settings::update_plot_dates(&mut self.invalidate_plot, plots, settings);
        }
    }

    /// Returns true if changes in plot settings occurred such that various cached values
    /// related to plot layout needs to be recalculated.
    pub fn cached_plots_invalidated(&self) -> bool {
        self.invalidate_plot
    }

    /// Returns the current `MipMap` settings as a [`MipMapConfiguration`]
    pub fn mipmap_cfg(&self) -> MipMapConfiguration {
        self.mipmap_settings.configuration()
    }
}
