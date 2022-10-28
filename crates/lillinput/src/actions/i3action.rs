//! Action for interacting with `i3`.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::actions::errors::ActionError;
use crate::actions::{Action, ActionType};
use i3ipc::I3Connection;

/// Action that executes `i3` commands.
#[derive(Debug)]
pub struct I3Action {
    /// `i3` RPC connection.
    connection: Rc<RefCell<Option<I3Connection>>>,
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
    use std::collections::HashMap;
    use std::env;
    use std::sync::{Arc, Mutex};

    use crate::actions::{ActionController, ActionEvent, ActionMap};
    use crate::opts::StringifiedAction;
    use crate::settings::Settings;
    use crate::test_utils::{default_test_settings, init_listener};

    use serial_test::serial;

    #[test]
    #[serial]
    /// Test the triggering of commands for the 4x2 swipe actions.
    fn test_i3_swipe_actions() {
        // Initialize the command line options.
        let mut settings: Settings = default_test_settings();
        settings.enabled_action_types = vec!["i3".to_string()];
        settings.actions = HashMap::from([
            (
                ActionEvent::ThreeFingerSwipeLeft.to_string(),
                vec![StringifiedAction::new("i3", "swipe left 3")],
            ),
            (
                ActionEvent::ThreeFingerSwipeRight.to_string(),
                vec![StringifiedAction::new("i3", "swipe right 3")],
            ),
            (
                ActionEvent::ThreeFingerSwipeUp.to_string(),
                vec![StringifiedAction::new("i3", "swipe up 3")],
            ),
            (
                ActionEvent::ThreeFingerSwipeDown.to_string(),
                vec![StringifiedAction::new("i3", "swipe down 3")],
            ),
            (
                ActionEvent::FourFingerSwipeLeft.to_string(),
                vec![StringifiedAction::new("i3", "swipe left 4")],
            ),
            (
                ActionEvent::FourFingerSwipeRight.to_string(),
                vec![StringifiedAction::new("i3", "swipe right 4")],
            ),
            (
                ActionEvent::FourFingerSwipeUp.to_string(),
                vec![StringifiedAction::new("i3", "swipe up 4")],
            ),
            (
                ActionEvent::FourFingerSwipeDown.to_string(),
                vec![StringifiedAction::new("i3", "swipe down 4")],
            ),
        ]);

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

        // Trigger swipe in the 4 directions.
        let mut action_map: ActionMap = ActionMap::new(&settings);
        action_map.populate_actions(&settings);
        action_map.receive_end_event(10.0, 0.0, 3).ok();
        action_map.receive_end_event(-10.0, 0.0, 3).ok();
        action_map.receive_end_event(0.0, 10.0, 3).ok();
        action_map.receive_end_event(0.0, -10.0, 3).ok();
        action_map.receive_end_event(10.0, 0.0, 4).ok();
        action_map.receive_end_event(-10.0, 0.0, 4).ok();
        action_map.receive_end_event(0.0, 10.0, 4).ok();
        action_map.receive_end_event(0.0, -10.0, 4).ok();
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
        // Initialize the command line options.
        let mut settings: Settings = default_test_settings();
        settings.enabled_action_types = vec!["i3".to_string()];
        settings.actions.insert(
            ActionEvent::ThreeFingerSwipeRight.to_string(),
            vec![
                StringifiedAction::new("i3", "swipe right"),
                StringifiedAction::new("command", "touch /tmp/swipe-right"),
            ],
        );

        // Create the action map.
        env::set_var("I3SOCK", "/tmp/non-existing-socket");
        let mut action_map: ActionMap = ActionMap::new(&settings);
        action_map.populate_actions(&settings);

        // Assert that only the command action is created.
        assert_eq!(
            action_map
                .actions
                .get(&ActionEvent::ThreeFingerSwipeRight)
                .unwrap()
                .len(),
            1
        );
    }
}
