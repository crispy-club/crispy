use crate::controller::Controller;
use crate::custom_text_edit::{TextEdit, TextEditOutput};
use logos::Logos;
use nih_plug::nih_log;
use nih_plug::prelude::Editor;
use nih_plug_egui::{
    create_egui_editor,
    egui::{
        self,
        cache::{ComputerMut, FrameCache},
        text::{CCursor, LayoutJob},
        text_selection::CCursorRange,
        CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Galley,
        ScrollArea, TextFormat, TextStyle, TopBottomPanel, Ui,
    },
    EguiState,
};
// use rhai_fmt::{format_source, Options};
// use rhai_rowan::parser::Parser;
use rhai_rowan::syntax::SyntaxKind;
use std::sync::Arc;

const EDITOR_ID: &'static str = "main_text_editor";
const INDENT_SPACES: i32 = 4;
const WINDOW_SIZE: (u32, u32) = (1024, 768);

pub fn create_editor(_controller: Arc<Controller>) -> Option<Box<dyn Editor>> {
    let egui_state = EguiState::from_size(WINDOW_SIZE.0, WINDOW_SIZE.1);
    let editor = TextEditor::new();

    create_egui_editor(
        egui_state.clone(),
        editor,
        |_ctx: &Context, state: &mut TextEditor| {
            let syntax_highlighter = SyntaxHighlighter::default();
            state.layout_cache = Some(FrameCache::new(syntax_highlighter));
        },
        move |ctx, _setter, state: &mut TextEditor| {
            setup_fonts(ctx);

            TopBottomPanel::bottom("console").show(ctx, |ui| {
                ui.label("things will be printed down here");
            });
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    state.show(ui);
                });
            });
        },
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Event {
    KeyClosedCurly,
    KeyOpenCurly,
    KeyEnter,
    KeySpaceBar,
    Kill,
    Yank,
    Copy,
    Cut,
    Other,
}

struct TextEditor {
    contents: String,
    event_history: [Option<(Event, i32)>; 5],
    kill_buffer: Option<String>,
    layout_cache: Option<FrameCache<LayoutJob, SyntaxHighlighter>>,
}

impl TextEditor {
    fn new() -> Self {
        Self {
            contents: String::new(),
            event_history: [None; 5],
            kill_buffer: None,
            layout_cache: None,
        }
    }

    fn add_to_event_history(&mut self, event: &egui::Event) {
        nih_log!("event {:?}", event);
        let i_event = Self::internal_event(event);

        if i_event == Event::Other {
            return;
        }
        if let Some(first_event) = self.event_history[0] {
            if i_event == first_event.0 {
                self.event_history[0] = Some((i_event, first_event.1 + 1));
            } else {
                self.prepend_event(i_event)
            }
        } else {
            self.prepend_event(i_event)
        }
    }

    fn clear_events(&mut self, idx: i32) {
        assert!(idx < self.event_history.len() as i32);
        for i in 0..idx {
            self.event_history[i as usize] = None;
        }
    }

    fn copy(&mut self, ui: &Ui, output: &TextEditOutput) {
        nih_log!("contents at time of copy() -> {:?}", self.contents);
        ui.ctx().output(|o| {
            nih_log!("(copy method) o.copied_text -> {:?}", o.copied_text);
        });
    }

    fn cut(&mut self, ui: &Ui, output: &TextEditOutput) {
        nih_log!("contents at time of cut() -> {:?}", self.contents);
        ui.ctx().output(|o| {
            nih_log!("(cut method) o.copied_text -> {:?}", o.copied_text);
        });
    }

    fn handle_event_history(&mut self, ui: &Ui, output: &TextEditOutput) {
        nih_log!("handle event history {:?}", self.event_history);
        match (
            self.event_history[2],
            self.event_history[1],
            self.event_history[0],
        ) {
            (
                Some((Event::KeySpaceBar, _)),
                Some((Event::KeyOpenCurly, 1)),
                Some((Event::KeyEnter, 1)),
            ) => {
                self.indent_cursor(ui, output);
                self.clear_events(3)
            }
            _ => {}
        }
        match (self.event_history[1], self.event_history[0]) {
            (Some((Event::KeyClosedCurly, 1)), Some((Event::KeyEnter, 1))) => {
                self.outdent_closed_curly(ui, output);
                self.clear_events(2)
            }
            _ => {}
        }
        match self.event_history[0] {
            Some((Event::KeyEnter, 1)) => {
                self.indent_to_match_previous_line(ui, output);
                self.clear_events(1)
            }
            Some((Event::Kill, _)) => (), //self.kill(ui, output),
            Some((Event::Yank, _)) => self.yank(ui, output),
            Some((Event::Copy, _)) => self.copy(ui, output),
            Some((Event::Cut, _)) => self.cut(ui, output),
            _ => {}
        }
    }

