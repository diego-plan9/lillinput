//! Action for interacting with `i3`.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::actions::{Action, ActionTypes};
use i3ipc::I3Connection;
use log::warn;

/// Action that executes `i3` commands.
pub struct I3Action {
    connection: Rc<RefCell<I3Connection>>,
    command: String,
}

/// Extended trait for construction new actions.
pub trait I3ActionExt {
    fn new(command: String, connection: Rc<RefCell<I3Connection>>) -> Self;
}

impl Action for I3Action {
    fn execute_command(&mut self) {
        // Perform the command, if specified.
        match Rc::clone(&self.connection)
            .borrow_mut()
            .run_command(&self.command)
        {
            Err(error) => warn!("i3: command invocation resulted in error: {}", error),
            Ok(command_reply) => {
                for outcome in command_reply.outcomes.iter().filter(|x| !x.success) {
                    warn!(
                        "i3: command execution resulted in error: {:?}",
                        outcome.error
                    );
                }
            }
        }
    }

    fn fmt_command(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:<{}>", ActionTypes::I3, self.command)
    }
}

impl I3ActionExt for I3Action {
    fn new(command: String, connection: Rc<RefCell<I3Connection>>) -> Self {
        I3Action {
            connection,
            command,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::env;
    use std::sync::{Arc, Mutex};

    use crate::actions::{ActionController, ActionEvents, ActionMap, Settings};
    use crate::test_utils::{default_test_settings, init_listener};

    #[test]
    /// Test the triggering of commands for the 4x2 swipe actions.
    fn test_i3_swipe_actions() {
        // Initialize the command line options.
        let mut settings: Settings = default_test_settings();
        settings.enabled_action_types = vec!["i3".to_string()];
        settings.actions = HashMap::from([
            (
                ActionEvents::ThreeFingerSwipeLeft.to_string(),
                vec!["i3:swipe left 3".to_string()],
            ),
            (
                ActionEvents::ThreeFingerSwipeRight.to_string(),
                vec!["i3:swipe right 3".to_string()],
            ),
            (
                ActionEvents::ThreeFingerSwipeUp.to_string(),
                vec!["i3:swipe up 3".to_string()],
            ),
            (
                ActionEvents::ThreeFingerSwipeDown.to_string(),
                vec!["i3:swipe down 3".to_string()],
            ),
            (
                ActionEvents::FourFingerSwipeLeft.to_string(),
                vec!["i3:swipe left 4".to_string()],
            ),
            (
                ActionEvents::FourFingerSwipeRight.to_string(),
                vec!["i3:swipe right 4".to_string()],
            ),
            (
                ActionEvents::FourFingerSwipeUp.to_string(),
                vec!["i3:swipe up 4".to_string()],
            ),
            (
                ActionEvents::FourFingerSwipeDown.to_string(),
                vec!["i3:swipe down 4".to_string()],
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
        init_listener(Arc::clone(&message_log));

        // Trigger swipe in the 4 directions.
        let mut action_map: ActionMap = ActionController::new(&settings);
        action_map.populate_actions(&settings);
        action_map.receive_end_event(&10.0, &0.0, 3);
        action_map.receive_end_event(&-10.0, &0.0, 3);
        action_map.receive_end_event(&0.0, &10.0, 3);
        action_map.receive_end_event(&0.0, &-10.0, 3);
        action_map.receive_end_event(&10.0, &0.0, 4);
        action_map.receive_end_event(&-10.0, &0.0, 4);
        action_map.receive_end_event(&0.0, &10.0, 4);
        action_map.receive_end_event(&0.0, &-10.0, 4);

        // Assert over the expected messages.
        let messages = message_log.lock().unwrap();
        assert_eq!(messages.len(), 8);
        for (message, expected_command) in messages.iter().zip(expected_commands.iter()) {
            assert_eq!(message, expected_command);
        }
    }

    #[test]
    ///Test graceful handling of unavailable i3 connection.
    fn test_i3_not_available() {
        // Initialize the command line options.
        let mut settings: Settings = default_test_settings();
        settings.enabled_action_types = vec!["i3".to_string()];
        settings.actions.insert(
            ActionEvents::ThreeFingerSwipeRight.to_string(),
            vec![
                "i3:swipe right".to_string(),
                "command:touch /tmp/swipe-right".to_string(),
            ],
        );

        // Create the action map.
        env::set_var("I3SOCK", "/tmp/non-existing-socket");
        let mut action_map: ActionMap = ActionController::new(&settings);
        action_map.populate_actions(&settings);

        // Assert that only the command action is created.
        assert_eq!(
            action_map
                .actions
                .get(&ActionEvents::ThreeFingerSwipeRight)
                .unwrap()
                .len(),
            1
        );
    }
}
