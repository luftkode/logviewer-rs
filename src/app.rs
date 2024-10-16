use std::time::Duration;

use crate::{plot::LogPlotUi, util::format_data_size};
use egui::{Color32, DroppedFile, Hyperlink, RichText, TextStyle};
use egui_notify::Toasts;
use log_if::prelude::Plotable;
use supported_formats::{SupportedFormat, SupportedLogs};

mod preview_dropped;
pub mod supported_formats;
mod util;

/// if a log is loaded from content that exceeds this many unparsed bytes:
/// - Show a toasts warning notification
/// - Show warnings in the UI when viewing parse info for the loaded log
pub const WARN_ON_UNPARSED_BYTES_THRESHOLD: usize = 128;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[allow(
    missing_debug_implementations,
    reason = "Some of the nested types are from egui or egui_plot and we cannot implement Debug for them"
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    toasts: Toasts,
    dropped_files: Vec<DroppedFile>,
    picked_path: Option<String>,
    logs: SupportedLogs,
    plot: LogPlotUi,
    font_size: Option<f32>,
    error_message: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            toasts: Toasts::default(),
            dropped_files: Vec::new(),
            picked_path: None,
            logs: SupportedLogs::default(),
            plot: LogPlotUi::default(),
            font_size: Some(Self::DEFAULT_FONT_SIZE),
            error_message: None,
        }
    }
}

impl App {
    const DEFAULT_FONT_SIZE: f32 = 16.0;

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn configure_text_styles(ctx: &egui::Context, font_size: f32) {
        let mut style = (*ctx.style()).clone();
        for font_id in style.text_styles.values_mut() {
            font_id.size = font_size;
        }
        ctx.set_style(style);
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        Self::configure_text_styles(ctx, self.font_size.unwrap_or_default());
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!

                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                if ui.button("Reset plot").clicked() {
                    if self.plot.plot_count() == 0 {
                        self.toasts
                            .warning("No loaded plots...")
                            .duration(Some(std::time::Duration::from_secs(3)));
                    } else {
                        self.toasts
                            .info("All loaded logs removed...")
                            .duration(Some(std::time::Duration::from_secs(3)));
                    }
                    self.logs = SupportedLogs::default();
                    self.plot = LogPlotUi::default();
                    self.dropped_files.clear();
                }

                ui.label("Font size:");
                if let Some(ref mut font_size) = self.font_size {
                    if ui
                        .add(
                            egui::DragValue::new(font_size)
                                .speed(0.1)
                                .range(8.0..=32.0)
                                .suffix("px"),
                        )
                        .changed()
                    {}
                }

                egui::widgets::global_theme_preference_buttons(ui);
                ui.add(Hyperlink::from_label_and_url(
                    "Homepage",
                    "https://github.com/luftkode/logviewer-rs",
                ));

                if is_web {
                    _ = ui.label(format!("Logviewer v{}", env!("CARGO_PKG_VERSION")));
                }
            });
            collapsible_instructions(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            notify_if_logs_added(&mut self.toasts, self.logs.logs());
            self.plot.ui(ui, &self.logs.take_logs(), &mut self.toasts);

            if self.dropped_files.is_empty() {
                // Display the message when no files have been dropped and no logs are loaded
                util::draw_empty_state(ui);
            } else {
                ui.group(|ui| {
                    ui.label("Dropped files:");
                    for file in &self.dropped_files {
                        ui.label(util::file_info(file));
                    }
                });
            }

            preview_dropped::preview_files_being_dropped(ctx);
            // Collect dropped files:
            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    self.dropped_files.clone_from(&i.raw.dropped_files);
                    match self.logs.parse_dropped_files(&self.dropped_files) {
                        Ok(()) => {
                            log::info!("OK parsing dropped file(s)");
                            self.toasts.success("Parsing complete");

                            self.error_message = None; // Clear any previous error message on success
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Error parsing dropped files: {e}"));
                        }
                    }
                }
            });

            self.show_error(ui);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
        self.toasts.show(ctx);
    }
}

impl App {
    fn show_error(&mut self, ui: &egui::Ui) {
        if let Some(error) = self.error_message.clone() {
            let screen_rect = ui.ctx().screen_rect();
            let window_width = screen_rect.width().clamp(400.0, 600.0);
            let window_height = screen_rect.height().clamp(200.0, 300.0);

            egui::Window::new(RichText::new("⚠").size(40.0))
                .fixed_size([window_width, window_height])
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        let error_text = RichText::new(&error)
                            .text_style(TextStyle::Body)
                            .size(18.0)
                            .color(Color32::RED);
                        ui.label(error_text);
                        ui.add_space(20.0);

                        let button_text = RichText::new("OK")
                            .text_style(TextStyle::Heading)
                            .size(18.0)
                            .strong();

                        let button_size = egui::Vec2::new(100.0, 40.0);
                        if ui
                            .add_sized(button_size, egui::Button::new(button_text))
                            .on_hover_text("Click to dismiss the error")
                            .clicked()
                        {
                            self.error_message = None;
                        }
                        ui.add_space(20.0);
                    });
                });
        }
    }
}

fn collapsible_instructions(ui: &mut egui::Ui) {
    ui.collapsing("Instructions", |ui| {
        ui.label("Pan by dragging, or scroll (+ shift = horizontal).");
        ui.label("Box zooming: Right click to zoom in and zoom out using a selection.");
        if cfg!(target_arch = "wasm32") {
            ui.label("Zoom with ctrl / ⌘ + pointer wheel, or with pinch gesture.");
        } else if cfg!(target_os = "macos") {
            ui.label("Zoom with ctrl / ⌘ + scroll.");
        } else {
            ui.label("Zoom with ctrl + scroll.");
        }
        ui.label("Reset view with double-click.");
    });
}

/// Displays a toasts notification if logs are added with the names of all added logs
fn notify_if_logs_added(toasts: &mut Toasts, logs: &[SupportedFormat]) {
    if !logs.is_empty() {
        let mut log_names_str = String::new();
        for l in logs {
            log_names_str.push('\n');
            log_names_str.push('\t');
            log_names_str.push_str(l.descriptive_name());
        }
        toasts
            .info(format!(
                "{} log{} added{log_names_str}",
                logs.len(),
                if logs.len() == 1 { "" } else { "s" }
            ))
            .duration(Some(Duration::from_secs(2)));
        for l in logs {
            if let Some(parse_info) = l.parse_info() {
                log::debug!(
                    "Unparsed bytes for {remainder}:{log_name}",
                    remainder = parse_info.remainder_bytes(),
                    log_name = l.descriptive_name()
                );
                if parse_info.remainder_bytes() > WARN_ON_UNPARSED_BYTES_THRESHOLD {
                    toasts
                        .warning(format!(
                    "Could only parse {parsed}/{total} for {log_name}\n{remainder} remain unparsed",
                    parsed = format_data_size(parse_info.parsed_bytes()),
                    total = format_data_size(parse_info.total_bytes()),
                    log_name = l.descriptive_name(),
                    remainder = format_data_size(parse_info.remainder_bytes())
                ))
                        .duration(Some(Duration::from_secs(30)));
                }
            }
        }
    }
}
