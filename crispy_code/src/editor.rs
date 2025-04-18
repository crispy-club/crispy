use crate::controller::Controller;
use logos::Logos;
use nih_plug::prelude::Editor;
use nih_plug_egui::{
    create_egui_editor,
    egui::{
        cache::{ComputerMut, FrameCache},
        text::LayoutJob,
        CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Galley,
        ScrollArea, TextEdit, TextFormat, TextStyle, TopBottomPanel, Ui,
    },
    EguiState,
};
use std::sync::Arc;

const WINDOW_SIZE: (u32, u32) = (1024, 768);

#[derive(Default)]
struct TextEditor {
    contents: String,
    layout_cache: FrameCache<LayoutJob, SyntaxHighlighter>,
}

pub fn create_editor(_controller: Arc<Controller>) -> Option<Box<dyn Editor>> {
    let egui_state = EguiState::from_size(WINDOW_SIZE.0, WINDOW_SIZE.1);
    let syntax_highlighter = SyntaxHighlighter::default();
    let editor = TextEditor {
        contents: String::new(),
        layout_cache: FrameCache::new(syntax_highlighter),
    };
    // let mut layout_cache: FrameCache<LayoutJob, SyntaxHighlighter> =
    //     FrameCache::new(syntax_highlighter);

    create_egui_editor(
        egui_state.clone(),
        editor,
        |_, _| {},
        move |ctx, _setter, state: &mut TextEditor| {
            setup_fonts(ctx);

            TopBottomPanel::bottom("console").show(ctx, |ui| {
                ui.label("things will be printed down here");
            });
            let mut layouter = |ui: &Ui, contents: &str, wrap_width: f32| -> Arc<Galley> {
                let mut layout_job: LayoutJob = state.layout_cache.get(contents);
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let text_edit = TextEdit::multiline(&mut state.contents)
                        .font(TextStyle::Monospace)
                        .lock_focus(true)
                        .hint_text("Your code here...")
                        .frame(true)
                        .desired_width(f32::INFINITY)
                        .clip_text(true)
                        .layouter(&mut layouter);
                    ui.add_sized(ui.available_size(), text_edit);
                });
            });
        },
    )
}

#[derive(Clone, Debug, Logos, PartialEq)]
pub enum RhaiToken {
    #[regex(r"(if|else|while|for|in|let|const|return|break|continue|true|false|null|this)")]
    Keyword,
    #[token(";")]
    Semicolon,
    #[regex(r"[ \t]")]
    Whitespace,
    #[regex(r"[\r\n\f]")]
    Newline,
    #[regex(r"[a-zA-Z][_a-zA-Z0-9]*")]
    VariableName,
    #[token("=")]
    Assignment,
    #[regex(r#"//.*"#)]
    Comment,
    #[token("fn")]
    Fn,
    #[regex(r#""[^"]*""#)]
    NormalString,
    // Only support a single '#' for now
    #[regex(r##"#"[^("#)]*"#"##)]
    RawString,
    #[regex(r"[0-9]+")]
    Numeric,
}

#[derive(Default)]
struct SyntaxHighlighter {}

impl ComputerMut<&str, LayoutJob> for SyntaxHighlighter {
    fn compute(&mut self, key: &str) -> LayoutJob {
        rhai_syntax_highlighting(key)
    }
}

fn rhai_syntax_highlighting(contents: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut lexer = RhaiToken::lexer(contents);
    while let Some(token) = lexer.next() {
        let token_str = lexer.slice();
        job.append(
            token_str,
            0.0,
            TextFormat {
                font_id: FontId::monospace(12.0),
                color: get_color(token),
                ..Default::default()
            },
        );
    }
    job
}

fn get_color<E>(tok: Result<RhaiToken, E>) -> Color32 {
    match tok {
        Ok(RhaiToken::Keyword) => Color32::LIGHT_RED,
        Ok(RhaiToken::Comment) => Color32::GRAY,
        Ok(RhaiToken::Whitespace | RhaiToken::Newline) => Color32::BLACK,
        Ok(
            RhaiToken::VariableName
            | RhaiToken::Assignment
            | RhaiToken::Semicolon
            | RhaiToken::Numeric,
        ) => Color32::WHITE,
        Ok(RhaiToken::Fn) => Color32::GOLD,
        Ok(RhaiToken::NormalString | RhaiToken::RawString) => Color32::ORANGE,
        // Error can happen when the lexer encounters an incomplete pattern.
        // Fallback to white.
        Err(_) => Color32::WHITE,
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

    #[test]
    fn test_syntax_highlighting() {
        {
            let contents = r#"let foo = "bar";"#;
            let lexer = RhaiToken::lexer(contents);
            assert_eq!(
                lexer.map(|res| res.unwrap()).collect::<Vec<RhaiToken>>(),
                vec![
                    RhaiToken::Keyword,
                    RhaiToken::Whitespace,
                    RhaiToken::VariableName,
                    RhaiToken::Whitespace,
                    RhaiToken::Assignment,
                    RhaiToken::Whitespace,
                    RhaiToken::NormalString,
                    RhaiToken::Semicolon,
                ]
            );
        }
        {
            let contents = r#"let foo = "bar";
// this is a comment"#;
            let lexer = RhaiToken::lexer(contents);
            assert_eq!(
                lexer.map(|res| res.unwrap()).collect::<Vec<RhaiToken>>(),
                vec![
                    RhaiToken::Keyword,
                    RhaiToken::Whitespace,
                    RhaiToken::VariableName,
                    RhaiToken::Whitespace,
                    RhaiToken::Assignment,
                    RhaiToken::Whitespace,
                    RhaiToken::NormalString,
                    RhaiToken::Semicolon,
                    RhaiToken::Newline,
                    RhaiToken::Comment,
                ]
            );
        }
        {
            let contents = r##"let bar = #"this is a raw string"#;"##;
            let lexer = RhaiToken::lexer(contents);
            assert_eq!(
                lexer.map(|res| res.unwrap()).collect::<Vec<RhaiToken>>(),
                vec![
                    RhaiToken::Keyword,
                    RhaiToken::Whitespace,
                    RhaiToken::VariableName,
                    RhaiToken::Whitespace,
                    RhaiToken::Assignment,
                    RhaiToken::Whitespace,
                    RhaiToken::RawString,
                    RhaiToken::Semicolon,
                ]
            );
        }
    }
}
