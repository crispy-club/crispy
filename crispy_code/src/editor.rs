use crate::controller::Controller;
use iced_baseview::baseview::WindowOpenOptions; //, WindowScalePolicy};
                                                // use crossbeam::channel;
use crossbeam::atomic::AtomicCell;
// use iced::futures::FutureExt;
use iced::highlighter;
use iced::keyboard;
use iced::widget::{
    button, center, column, container, horizontal_space, pick_list, row, text, text_editor,
    toggler, tooltip,
};
use iced::{Center, Element, Fill, Font, Renderer, Subscription, Theme};
use iced_baseview::baseview::WindowScalePolicy;
use iced_baseview::settings::{IcedBaseviewSettings, Settings};
use iced_baseview::window::{WindowHandle, WindowSubs};
use iced_baseview::Application;
use iced_runtime::Task;
use nih_plug::prelude::{Editor, GuiContext, ParentWindowHandle};
use std::ffi;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct CodeEditor {
    pub(crate) controller: Arc<Controller>,
    pub(crate) scaling_factor: AtomicCell<Option<f32>>,

    file: Option<PathBuf>,
    content: text_editor::Content,
    theme: highlighter::Theme,
    word_wrap: bool,
    is_loading: bool,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ActionPerformed(text_editor::Action),
    ThemeSelected(highlighter::Theme),
    WordWrapToggled(bool),
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    FileSaved(Result<PathBuf, Error>),
}

// impl std::fmt::Debug for Message {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::ActionPerformed(action) => write!(f, "text_editor::Action {:?}", action),
//             Self::ThemeSelected(theme) => write!(f, "highlighter::Theme {:?}", theme),
//             Self::WordWrapToggled(toggled) => write!(f, "WordWrapToggled({:?})", toggled),
//             Self::NewFile => write!(f, "NewFile"),
//             Self::OpenFile => write!(f, "OpenFile"),
//             Self::FileOpened(result) => write!(f, "FileOpened({:?})", result),
//             Self::SaveFile => write!(f, "SaveFile"),
//             Self::FileSaved(result) => write!(f, "FileSaved({:?})", result),
//         }
//     }
// }

// impl Clone for Message {
//     fn clone(&self) -> Self {
//         match self {
//             Self::Foo => Self::Foo,
//         }
//     }
// }

/// The window handle used for [`IcedEditorWrapper`].
struct IcedEditorHandle<Message: 'static + Send> {
    window: WindowHandle<Message>,
}

/// The window handle enum stored within 'WindowHandle' contains raw pointers. Is there a way around
/// having this requirement?
unsafe impl<Message: Send> Send for IcedEditorHandle<Message> {}

impl<Message: Send> Drop for IcedEditorHandle<Message> {
    fn drop(&mut self) {
        self.window.close_window();
    }
}

