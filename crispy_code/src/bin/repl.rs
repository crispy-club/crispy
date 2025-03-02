use crispy_code::dsl::notes;
use crispy_code::pattern::NamedPattern;
use crispy_code::plugin::play;
use rhai::{Dynamic, Engine, EvalAltResult};
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::{History, SearchDirection};
use rustyline::{Cmd, DefaultEditor, Event, EventHandler, KeyCode, KeyEvent, Modifiers, Movement};

const HISTORY_FILE: &str = ".rhai-repl-history";

/// Print help text.
fn print_help() {
    println!("help       => print this help");
    println!("quit, exit => quit");
    println!("keys       => print list of key bindings");
    println!("history    => print lines history");
    println!("!!         => repeat the last history line");
    println!("!<#>       => repeat a particular history line");
    println!("!<text>    => repeat the last history line starting with some text");
    println!("!?<text>   => repeat the last history line containing some text");
    println!("strict     => toggle on/off Strict Variables Mode");
    println!();
    println!("press Ctrl-Enter or end a line with `\\`");
    println!("to continue to the next line.");
    println!();
}

/// Print key bindings.
fn print_keys() {
    println!("Home              => move to beginning of line");
    println!("Ctrl-Home         => move to beginning of input");
    println!("End               => move to end of line");
    println!("Ctrl-End          => move to end of input");
    println!("Left              => move left");
    println!("Ctrl-Left         => move left by one word");
    println!("Right             => move right by one word");
    println!("Ctrl-Right        => move right");
    println!("Up                => previous line or history");
    println!("Ctrl-Up           => previous history");
    println!("Down              => next line or history");
    println!("Ctrl-Down         => next history");
    println!("Ctrl-R            => reverse search history");
    println!("                     (Ctrl-S forward, Ctrl-G cancel)");
    println!("Ctrl-L            => clear screen");
    #[cfg(target_family = "windows")]
    println!("Escape            => clear all input");
    println!("Ctrl-C            => exit");
    println!("Ctrl-D            => EOF (when line empty)");
    println!("Ctrl-H, Backspace => backspace");
    println!("Ctrl-D, Del       => delete character");
    println!("Ctrl-U            => delete from start");
    println!("Ctrl-W            => delete previous word");
    println!("Ctrl-T            => transpose characters");
    println!("Ctrl-V            => insert special character");
    println!("Ctrl-Y            => paste yank");
    #[cfg(target_family = "unix")]
    println!("Ctrl-Z            => suspend");
    #[cfg(target_family = "windows")]
    println!("Ctrl-Z            => undo");
    println!("Ctrl-_            => undo");
    println!("Enter             => run code");
    println!("Shift-Ctrl-Enter  => continue to next line");
    println!();
    println!("Plus all standard Emacs key bindings");
    println!();
}

// Setup the Rustyline editor.
fn setup_editor() -> DefaultEditor {
    //env_logger::init();
    let config = Builder::new()
        .tab_stop(4)
        .indent_size(4)
        .bracketed_paste(true)
        .build();
    let mut rl = DefaultEditor::with_config(config).unwrap();

    // Bind more keys

    // On Windows, Esc clears the input buffer
    #[cfg(target_family = "windows")]
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Esc, Modifiers::empty())]),
        EventHandler::Simple(Cmd::Kill(Movement::WholeBuffer)),
    );
    // On Windows, Ctrl-Z is undo
    #[cfg(target_family = "windows")]
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent::ctrl('z')]),
        EventHandler::Simple(Cmd::Undo(1)),
    );
    // Map Ctrl-Enter to insert a new line - bypass need for `\` continuation
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Char('J'), Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::Newline),
    );
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Enter, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::Newline),
    );
    // Map Ctrl-Home and Ctrl-End for beginning/end of input
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Home, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::Move(Movement::BeginningOfBuffer)),
    );
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::End, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::Move(Movement::EndOfBuffer)),
    );
    // Map Ctrl-Up and Ctrl-Down to skip up/down the history, even through multi-line histories
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Down, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::NextHistory),
    );
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent(KeyCode::Up, Modifiers::CTRL)]),
        EventHandler::Simple(Cmd::PreviousHistory),
    );

    // Load the history file
    if rl.load_history(HISTORY_FILE).is_err() {
        eprintln!("! No previous lines history!");
    }

    rl
}