    fn indent_cursor(&mut self, ui: &Ui, output: &TextEditOutput) {
        if let Some(cursor_range) = output.cursor_range {
            let cursor_pos = cursor_range.primary.ccursor.index;
            if let Some(new_cursor_pos) = cursor_pos.checked_add(INDENT_SPACES as usize) {
                if let Some(mut state) = TextEdit::load_state(ui.ctx(), EDITOR_ID.into()) {
                    self.contents
                        .push_str(" ".repeat(INDENT_SPACES as usize).as_str());
                    let ccursor = CCursor::new(new_cursor_pos);
                    state
                        .cursor
                        .set_char_range(Some(CCursorRange::one(ccursor)));
                    state.store(ui.ctx(), EDITOR_ID.into());
                }
            }
        }
    }

    fn indent_to_match_previous_line(&mut self, ui: &Ui, output: &TextEditOutput) {
        let lines: Vec<&str> = self.contents.as_str().lines().collect();
        if lines.len() < 2 {
            return;
        }
        let previous_line = lines[lines.len() - 1];
        let previous_line_spaces = get_leading_whitespace(previous_line);
        if previous_line_spaces == 0 {
            return;
        }
        if let Some(cursor_range) = output.cursor_range {
            let cursor_pos = cursor_range.primary.ccursor.index;
            if let Some(new_cursor_pos) = cursor_pos.checked_add(previous_line_spaces as usize) {
                if let Some(mut state) = TextEdit::load_state(ui.ctx(), EDITOR_ID.into()) {
                    self.contents
                        .push_str(" ".repeat(previous_line_spaces as usize).as_str());
                    let ccursor = CCursor::new(new_cursor_pos);
                    state
                        .cursor
                        .set_char_range(Some(CCursorRange::one(ccursor)));
                    state.store(ui.ctx(), EDITOR_ID.into());
                }
            }
        }
    }

    fn internal_event(event: &egui::Event) -> Event {
        match event {
            egui::Event::Text(text) => match text.as_str() {
                "}" => Event::KeyClosedCurly,
                "{" => Event::KeyOpenCurly,
                " " => Event::KeySpaceBar,
                _ => Event::Other,
            },
            egui::Event::Key {
                pressed: false, // trigger events when the key is released
                key,
                modifiers,
                ..
            } => match key {
                egui::Key::Enter => Event::KeyEnter,
                egui::Key::K if modifiers.ctrl => Event::Kill,
                egui::Key::Y if modifiers.ctrl => Event::Yank,
                _ => Event::Other,
            },
            egui::Event::Copy => Event::Copy,
            egui::Event::Cut => Event::Cut,
            _ => Event::Other,
        }
    }

    fn kill(&mut self, ui: &Ui, output: &TextEditOutput) {
        nih_log!("contents at time of kill() -> {:?}", self.contents);
        ui.ctx().output(|o| {
            nih_log!("o.copied_text -> {:?}", o.copied_text);
        });
    }

    fn outdent_closed_curly(&mut self, ui: &Ui, output: &TextEditOutput) {
        let mut lines: Vec<&str> = self.contents.as_str().lines().collect();
        let num_lines = lines.len();
        if num_lines < 2 {
            return;
        }
        let previous_line = lines[num_lines - 1];
        nih_log!("previous line -> {:?}", previous_line);
        let previous_line_spaces = get_leading_whitespace(previous_line);
        if previous_line_spaces < INDENT_SPACES as usize {
            return;
        }
        let indent_spaces = previous_line_spaces - INDENT_SPACES as usize;
        let replacement = " ".repeat(indent_spaces) + "}\n";
        lines[num_lines - 1] = replacement.as_str();
        self.contents = lines.join("\n");
        if indent_spaces == 0 {
            // Edit previous line.
            return;
        }
        // TODO: nested blocks
    }

    fn prepend_event(&mut self, i_event: Event) {
        for idx in (1..self.event_history.len()).rev() {
            self.event_history[idx] = self.event_history[idx - 1];
        }
        self.event_history[0] = Some((i_event, 1));
    }

