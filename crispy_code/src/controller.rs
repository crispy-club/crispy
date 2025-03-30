use crate::pattern::{NamedPattern, Pattern};
use axum::{
    extract::Path, extract::State, http::StatusCode, response, routing::post, Json, Router,
};
use rtrb::Producer;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    // Would be cool to have this pattern-list feature
    // once we have events coming out of the plugin.
    // PatternList(Vec<String>),
    PatternStart(NamedPattern),
    PatternStop(String),
    PatternStopAll,
    PatternClear(String),
    PatternClearAll,
}

pub struct Controller {
    pub commands_tx: Mutex<Producer<Command>>,
}

pub fn create_router(commands: Arc<Controller>) -> Router {
    return Router::new()
        .route("/start/:pattern_name", post(handler_start_pattern))
        .route("/stop/:pattern_name", post(handler_stop_pattern))
        .route("/stopall", post(handler_stopall))
        .route("/clear/:pattern_name", post(handler_clear_pattern))
        .route("/clearall", post(handler_clearall))
        .with_state(commands);
}

#[axum::debug_handler]
pub async fn handler_start_pattern(
    State(controller): State<Arc<Controller>>,
    Path(pattern_name): Path<String>,
    Json(pattern): Json<Pattern>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands_tx.lock().unwrap();
    // TODO: handle when the queue is full

    let named_pattern = NamedPattern {
        channel: pattern.channel,
        name: pattern_name,
        events: pattern.events,
        length_bars: pattern.length_bars,
    };
    match cmds.push(Command::PatternStart(named_pattern)) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
pub async fn handler_stop_pattern(
    State(controller): State<Arc<Controller>>,
    Path(pattern_name): Path<String>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands_tx.lock().unwrap();
    // TODO: handle when the queue is full
    match cmds.push(Command::PatternStop(pattern_name)) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
pub async fn handler_stopall(
    State(controller): State<Arc<Controller>>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands_tx.lock().unwrap();
    // TODO: handle when the queue is full
    match cmds.push(Command::PatternStopAll) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
pub async fn handler_clear_pattern(
    State(controller): State<Arc<Controller>>,
    Path(pattern_name): Path<String>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands_tx.lock().unwrap();
    // TODO: handle when the queue is full
    match cmds.push(Command::PatternClear(pattern_name)) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
pub async fn handler_clearall(
    State(controller): State<Arc<Controller>>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands_tx.lock().unwrap();
    // TODO: handle when the queue is full
    match cmds.push(Command::PatternClearAll) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::*;
    use crate::dur::Dur;
    use crate::pattern::{Event, EventType, NamedPattern, Note};
    use axum_test::TestServer;
    use rtrb::RingBuffer;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_pattern_start_endpoint_missing_channel() {
        let (commands_tx, mut commands_rx) = RingBuffer::<Command>::new(256); // Arbitrary buffer size
        let controller = Arc::new(Controller {
            commands_tx: Mutex::new(commands_tx),
        });
        let router = create_router(controller);
        let server = TestServer::new(router).unwrap();
        let response = server
            .post("/start/foo")
            // Not sure why the whitespace gets messed up by emacs rust-mode for this json structure
            .json(&json!({
                    "events": [
                        {
                            "action": {
                                "NoteEvent": {
                                    "note_num": 60,
                                    "velocity": 0.8,
                                    "dur": {"num": 1, "den": 2},
                                },
                            },
                            "dur": {"num": 1, "den": 1},
                        },
                    ],
                    "length_bars": {"num": 1, "den": 1},
            "channel": 1,
                }))
            .await;

        response.assert_status_ok();

        let received_val = commands_rx.pop().unwrap();

        assert_eq!(
            received_val,
            Command::PatternStart(NamedPattern {
                channel: 1 as u8,
                events: vec![Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60 as u8,
                        dur: Dur { num: 1, den: 2 },
                        velocity: 0.8,
                    }),
                    dur: Dur { num: 1, den: 1 },
                },],
                length_bars: Dur { num: 1, den: 1 },
                name: String::from("foo"),
            })
        );
    }

    #[tokio::test]
    async fn test_pattern_start_endpoint_channel_provided() {
        let (commands_tx, mut commands_rx) = RingBuffer::<Command>::new(256); // Arbitrary buffer size
        let controller = Arc::new(Controller {
            commands_tx: Mutex::new(commands_tx),
        });
        let router = create_router(controller);
        let server = TestServer::new(router).unwrap();
        let response = server
            .post("/start/foo")
            .json(&json!({
                "events": [
                     {
                         "action": {
                             "NoteEvent": {
                                 "note_num": 60,
                                 "velocity": 0.8,
                                 "dur": {"num": 1, "den": 2},
                             },
                         },
                         "dur": {"num": 1, "den": 1},
                     },
                ],
                "length_bars": {"num": 1, "den": 1},
                "channel": 2,
            }))
            .await;

        response.assert_status_ok();

        let received_val = commands_rx.pop().unwrap();

        assert_eq!(
            received_val,
            Command::PatternStart(NamedPattern {
                channel: 2 as u8,
                events: vec![Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60 as u8,
                        dur: Dur { num: 1, den: 2 },
                        velocity: 0.8,
                    }),
                    dur: Dur { num: 1, den: 1 },
                },],
                length_bars: Dur { num: 1, den: 1 },
                name: String::from("foo"),
            })
        );
    }
}
