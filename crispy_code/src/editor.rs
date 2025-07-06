use crate::controller::Controller;
use crate::custom_text_edit::{TextEdit, TextEditOutput};
use crate::scripting::setup_engine;
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
use rhai::{Engine, Scope};
use rhai_rowan::syntax::SyntaxKind;
use std::sync::{Arc, Mutex};

const EDITOR_ID: &'static str = "main_text_editor";
const INDENT_SPACES: i32 = 4;
const WINDOW_SIZE: (u32, u32) = (1024, 768);

pub fn create_editor(controller: Arc<Controller>) -> Option<Box<dyn Editor>> {
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
            let scripting_engine = Arc::new(Mutex::new(setup_engine(Arc::clone(&controller))));

            setup_fonts(ctx);

            TopBottomPanel::bottom("console").show(ctx, |ui| {
                ui.label("things will be printed down here");
            });
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    state.show(ui, scripting_engine);
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
    Abort,
    Kill,
    Yank,
    BackToIndentation,
    BackWord,
    ForwardWord,
    Snippet,
    SnippetSend,
    Other,
}

struct TextEditor {
    contents: String,
    event_history: [Option<(Event, i32)>; 5],
    kill_buffer: Option<String>,
    layout_cache: Option<FrameCache<LayoutJob, SyntaxHighlighter>>,
    snippet_anchor: Option<usize>,
}

impl TextEditor {
    fn new() -> Self {
        Self {
            contents: String::new(),
            event_history: [None; 5],
            kill_buffer: None,
            layout_cache: None,
            snippet_anchor: None,
        }
    }