    fn show(&mut self, ui: &mut Ui) {
        let mut layouter = |ui: &Ui, contents: &str, wrap_width: f32| -> Arc<Galley> {
            let mut layout_job: LayoutJob = self.layout_cache.as_mut().unwrap().get(contents);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        let output = TextEdit::multiline(&mut self.contents)
            .id(EDITOR_ID.into())
            .font(TextStyle::Monospace)
            .lock_focus(true)
            .hint_text("Your code here...")
            .frame(true)
            .desired_width(f32::INFINITY)
            .clip_text(true)
            .layouter(&mut layouter)
            .min_size(ui.available_size())
            .show(ui);

        let mut got_events = false;
        ui.input(|i| {
            for event in &i.events {
                self.add_to_event_history(event);
            }
            got_events = i.events.len() > 0;
        });
        if got_events {
            // Note that egui crashes if we call handle_event_history in the
            // ui.input callback above.
            self.handle_event_history(ui, &output);
            ui.ctx()
                .memory_mut(|mem| mem.request_focus(EDITOR_ID.into()));
        }
    }

    fn yank(&mut self, ui: &Ui, output: &TextEditOutput) {
        nih_log!("contents at time of kill() -> {:?}", self.contents);
    }
}

fn get_leading_whitespace(input: &str) -> usize {
    let mut spaces: usize = 0;
    for c in input.chars() {
        if c.is_whitespace() {
            if c == ' ' {
                spaces += 1 as usize;
            } else if c == '\t' {
                spaces += INDENT_SPACES as usize;
            }
        } else {
            return spaces;
        }
    }
    spaces
}

#[derive(Default)]
struct SyntaxHighlighter {}

impl SyntaxHighlighter {
    fn compute_layout(&mut self, contents: &str) -> LayoutJob {
        let mut job = LayoutJob::default();
        let mut lexer = SyntaxKind::lexer(contents);
        while let Some(token) = lexer.next() {
            let token_str = lexer.slice();
            job.append(
                token_str,
                0.0,
                TextFormat {
                    font_id: FontId::monospace(12.0),
                    color: get_color(&token),
                    ..Default::default()
                },
            );
        }
        job
    }
}

impl ComputerMut<&str, LayoutJob> for SyntaxHighlighter {
    fn compute(&mut self, contents: &str) -> LayoutJob {
        self.compute_layout(contents)
    }
}

fn get_color(tok: &SyntaxKind) -> Color32 {
    match *tok {
        tok if tok >= SyntaxKind::KW_LET && tok < SyntaxKind::KW_VAR => Color32::LIGHT_RED,
        tok if tok.is_reserved_keyword() => Color32::LIGHT_RED,
        SyntaxKind::COMMENT_LINE
        | SyntaxKind::COMMENT_LINE_DOC
        | SyntaxKind::COMMENT_BLOCK
        | SyntaxKind::COMMENT_BLOCK_DOC => Color32::GRAY,
        SyntaxKind::KW_FN => Color32::GOLD,
        SyntaxKind::LIT_STR => Color32::ORANGE,
        _ => Color32::WHITE,
    }
}

static FONT_DATA: &[u8] = include_bytes!("assets/fonts/Monaco.ttf");

fn setup_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    // Insert the font data
    fonts.font_data.insert(
        "Monaco".to_owned(),
        Arc::new(FontData::from_static(FONT_DATA)),
    );

    // Set the font to be used for proportional and monospace text
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Monaco".to_owned());

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("Monaco".to_owned());

    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod test {
    use crate::editor::*;
    use logos::Logos;
    use rhai_rowan::parser::Parser;
    use rhai_rowan::syntax::SyntaxKind;

    #[test]
    fn rhai_parsing() {
        let parse = Parser::new("fn foo() { let x = 1; }").parse_script();
        assert_eq!(parse.errors.len(), 0);
    }

    #[test]
    fn test_syntax_highlighting() {
        {
            let contents = r#"let foo = "bar";"#;
            let lexer = SyntaxKind::lexer(contents);
            assert_eq!(
                lexer.collect::<Vec<SyntaxKind>>(),
                vec![
                    SyntaxKind::KW_LET,
                    SyntaxKind::WHITESPACE,
                    SyntaxKind::IDENT,
                    SyntaxKind::WHITESPACE,
                    SyntaxKind::OP_ASSIGN,
                    SyntaxKind::WHITESPACE,
                    SyntaxKind::LIT_STR,
                    SyntaxKind::PUNCT_SEMI,
                ]
            );
        }
    }

    #[test]
    fn test_get_leading_whitespace() {
        assert_eq!(get_leading_whitespace("    let x = 1;"), 4);
        assert_eq!(get_leading_whitespace("\tlet x = 1;"), 4);
    }
}
