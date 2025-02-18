use crate::pattern::{NamedPattern, Pattern};
use axum::{extract::Path, extract::State, http::StatusCode, response, Json};
use rtrb::{Consumer, Producer};
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use crate::controller::*;
    use crate::dur::Dur;
    use crate::pattern::{Event, EventType, NamedPattern, Note};
    use crate::plugin::create_router;
    use axum_test::TestServer;
    use rtrb::RingBuffer;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_pattern_start_endpoint_missing_channel() {
        let (commands_tx, mut commands_rx) = RingBuffer::<Command>::new(256); // Arbitrary buffer size
        let (_, responses_rx) = RingBuffer::<Command>::new(256); // Arbitrary buffer size
        let commands = Arc::new(Controller {
            commands: Mutex::new(commands_tx),
            responses: Mutex::new(responses_rx),
        });
        let router = create_router(commands);
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
                length_bars: Some(Dur { num: 1, den: 1 }),
                name: String::from("foo"),
            })
        );
    }

    #[tokio::test]
    async fn test_pattern_start_endpoint_channel_provided() {
        let (commands_tx, mut commands_rx) = RingBuffer::<Command>::new(256); // Arbitrary buffer size
        let (_, responses_rx) = RingBuffer::<Command>::new(256); // Arbitrary buffer size
        let commands = Arc::new(Controller {
            commands: Mutex::new(commands_tx),
            responses: Mutex::new(responses_rx),
        });
        let router = create_router(commands);
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
                length_bars: Some(Dur { num: 1, den: 1 }),
                name: String::from("foo"),
            })
        );
    }
}
