use nih_plug::prelude::Editor;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, Vec2},
    resizable_window::ResizableWindow,
    EguiState,
};

fn create_editor() -> Option<Box<dyn Editor>> {
    create_egui_editor(
        EguiState::from_size(1024, 768),
        (),
        |_, _| {},
        move |egui_ctx, setter, _state| {
            ResizableWindow::new("res-wind")
                .min_size(Vec2::new(128.0, 128.0))
                .show(egui_ctx, egui_state.as_ref(), |ui| editor_ui(egui_ctx))
        },
    )
}

static FONT_DATA: &[u8] = include_bytes!("assets/fonts/Monaco.ttf");

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Insert the font data
    fonts.font_data.insert(
        "my_font".to_owned(),
        // egui::FontData::from_owned(my_font_bytes).into(),
        egui::FontData::from_owned(FONT_DATA.to_vec()).into(),
    );

    // Set the font to be used for proportional and monospace text
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_font".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("my_font".to_owned());

    ctx.set_fonts(fonts);
}

#[derive(Default, PartialEq)]
enum Enum {
    #[default]
    First,
    Second,
    Third,
}

#[derive(Default)]
struct MyApp {
    a_float: f32,
    box_checked: bool,
    my_enum: Enum,
    text: String,
    file_path: Option<String>,
}

fn update(ctx: &egui::Context, ui: egui::Ui) {
    if ui.button("Open").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                self.text = contents;
                self.file_path = Some(path.display().to_string());
            }
        }
    }

    if ui.button("Save").clicked() {
        if let Some(path) = &self.file_path {
            let _ = std::fs::write(path, &self.text);
        }
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.text)
                    .desired_rows(30)
                    .desired_width(f32::INFINITY),
            );
        });
}
