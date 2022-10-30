//! Action for interacting with `i3`.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::actions::errors::ActionError;
use crate::actions::{Action, ActionType};
use i3ipc::I3Connection;

/// Shared optional `i3` connection.
pub type SharedConnection = Rc<RefCell<Option<I3Connection>>>;

/// Action that executes `i3` commands.
#[derive(Debug)]
pub struct I3Action {
    /// `i3` RPC connection.
    connection: SharedConnection,
    /// `i3` command to be executed in this action.
    command: String,
}

impl I3Action {
    /// Create a new [`I3Action`].
    ///
    /// # Arguments
    ///
    /// * `command` - `i3` command to be executed in this action.
    /// * `connection` - `i3` RPC connection.
    #[must_use]
    pub fn new(command: String, connection: Rc<RefCell<Option<I3Connection>>>) -> Self {
        I3Action {
            connection,
            command,
        }
    }
}

impl Action for I3Action {
    fn execute_command(&mut self) -> Result<(), ActionError> {
        // Perform the command, if specified.
        let connection_rc = Rc::clone(&self.connection);
        let connection_option = &mut *connection_rc.borrow_mut();

        // Check if the i3 connection is valid.
        let connection = match connection_option {
            Some(connection) => connection,
            None => {
                return Err(ActionError::ExecutionError {
                    type_: "i3".into(),
                    message: "i3 connection is not set".into(),
                })
            }
        };

        match connection.run_command(&self.command) {
            Err(e) => Err(ActionError::ExecutionError {
                type_: "i3".into(),
                message: e.to_string(),
            }),
            Ok(command_reply) => {
                if command_reply.outcomes.iter().any(|x| !x.success) {
                    Err(ActionError::ExecutionError {
                        type_: "i3".into(),
                        message: "unsuccessful outcome(s)".into(),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }

    fn fmt_command(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:<{}>", ActionType::I3, self.command)
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    use super::I3Action;
    use crate::actions::errors::ActionError;
    use crate::actions::Action;
    use crate::controllers::defaultcontroller::DefaultController;
    use crate::controllers::Controller;
    use crate::events::defaultprocessor::DefaultProcessor;
    use crate::events::ActionEvent;
    use crate::test_utils::init_listener;

    use i3ipc::I3Connection;
    use serial_test::serial;

    #[test]
    #[serial]
    /// Test the triggering of commands for the 4x2 swipe actions.
    fn test_i3_swipe_actions() {
        // Create the expected commands (version + 4 swipes).
        let expected_commands = vec![
            "swipe right 3",
            "swipe left 3",
            "swipe up 3",
            "swipe down 3",
            "swipe right 4",
            "swipe left 4",
            "swipe up 4",
            "swipe down 4",
        ];

        // Create the listener and the shared storage for the commands.
        let message_log = Arc::new(Mutex::new(vec![]));
        let socket_file = init_listener(Arc::clone(&message_log));

        // Create the controller.
        let connection = Rc::new(RefCell::new(Some(I3Connection::connect().unwrap())));
        let actions_list: Vec<Box<dyn Action>> = vec![
            Box::new(I3Action::new(
                "swipe right 3".into(),
                Rc::clone(&connection),
            )),
            Box::new(I3Action::new("swipe left 3".into(), Rc::clone(&connection))),
            Box::new(I3Action::new("swipe up 3".into(), Rc::clone(&connection))),
            Box::new(I3Action::new("swipe down 3".into(), Rc::clone(&connection))),
            Box::new(I3Action::new(
                "swipe right 4".into(),
                Rc::clone(&connection),
            )),
            Box::new(I3Action::new("swipe left 4".into(), Rc::clone(&connection))),
            Box::new(I3Action::new("swipe up 4".into(), Rc::clone(&connection))),
            Box::new(I3Action::new("swipe down 4".into(), Rc::clone(&connection))),
        ];
        let processor = DefaultProcessor::new(5.0, "seat0").unwrap();
        let mut controller = DefaultController::new(
            Box::new(processor),
            HashMap::from([(ActionEvent::ThreeFingerSwipeRight, actions_list)]),
        );

        // Trigger swipe in the 4 directions.
        controller
            .receive_end_event(ActionEvent::ThreeFingerSwipeRight)
            .ok();
        controller
            .receive_end_event(ActionEvent::ThreeFingerSwipeLeft)
            .ok();
        controller
            .receive_end_event(ActionEvent::ThreeFingerSwipeUp)
            .ok();
        controller
            .receive_end_event(ActionEvent::ThreeFingerSwipeDown)
            .ok();
        controller
            .receive_end_event(ActionEvent::FourFingerSwipeRight)
            .ok();
        controller
            .receive_end_event(ActionEvent::FourFingerSwipeLeft)
            .ok();
        controller
            .receive_end_event(ActionEvent::FourFingerSwipeUp)
            .ok();
        controller
            .receive_end_event(ActionEvent::FourFingerSwipeDown)
            .ok();
        std::fs::remove_file(socket_file.path().file_name().unwrap()).ok();

        // Assert over the expected messages.
        let messages = message_log.lock().unwrap();
        assert_eq!(messages.len(), 8);
        for (message, expected_command) in messages.iter().zip(expected_commands.iter()) {
            assert_eq!(message, expected_command);
        }
    }

    #[test]
    #[serial]
    ///Test graceful handling of unavailable i3 connection.
    fn test_i3_not_available() {
        // Create the action.
        let mut action = I3Action::new(String::from("swipe right 3"), Rc::new(RefCell::new(None)));

        // Trigger a swipe.
        let result = action.execute_command();

        // Assert the command is not executed.
        assert_eq!(
            result,
            Err(ActionError::ExecutionError {
                type_: String::from("i3"),
                message: String::from("i3 connection is not set"),
            })
        );
    }
}