impl Editor for CodeEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let (unscaled_width, unscaled_height) = (200, 150);

        let scaling_factor = self.scaling_factor.load();

        // TODO: iced_baseview does not have gracefuly error handling for context creation failures.
        //       This will panic if the context could not be created.
        let window = iced_baseview::open_parented::<IcedApplication, ParentWindowHandle>(
            &parent,
            self.controller.clone(),
            Settings {
                window: WindowOpenOptions {
                    title: String::from("CODE"),
                    // Baseview should be doing the DPI scaling for us
                    size: iced_baseview::baseview::Size::new(
                        unscaled_width as f64,
                        unscaled_height as f64,
                    ),
                    scale: scaling_factor
                        .map(|factor| WindowScalePolicy::ScaleFactor(factor as f64))
                        .unwrap_or(WindowScalePolicy::SystemScaleFactor),
                    // gl_config: None,
                },
                iced_baseview: IcedBaseviewSettings {
                    ignore_non_modifier_keys: false,
                    always_redraw: true,
                },
                graphics_settings: iced_graphics::Settings {
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        Box::new(IcedEditorHandle { window })
    }

    fn size(&self) -> (u32, u32) {
        (200, 150)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        true
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}

impl CodeEditor {
    pub fn new(controller: Arc<Controller>) -> Self {
        Self {
            controller: controller,

            #[cfg(target_os = "macos")]
            scaling_factor: AtomicCell::new(None),
            #[cfg(not(target_os = "macos"))]
            scaling_factor: AtomicCell::new(Some(1.0)),

            file: None,
            content: text_editor::Content::new(),
            theme: highlighter::Theme::SolarizedDark,
            word_wrap: true,
            is_loading: true,
            is_dirty: false,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();

                self.content.perform(action);

                Task::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Task::none()
            }
            Message::WordWrapToggled(word_wrap) => {
                self.word_wrap = word_wrap;

                Task::none()
            }
            Message::NewFile => {
                if !self.is_loading {
                    self.file = None;
                    self.content = text_editor::Content::new();
                }

                Task::none()
            }
            Message::OpenFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    Task::perform(open_file(), Message::FileOpened)
                }
            }
            Message::FileOpened(result) => {
                self.is_loading = false;
                self.is_dirty = false;

                if let Ok((path, contents)) = result {
                    self.file = Some(path);
                    self.content = text_editor::Content::with_text(&contents);
                }

                Task::none()
            }
            Message::SaveFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    let text = self.content.text();

                    // Not sure what this block does!
                    // It doesn't compile though because the line_ending()
                    // method has been removed.
                    //
                    // if let Some(ending) = self.content.line_ending() {
                    //     if !text.ends_with(ending.as_str()) {
                    //         text.push_str(ending.as_str());
                    //     }
                    // }

                    Task::perform(save_file(self.file.clone(), text), Message::FileSaved)
                }
            }
            Message::FileSaved(result) => {
                self.is_loading = false;

                if let Ok(path) = result {
                    self.file = Some(path);
                    self.is_dirty = false;
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            action(new_icon(), "New file", Some(Message::NewFile)),
            action(
                open_icon(),
                "Open file",
                (!self.is_loading).then_some(Message::OpenFile)
            ),
            action(
                save_icon(),
                "Save file",
                self.is_dirty.then_some(Message::SaveFile)
            ),
            horizontal_space(),
            toggler(self.word_wrap)
                .label("Word Wrap")
                .on_toggle(Message::WordWrapToggled),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.theme),
                Message::ThemeSelected
            )
            .text_size(14)
            .padding([5, 10])
        ]
        .spacing(10)
        .align_y(Center);

        let status = row![
            text(if let Some(path) = &self.file {
                let path = path.display().to_string();

                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ]
        .spacing(10);

        column![
            controls,
            text_editor(&self.content)
                .height(Fill)
                .on_action(Message::ActionPerformed)
                .wrapping(if self.word_wrap {
                    text::Wrapping::Word
                } else {
                    text::Wrapping::None
                })
                .highlight(
                    self.file
                        .as_deref()
                        .and_then(Path::extension)
                        .and_then(ffi::OsStr::to_str)
                        .unwrap_or("rs"),
                    self.theme,
                )
                .key_binding(|key_press| {
                    match key_press.key.as_ref() {
                        keyboard::Key::Character("s") if key_press.modifiers.command() => {
                            Some(text_editor::Binding::Custom(Message::SaveFile))
                        }
                        _ => text_editor::Binding::from_key_press(key_press),
                    }
                }),
            status,
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

async fn open_file() -> Result<(PathBuf, Arc<String>), Error> {
    let picked_file = rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(picked_file).await
}

async fn load_file(path: impl Into<PathBuf>) -> Result<(PathBuf, Arc<String>), Error> {
    let path = path.into();

    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, contents: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(Path::to_owned)
            .ok_or(Error::DialogClosed)?
    };

    tokio::fs::write(&path, contents)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(center(content).width(30));

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(container::rounded_box)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

pub(crate) struct IcedApplication {
    editor: CodeEditor,
}

impl Application for IcedApplication {
    type Executor = iced_baseview::executor::Default;
    type Message = Message;
    type Flags = Arc<Controller>;
    type Theme = Theme;

    fn new(controller: Self::Flags) -> (Self, Task<Self::Message>) {
        let editor = CodeEditor::new(controller);
        (Self { editor }, Task::none())
    }

    fn title(&self) -> String {
        String::from("CODE")
    }

    fn update(&mut self, _message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    fn subscription(
        &self,
        _window_subs: &mut WindowSubs<Self::Message>,
    ) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        self.editor.view().into()
    }

    fn scale_policy(&self) -> WindowScalePolicy {
        WindowScalePolicy::SystemScaleFactor // use custom scaling factor?
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