    fn abort(&mut self, ui: &Ui) {
        self.snippet_abort(ui);
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

    fn back_to_indentation(&mut self, ui: &Ui, output: &TextEditOutput) {
        if let Some(cursor_range) = output.cursor_range {
            let len = self.contents.len();
            let cursor_pos = cursor_range.primary.ccursor.index;
            nih_log!("cursor_pos -> {:?}, len -> {:?}", cursor_pos, len);
            nih_log!("back_to_indentation self.contents -> '{:?}'", self.contents);
            let buf = self.contents.as_str();
            let start_idx = match buf[..cursor_pos - 1].rfind("\n") {
                None => 0,
                Some(idx) => idx,
            };
            let leading_whitespace = get_leading_whitespace(&buf[start_idx..cursor_pos - 1]);
            nih_log!("leading_whitespace -> {:?}", leading_whitespace);
            if let Some(new_cursor_pos) = start_idx.checked_add(leading_whitespace + 1) {
                self.set_cursor_pos(ui, new_cursor_pos);
            }
            self.contents.remove(cursor_pos - 1);
        }
    }

    fn back_word(&mut self, ui: &Ui, output: &TextEditOutput) {
        if let Some(cursor_range) = output.cursor_range {
            let cursor_pos = cursor_range.primary.ccursor.index;
            if let Some(new_cursor_pos) =
                find_prev_word_beginning(self.contents.as_str(), cursor_pos)
            {
                self.set_cursor_pos(ui, new_cursor_pos);
            }
        }
    }

    fn clear_events(&mut self, idx: i32) {
        assert!(idx < self.event_history.len() as i32);
        for i in 0..idx {
            self.event_history[i as usize] = None;
        }
    }

    fn forward_word(&mut self, ui: &Ui, output: &TextEditOutput) {
        if let Some(cursor_range) = output.cursor_range {
            let cursor_pos = cursor_range.primary.ccursor.index;
            if let Some(new_cursor_pos) =
                find_next_word_beginning(self.contents.as_str(), cursor_pos)
            {
                self.set_cursor_pos(ui, new_cursor_pos);
            }
        }
    }

    fn handle_event_history(
        &mut self,
        ui: &Ui,
        scripting_engine: Arc<Mutex<Engine>>,
        output: &TextEditOutput,
    ) {
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
                self.outdent_closed_curly();
                self.clear_events(2)
            }
            _ => {}
        }
        match self.event_history[0] {
            Some((Event::KeyEnter, 1)) => {
                self.indent_to_match_previous_line(ui, output);
                self.clear_events(1)
            }
            Some((Event::Abort, _)) => {
                self.abort(ui);
                self.clear_events(1)
            }
            Some((Event::Kill, _)) => {
                self.kill(output);
                self.clear_events(1)
            }
            Some((Event::Yank, _)) => {
                self.yank(ui, output);
                self.clear_events(1)
            }
            Some((Event::BackToIndentation, _)) => {
                self.back_to_indentation(ui, output);
                self.clear_events(1)
            }
            Some((Event::BackWord, _)) => {
                self.back_word(ui, output);
                self.clear_events(1)
            }
            Some((Event::ForwardWord, _)) => {
                self.forward_word(ui, output);
                self.clear_events(1)
            }
            Some((Event::Snippet, _)) => {
                self.snippet(output);
                self.clear_events(1)
            }
            Some((Event::SnippetSend, _)) => {
                self.snippet_send(ui, scripting_engine);
                self.clear_events(1)
            }
            _ => {}
        }
    }

    fn indent_cursor(&mut self, ui: &Ui, output: &TextEditOutput) {
        if let Some(cursor_range) = output.cursor_range {
            let cursor_pos = cursor_range.primary.ccursor.index;
            self.contents
                .push_str(" ".repeat(INDENT_SPACES as usize).as_str());
            if let Some(new_cursor_pos) = cursor_pos.checked_add(INDENT_SPACES as usize) {
                self.set_cursor_pos(ui, new_cursor_pos);
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
                "µ" => Event::BackToIndentation,
                "∫" => Event::BackWord,
                "ƒ" => Event::ForwardWord,
                _ => Event::Other,
            },
            egui::Event::Key {
                pressed: false, // trigger events when the key is released
                key,
                modifiers,
                ..
            } => match key {
                egui::Key::Enter if modifiers.ctrl => Event::SnippetSend,
                egui::Key::Enter => Event::KeyEnter,
                egui::Key::G if modifiers.ctrl => Event::Abort,
                egui::Key::K if modifiers.ctrl => Event::Kill,
                egui::Key::Y if modifiers.ctrl => Event::Yank,
                egui::Key::Space if modifiers.ctrl => Event::Snippet,
                // These matches that look for modifiers.alt never trigger
                egui::Key::M if modifiers.alt => Event::BackToIndentation,
                egui::Key::F if modifiers.alt => Event::ForwardWord,
                egui::Key::B if modifiers.alt => Event::BackWord,
                _ => Event::Other,
            },
            _ => Event::Other,
        }
    }

    fn kill(&mut self, output: &TextEditOutput) {
        if let Some(cursor_range) = output.cursor_range {
            let start_idx = cursor_range.primary.ccursor.index;
            let buf = self.contents.as_str();
            let end_idx = match buf[start_idx..].find("\n") {
                None => self.contents.len(),
                Some(idx) => idx + start_idx,
            };
            if end_idx > start_idx {
                self.kill_buffer = Some(buf[start_idx..end_idx].to_string());
                self.contents.replace_range(start_idx..end_idx, "");
            } else {
                nih_log!("end_idx ({:?}) <= start_idx ({:?})", end_idx, start_idx);
            }
        } else {
            nih_log!("output.cursor_range is None");
        }
    }

    fn outdent_closed_curly(&mut self) {
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

    fn set_cursor_pos(&self, ui: &Ui, new_cursor_pos: usize) {
        if let Some(mut state) = TextEdit::load_state(ui.ctx(), EDITOR_ID.into()) {
            let ccursor = CCursor::new(new_cursor_pos);
            state
                .cursor
                .set_char_range(Some(CCursorRange::one(ccursor)));
            state.store(ui.ctx(), EDITOR_ID.into());
        }
    }

    fn show(&mut self, ui: &mut Ui, scripting_engine: Arc<Mutex<Engine>>) {
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
            .frame(false)
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
            self.handle_event_history(ui, scripting_engine, &output);
            self.snippet_highlight(ui);
        }
    }

    fn snippet(&mut self, output: &TextEditOutput) {
        if let Some(cursor_range) = output.cursor_range {
            self.snippet_anchor = Some(cursor_range.primary.ccursor.index);
        }
    }

    fn snippet_abort(&mut self, ui: &Ui) {
        if let Some(mut state) = TextEdit::load_state(ui.ctx(), EDITOR_ID.into()) {
            if let Some(curr_char_range) = state.cursor.char_range() {
                state
                    .cursor
                    .set_char_range(Some(CCursorRange::one(curr_char_range.primary)));
                state.store(ui.ctx(), EDITOR_ID.into());
            }
        }
        self.snippet_anchor = None;
    }

    fn snippet_send(&mut self, ui: &Ui, scripting_engine: Arc<Mutex<Engine>>) {
        let anchor = self.snippet_anchor.unwrap();
        let mut scope = Scope::new();

        if let Some(state) = TextEdit::load_state(ui.ctx(), EDITOR_ID.into()) {
            if let Some(char_range) = state.cursor.char_range() {
                let cursor_pos = char_range.primary.index;
                if anchor == cursor_pos {
                    return;
                }
                let (start, end) = if anchor < cursor_pos {
                    (CCursor::new(anchor), CCursor::new(cursor_pos))
                } else {
                    (CCursor::new(cursor_pos), CCursor::new(anchor))
                };
                let engine = scripting_engine.lock().unwrap();
                let snippet_contents = &self.contents.as_str()[start.index..end.index];

                if let Err(err) = engine.run_with_scope(&mut scope, &snippet_contents) {
                    nih_log!("error executing snippet contents: {:?}", err);
                } else {
                    nih_log!("executed snippet contents: {:?}", snippet_contents);
                }
            }
        }
        self.snippet_abort(ui);
    }

    fn snippet_highlight(&mut self, ui: &Ui) {
        if self.snippet_anchor.is_none() {
            return;
        }
        let anchor = self.snippet_anchor.unwrap();
        if let Some(mut state) = TextEdit::load_state(ui.ctx(), EDITOR_ID.into()) {
            if let Some(char_range) = state.cursor.char_range() {
                let cursor_pos = char_range.primary.index;
                if anchor == cursor_pos {
                    return;
                }
                let (start, end) = if anchor < cursor_pos {
                    (CCursor::new(anchor), CCursor::new(cursor_pos))
                } else {
                    (CCursor::new(cursor_pos), CCursor::new(anchor))
                };
                state
                    .cursor
                    .set_char_range(Some(CCursorRange::two(start, end)));
                state.store(ui.ctx(), EDITOR_ID.into());
            }
        }
    }

    fn yank(&mut self, ui: &Ui, output: &TextEditOutput) {
        if self.kill_buffer.is_none() {
            return;
        }
        if let Some(buf) = &self.kill_buffer {
            let kb_size = buf.len();
            if let Some(cursor_range) = output.cursor_range {
                let start_idx = cursor_range.primary.ccursor.index;
                self.contents.insert_str(start_idx, buf.as_str());
                self.set_cursor_pos(ui, kb_size + start_idx);
            }
        }
    }
}

