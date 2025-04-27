mod builder;
mod output;
mod state;
mod text_buffer;

pub use {
    builder::TextEdit, nih_plug_egui::egui::text_selection::TextCursorState,
    output::TextEditOutput, state::TextEditState, text_buffer::TextBuffer,
};
