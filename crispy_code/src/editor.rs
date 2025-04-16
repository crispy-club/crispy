use crate::controller::Controller;
use nih_plug::prelude::Editor;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, Vec2},
    resizable_window::ResizableWindow,
    EguiState,
};
use std::sync::Arc;

#[derive(Default)]
struct TextEditor {
    text: String,
    file_path: Option<String>,
}

pub fn create_editor(_controller: Arc<Controller>) -> Option<Box<dyn Editor>> {
    let egui_state = EguiState::from_size(1024, 768);
    let editor = TextEditor::default();

    create_egui_editor(
        egui_state.clone(),
        editor,
        |_, _| {},
        move |egui_ctx, _setter, state: &mut TextEditor| {
            ResizableWindow::new("res-wind")
                .min_size(Vec2::new(128.0, 128.0))
                .show(egui_ctx, egui_state.as_ref(), |ui| {
                    update(egui_ctx, ui, state)
                });
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

fn update(_ctx: &egui::Context, ui: &mut egui::Ui, editor: &mut TextEditor) {
    if ui.button("Open").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                editor.text = contents;
                editor.file_path = Some(path.display().to_string());
            }
        }
    }

    if ui.button("Save").clicked() {
        if let Some(path) = &editor.file_path {
            let _ = std::fs::write(path, &editor.text);
        }
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut editor.text)
                    .desired_rows(30)
                    .desired_width(f32::INFINITY),
            );
        });
}