fn find_next_word_beginning(input: &str, pos: usize) -> Option<usize> {
    if pos > input.len() {
        return None;
    }
    let mut saw_nonword = false;

    for (idx, chr) in input.chars().enumerate() {
        if idx < pos {
            continue;
        }
        if !chr.is_alphanumeric() {
            saw_nonword = true;
            continue;
        }
        if chr.is_alphanumeric() && saw_nonword {
            return Some(idx as usize);
        }
    }
    Some(input.len() - 1 as usize)
}

fn find_prev_word_beginning(input: &str, pos: usize) -> Option<usize> {
    if pos > input.len() {
        return None;
    }
    let mut saw_word = false;

    for (idx, chr) in input.chars().rev().enumerate() {
        if idx < input.len() - pos {
            continue;
        }
        if chr.is_alphanumeric() {
            saw_word = true;
            continue;
        }
        if !chr.is_alphanumeric() && saw_word {
            return Some(input.len() - idx as usize);
        }
    }
    Some(0 as usize)
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
    use crate::controller::Controller;
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

    #[test]
    fn test_find_next_word_beginning() {
        let contents = r#"fn foo() { return 1; }"#;
        let current_cursor_pos = 3 as usize;
        assert_eq!(
            find_next_word_beginning(contents, current_cursor_pos),
            Some(11 as usize)
        );
    }

    #[test]
    fn test_find_prev_word_beginning() {
        {
            let contents = r#"fn foo() { return 1; }"#;
            let current_cursor_pos = 11 as usize;
            assert_eq!(
                find_prev_word_beginning(contents, current_cursor_pos),
                Some(3 as usize)
            );
        }
        {
            let contents = r#"fn foo() { return 1; }"#;
            let current_cursor_pos = 3 as usize;
            assert_eq!(
                find_prev_word_beginning(contents, current_cursor_pos),
                Some(0 as usize)
            );
        }
    }

    #[test]
    fn test_create_editor() {
        let (controller, _) = Controller::new();
        let editor = create_editor(controller);
        assert!(editor.is_some());
    }
}
