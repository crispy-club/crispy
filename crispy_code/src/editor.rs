use crate::controller::Controller;
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
        widgets::text_edit::TextEditOutput,
        CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Galley,
        ScrollArea, TextEdit, TextFormat, TextStyle, TopBottomPanel, Ui,
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
    KeyOpenCurly,
    KeyEnter,
    KeySpaceBar,
    Other,
}

struct TextEditor {
    contents: String,
    event_history: [Option<(Event, i32)>; 5],
    layout_cache: Option<FrameCache<LayoutJob, SyntaxHighlighter>>,
}

impl TextEditor {
    fn new() -> Self {
        Self {
            contents: String::new(),
            event_history: [None; 5],
            layout_cache: None,
        }
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
            self.handle_event_history(ui, &output);
            ui.ctx()
                .memory_mut(|mem| mem.request_focus(EDITOR_ID.into()));
        }
    }

    fn to_internal_event(event: &egui::Event) -> Event {
        nih_log!("converting event {:?} to internal event", event);
        match event {
            egui::Event::Text(text) => match text.as_str() {
                "{" => Event::KeyOpenCurly,
                " " => Event::KeySpaceBar,
                _ => Event::Other,
            },
            egui::Event::Key {
                pressed: false, // trigger events when the key is released
                key,
                ..
            } => match key {
                egui::Key::Enter => Event::KeyEnter,
                _ => Event::Other,
            },
            _ => Event::Other,
        }
    }

    fn add_to_event_history(&mut self, event: &egui::Event) {
        let i_event = Self::to_internal_event(event);

        if i_event == Event::Other {
            return;
        }
        nih_log!("got internal event {:?}", i_event);

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

    fn prepend_event(&mut self, i_event: Event) {
        for idx in (1..self.event_history.len()).rev() {
            self.event_history[idx] = self.event_history[idx - 1];
        }
        self.event_history[0] = Some((i_event, 1));
    }

    fn handle_event_history(&mut self, ui: &Ui, output: &TextEditOutput) {
        // One of the tricky things about this method is it will need to reset the event history
        // because we only want to act once on a given set of events.
        // The naive thing to do would be to set the events to None.
        // This actually seems fine.
        // Are there issues with having mixed None/Some in the history?
        // I don't think there will be....
        nih_log!("handle event history {:?}", self.event_history);
        match self.event_history[0] {
            Some(first_event) => match self.event_history[1] {
                Some(second_event) => match self.event_history[2] {
                    Some(third_event) => {
                        match (first_event, second_event, third_event) {
                            (
                                (Event::KeyEnter, 1),
                                (Event::KeyOpenCurly, 1),
                                (Event::KeySpaceBar, _),
                            ) => {
                                // indent cursor
                                nih_log!("indenting cursor");
                                self.indent_cursor(ui, output);
                                nih_log!("clearing events");
                                self.event_history[0] = None;
                                self.event_history[1] = None;
                                self.event_history[2] = None;
                            }
                            _ => {
                                nih_log!(
                                    "taking no action! {:?} {:?} {:?}",
                                    first_event,
                                    second_event,
                                    third_event
                                );
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    fn indent_cursor(&mut self, ui: &Ui, output: &TextEditOutput) {
        if let Some(cursor_range) = output.cursor_range {
            let cursor_pos = cursor_range.primary.ccursor.index;
            if let Some(new_cursor_pos) = cursor_pos.checked_add(INDENT_SPACES as usize) {
                nih_log!("going to set new cursor position {:?}", new_cursor_pos);
                if let Some(mut state) = TextEdit::load_state(ui.ctx(), EDITOR_ID.into()) {
                    nih_log!("loaded state");
                    self.contents
                        .push_str(" ".repeat(INDENT_SPACES as usize).as_str());
                    let ccursor = CCursor::new(new_cursor_pos);
                    state
                        .cursor
                        .set_char_range(Some(CCursorRange::one(ccursor)));
                    nih_log!("storing state");
                    state.store(ui.ctx(), EDITOR_ID.into());
                }
            }
        }
    }
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
}
