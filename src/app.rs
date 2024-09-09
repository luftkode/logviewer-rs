use crate::plot::LogPlot;
use egui::{DroppedFile, Hyperlink};
use std::time::{Duration, SystemTime};
use supported_logs::SupportedLogs;

mod supported_logs;
mod util;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    dropped_files: Vec<DroppedFile>,
    picked_path: Option<String>,
    logs: SupportedLogs,
    plot: LogPlot,
    font_size: f32,
    is_playing: bool,               // Whether the plot is playing
    start_time: Option<SystemTime>, // Store the time when the animation started
    elapsed_time: Duration,
    elapsed_last_plot_update: f64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            dropped_files: Vec::new(),
            picked_path: None,
            logs: SupportedLogs::default(),
            plot: LogPlot::default(),
            font_size: Self::DEFAULT_FONT_SIZE,
            is_playing: false,
            start_time: None,
            elapsed_time: Duration::from_secs(0),
            elapsed_last_plot_update: 0.0,
        }
    }
}

impl App {
    const DEFAULT_FONT_SIZE: f32 = 16.0;

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        // Set default font size for all font styles
        let mut style = (*cc.egui_ctx.style()).clone();
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = Self::DEFAULT_FONT_SIZE;
        }
        cc.egui_ctx.set_style(style);

        Default::default()
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
                if ui.button("Reset").clicked() {
                    *self = Self::default();
                }

                ui.label("Font size:");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.font_size)
                            .speed(0.1)
                            .range(8.0..=32.0)
                            .suffix("px"),
                    )
                    .changed()
                {
                    // Update the font size for all text styles
                    let mut style = (*ctx.style()).clone();
                    for (_text_style, font_id) in style.text_styles.iter_mut() {
                        font_id.size = self.font_size;
                    }
                    ctx.set_style(style);
                }
                egui::widgets::global_dark_light_mode_buttons(ui);
                ui.add(Hyperlink::from_label_and_url(
                    "Homepage",
                    "https://github.com/luftkode/logviewer-rs",
                ));
                if ui
                    .button(if self.is_playing { "Pause" } else { "Play" })
                    .clicked()
                {
                    self.is_playing = !self.is_playing;
                    if self.is_playing {
                        self.start_time = Some(SystemTime::now());
                    } else {
                        // Pause: accumulate the time played so far
                        if let Some(start) = self.start_time {
                            // Add the time played since the last "start"
                            self.elapsed_time += start.elapsed().unwrap_or_default();
                            self.start_time = None; // Stop tracking the current time
                        }
                    }
                }
                if self.is_playing {
                    if let Some(start) = self.start_time {
                        // Calculate time passed since the current play session started
                        let time_since_last_start = start.elapsed().unwrap_or_default();
                        let total_elapsed_time = self.elapsed_time + time_since_last_start;

                        let seconds_elapsed = total_elapsed_time.as_secs_f64();
                        ui.label(format!("{:.2}s", seconds_elapsed));

                        // Make sure the GUI is repainted while the timer is running
                        ctx.request_repaint();
                    }
                } else {
                    // Display the total time passed when paused
                    let seconds_elapsed = self.elapsed_time.as_secs_f64();
                    ui.label(format!("{:.2}s", seconds_elapsed));
                }
            });
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
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            let play_timer_update_val = if self.is_playing {
                self.start_time.and_then(|start_time| {
                    let current_elapsed = start_time.elapsed().unwrap_or_default();
                    let total_elapsed = self.elapsed_time + current_elapsed;
                    let elapsed_since_last_update =
                        total_elapsed.as_millis() as f64 - self.elapsed_last_plot_update;

                    self.elapsed_last_plot_update = total_elapsed.as_millis() as f64;

                    if elapsed_since_last_update > 0.0 {
                        Some(elapsed_since_last_update)
                    } else {
                        None
                    }
                })
            } else {
                None
            };
            self.plot.ui(
                ui,
                self.logs.mbed_pid_log(),
                self.logs.mbed_status_log(),
                self.logs.generator_log(),
                play_timer_update_val,
            );

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

            util::preview_files_being_dropped(ctx);
            // Collect dropped files:
            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    self.dropped_files.clone_from(&i.raw.dropped_files);
                    SupportedLogs::parse_dropped_files(&self.dropped_files, &mut self.logs);
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
