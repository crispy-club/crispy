use crate::pattern::NamedPattern;
use nih_plug::prelude::nih_log;
use rtrb::{Consumer, Producer, RingBuffer};
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

impl Controller {
    pub fn new() -> (Arc<Controller>, Consumer<Command>) {
        // Arbitrary buffer size
        let (commands_tx, commands_rx) = RingBuffer::<Command>::new(256);
        let controller = Arc::new(Controller {
            commands_tx: Mutex::new(commands_tx),
        });
        (controller, commands_rx)
    }

    pub fn start(&self, np: NamedPattern) {
        let mut cmds = self.commands_tx.lock().unwrap();
        match cmds.push(Command::PatternStart(np.clone())) {
            Ok(_) => {
                // TODO: status line
                nih_log!("ran pattern {:?}", np.clone().name)
            }
            Err(err) => {
                // TODO: status line
                nih_log!("error running pattern {:?}: {:?}", np.clone().name, err)
            }
        }
    }

    pub fn stop(&self, np: NamedPattern) {
        let mut cmds = self.commands_tx.lock().unwrap();
        match cmds.push(Command::PatternStop(np.clone().name)) {
            Ok(_) => {
                // TODO: status line
                nih_log!("ran pattern {:?}", np.clone().name)
            }
            Err(err) => {
                // TODO: status line
                nih_log!("error running pattern {:?}: {:?}", np.clone().name, err)
            }
        }
    }

    pub fn stopall(&self) {
        let mut cmds = self.commands_tx.lock().unwrap();
        match cmds.push(Command::PatternStopAll) {
            Ok(_) => {
                // TODO: status line
                nih_log!("stopped all patterns");
            }
            Err(err) => {
                // TODO: status line
                nih_log!("error stopping all patterns: {:?}", err)
            }
        }
    }

    pub fn clear(&self, pattern_name: &str) {
        let mut cmds = self.commands_tx.lock().unwrap();
        match cmds.push(Command::PatternClear(pattern_name.to_string())) {
            Ok(_) => {
                // TODO: status line
                nih_log!("ran pattern {:?}", pattern_name)
            }
            Err(err) => {
                // TODO: status line
                nih_log!("error running pattern {:?}: {:?}", pattern_name, err)
            }
        }
    }

    pub fn clearall(&self) {
        let mut cmds = self.commands_tx.lock().unwrap();
        match cmds.push(Command::PatternClearAll) {
            Ok(_) => {
                // TODO: status line
                nih_log!("cleared all patterns");
            }
            Err(err) => {
                // TODO: status line
                nih_log!("error clearing all patterns: {:?}", err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::*;
    use crate::dur::Dur;
    use crate::pattern::NamedPattern;

    #[test]
    fn test_controller_stop() {
        let (controller, mut commands_rx) = Controller::new();
        controller.stop(NamedPattern {
            channel: 1,
            events: vec![],
            length_bars: Dur::new(1, 1),
            name: String::from("foo"),
        });
        assert_eq!(
            commands_rx.pop(),
            Ok(Command::PatternStop(String::from("foo")))
        );
    }

    #[test]
    fn test_controller_stopall() {
        let (controller, mut commands_rx) = Controller::new();
        controller.stopall();
        assert_eq!(commands_rx.pop(), Ok(Command::PatternStopAll));
    }

    #[test]
    fn test_controller_clear() {
        let (controller, mut commands_rx) = Controller::new();
        controller.clear("foo");
        assert_eq!(
            commands_rx.pop(),
            Ok(Command::PatternClear(String::from("foo")))
        );
    }

    #[test]
    fn test_controller_clearall() {
        let (controller, mut commands_rx) = Controller::new();
        controller.clearall();
        assert_eq!(commands_rx.pop(), Ok(Command::PatternClearAll));
    }
}
