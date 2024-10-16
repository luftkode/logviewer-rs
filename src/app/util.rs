use egui::{DroppedFile, Hyperlink, RichText, Stroke};

pub fn file_info(file: &DroppedFile) -> String {
    let path = file
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .or_else(|| (!file.name.is_empty()).then(|| file.name.clone()))
        .unwrap_or_else(|| "???".to_owned());

    let mut info = vec![path];
    if !file.mime.is_empty() {
        info.push(format!("type: {}", file.mime));
    }
    if let Some(bytes) = &file.bytes {
        info.push(format!("{} bytes", bytes.len()));
    }

    info.join(" ")
}

pub fn draw_empty_state(gui: &mut egui::Ui) {
    gui.vertical_centered(|arg_ui| {
        arg_ui.add_space(100.0);
        arg_ui.heading(RichText::new("Drag and drop files, directories/folder, or zip archives onto this window").size(40.0));
        arg_ui.add_space(40.0);

        let table_width = arg_ui.available_width() * 0.8;
        egui::Frame::none()
            .fill(arg_ui.style().visuals.extreme_bg_color)
            .stroke(Stroke::new(1.0, arg_ui.style().visuals.widgets.active.bg_fill))
            .inner_margin(10.0)
            .outer_margin(0.0)
            .show(arg_ui, |inner_arg_ui| {
                inner_arg_ui.set_width(table_width);
                egui::Grid::new("supported_formats_grid")
                        .num_columns(2)
                        .spacing([40.0, 10.0])
                        .striped(true)
                        .show(inner_arg_ui, |ui| {
                            ui.colored_label(
                                ui.style().visuals.strong_text_color(),
                                "Supported Formats",
                            );
                            ui.colored_label(
                                ui.style().visuals.strong_text_color(),
                                "Description",
                            );
                            ui.end_row();

                            ui.label(RichText::new("Mbed Motor Control").strong());
                            ui.label("Logs from Mbed-based motor controller");
                            ui.add(Hyperlink::from_label_and_url(
                                "https://github.com/luftkode/mbed-motor-control",
                                "https://github.com/luftkode/mbed-motor-control",
                            ));

                            ui.end_row();

                            ui.label("• PID Logs");
                            ui.label("Contains PID controller data");
                            ui.end_row();

                            ui.label("• Status Logs");
                            ui.label(
                                "General status information such as engine temperature, and controller state machine information",
                            );
                            ui.end_row();

                            ui.label(RichText::new("Generator").strong());
                            ui.label("Logs from the generator");

                            ui.add(Hyperlink::from_label_and_url(
                                "https://github.com/luftkode/delphi_generator_linux",
                                "https://github.com/luftkode/delphi_generator_linux",
                            ));
                            ui.end_row();


                            ui.label(RichText::new("Bifrost TX Loop Current").strong());
                            ui.label("Loop Current measurements");

                            ui.add(Hyperlink::from_label_and_url(
                                "https://github.com/luftkode/bifrost-app",
                                "https://github.com/luftkode/bifrost-app",
                            ));
                            ui.end_row();
                        });
            });
    });
}
