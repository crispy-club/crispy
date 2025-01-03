use crate::pattern::{NamedPattern, Pattern};
use axum::{extract::Path, extract::State, http::StatusCode, response, Json};
use rtrb::{Consumer, Producer};
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Command {
    PatternList(Vec<String>),
    PatternStart(NamedPattern),
    PatternStop(String),
}

#[allow(dead_code)]
pub struct Controller {
    pub commands: Mutex<Producer<Command>>,
    pub responses: Mutex<Consumer<Command>>,
}

#[axum::debug_handler]
pub async fn start_pattern(
    State(controller): State<Arc<Controller>>,
    Path(pattern_name): Path<String>,
    Json(pattern): Json<Pattern>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands.lock().unwrap();
    // TODO: handle when the queue is full

    let named_pattern = match pattern.channel {
        Some(channel) => NamedPattern {
            channel: channel,
            name: pattern_name,
            events: pattern.events,
            length_bars: pattern.length_bars,
        },
        None => NamedPattern {
            channel: 1,
            name: pattern_name,
            events: pattern.events,
            length_bars: pattern.length_bars,
        },
    };
    match cmds.push(Command::PatternStart(named_pattern)) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
pub async fn stop_pattern(
    State(controller): State<Arc<Controller>>,
    Path(pattern_name): Path<String>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands.lock().unwrap();
    // TODO: handle when the queue is full
    match cmds.push(Command::PatternStop(pattern_name)) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