fn main() {
    // Initialize scripting engine
    let mut engine = Engine::new();

    engine.build_type::<NamedPattern>();

    engine.register_fn("notes", notes);
    engine.register_fn(
        "play",
        |np: NamedPattern| -> Result<(), Box<EvalAltResult>> {
            play(np).map_err(|e| {
                Box::new(EvalAltResult::ErrorRuntime(
                    Dynamic::from(e.to_string()),
                    rhai::Position::NONE,
                ))
            })
        },
    );

    // REPL line editor setup
    let mut rl = setup_editor();

    // REPL loop
    let mut input = String::new();
    let mut replacement = None;
    let mut replacement_index = 0;
    let mut history_offset = 1;

    print_help();

    'main_loop: loop {
        if let Some(replace) = replacement.take() {
            input = replace;
            if rl
                .add_history_entry(input.clone())
                .expect("Failed to add history entry")
            {
                history_offset += 1;
            }
            if input.contains('\n') {
                println!("[{replacement_index}] ~~~~");
                println!("{input}");
                println!("~~~~");
            } else {
                println!("[{replacement_index}] {input}");
            }
            replacement_index = 0;
        } else {
            input.clear();

            loop {
                let prompt = if input.is_empty() { "repl> " } else { "    > " };

                match rl.readline(prompt) {
                    // Line continuation
                    Ok(mut line) if line.ends_with('\\') => {
                        line.pop();
                        input += &line;
                        input += "\n";
                    }
                    Ok(line) => {
                        input += &line;
                        let cmd = input.trim();
                        if !cmd.is_empty()
                            && !cmd.starts_with('!')
                            && cmd.trim() != "history"
                            && rl
                                .add_history_entry(input.clone())
                                .expect("Failed to add history entry")
                        {
                            history_offset += 1;
                        }
                        break;
                    }

                    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break 'main_loop,

                    Err(err) => {
                        eprintln!("Error: {err:?}");
                        break 'main_loop;
                    }
                }
            }
        }

        let cmd = input.trim();

        if cmd.is_empty() {
            continue;
        }

        // Implement standard commands
        match cmd {
            "help" => {
                print_help();
                continue;
            }
            "keys" => {
                print_keys();
                continue;
            }
            "exit" | "quit" => break, // quit
            "history" => {
                for (i, h) in rl.history().iter().enumerate() {
                    match &h.lines().collect::<Vec<_>>()[..] {
                        [line] => println!("[{}] {line}", history_offset + i),
                        lines => {
                            for (x, line) in lines.iter().enumerate() {
                                let number = format!("[{}]", history_offset + i);
                                if x == 0 {
                                    println!("{number} {}", line.trim_end());
                                } else {
                                    println!("{0:>1$} {2}", "", number.len(), line.trim_end());
                                }
                            }
                        }
                    }
                }
                continue;
            }
            "strict" if engine.strict_variables() => {
                engine.set_strict_variables(false);
                println!("Strict Variables Mode turned OFF.");
                continue;
            }
            "strict" => {
                engine.set_strict_variables(true);
                println!("Strict Variables Mode turned ON.");
                continue;
            }
            "!!" => {
                match rl.history().iter().last() {
                    Some(line) => {
                        replacement = Some(line.clone());
                        replacement_index = history_offset + rl.history().len() - 1;
                    }
                    None => eprintln!("No lines history!"),
                }
                continue;
            }
            _ if cmd.starts_with("!?") => {
                let text = cmd[2..].trim();
                let history = rl
                    .history()
                    .iter()
                    .rev()
                    .enumerate()
                    .find(|(.., h)| h.contains(text));

                match history {
                    Some((n, line)) => {
                        replacement = Some(line.clone());
                        replacement_index = history_offset + (rl.history().len() - 1 - n);
                    }
                    None => eprintln!("History line not found: {text}"),
                }
                continue;
            }
            _ if cmd.starts_with('!') => {
                if let Ok(num) = cmd[1..].parse::<usize>() {
                    if num >= history_offset {
                        if let Some(line) = rl
                            .history()
                            .get(num - history_offset, SearchDirection::Forward)
                            .expect("Failed to get history entry")
                        {
                            replacement = Some(line.entry.into());
                            replacement_index = num;
                            continue;
                        }
                    }
                } else {
                    let prefix = cmd[1..].trim();
                    if let Some((n, line)) = rl
                        .history()
                        .iter()
                        .rev()
                        .enumerate()
                        .find(|(.., h)| h.trim_start().starts_with(prefix))
                    {
                        replacement = Some(line.clone());
                        replacement_index = history_offset + (rl.history().len() - 1 - n);
                        continue;
                    }
                }
                eprintln!("History line not found: {}", &cmd[1..]);
                continue;
            }
            _ => (),
        }
    }

    rl.save_history(HISTORY_FILE)
        .expect("Failed to save history");

    println!("Bye!");
}
