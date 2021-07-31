//! Traits for actions.
//!
//! Provides the interface for defining `Action`s that handle the different
//! `ActionEvents`.

pub mod commandaction;
pub mod controller;
pub mod i3action;

use super::{ActionEvents, ActionTypes, Opts};
use i3ipc::I3Connection;

use std::cell::RefCell;
use std::rc::Rc;

/// Map between events and actions.
pub struct ActionMap {
    threshold: f64,
    connection: Option<Rc<RefCell<I3Connection>>>,
    swipe_left: Vec<Box<dyn Action>>,
    swipe_right: Vec<Box<dyn Action>>,
    swipe_up: Vec<Box<dyn Action>>,
    swipe_down: Vec<Box<dyn Action>>,
}

/// Controller that connects events and actions.
pub trait ActionController {
    fn new(opts: &Opts) -> Self;

    /// Create the individual actions used by this controller.
    ///
    /// Parse the command line arguments and add the individual actions to
    /// the internal structures in this controller.
    ///
    /// # Arguments
    ///
    /// * `self` - action controller.
    /// * `opts` - command line arguments.
    fn populate_actions(&mut self, opts: &Opts);

    /// Receive the end of swipe gesture event.
    ///
    /// # Arguments
    ///
    /// * `self` - action controller.
    /// * `dx` - the current position in the `x` axis
    /// * `dy` - the current position in the `y` axis
    fn receive_end_event(&mut self, dx: &f64, dy: &f64);
}

/// Handler for a single action triggered by an event.
pub trait Action {
    /// Execute the command for this action.
    fn execute_command(&mut self);
}

/// Extended trait for construction new actions.
pub trait ActionExt {
    /// Return a new action.
    fn new(command: String) -> Self;
}

#[cfg(test)]
mod test {
    use super::{ActionController, ActionMap, Opts};
    use crate::test_utils::init_listener;
    use clap::Clap;
    use std::env;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    #[test]
    /// Test the triggering of commands for a single swipe action.
    fn test_command_single_action() {
        // File that will be touched .
        let expected_file = "/tmp/swipe-right";
        std::fs::remove_file(expected_file).ok();

        // Initialize the command line options.
        let mut opts: Opts = Opts::parse();
        opts.enabled_action_types = vec!["command".to_string()];
        opts.swipe_right_3 = vec!["command:touch /tmp/swipe-right".to_string()];

        // Trigger a swipe.
        let mut action_map: ActionMap = ActionController::new(&opts);
        action_map.populate_actions(&opts);
        action_map.receive_end_event(&10.0, &0.0);

        // Assert.
        assert!(Path::new(expected_file).exists());
    }

    #[test]
    /// Test the triggering of commands for the 4 swipe actions.
    fn test_i3_swipe_actions() {
        // Initialize the command line options.
        let mut opts: Opts = Opts::parse();
        opts.enabled_action_types = vec!["i3".to_string()];
        opts.swipe_right_3 = vec!["i3:swipe right".to_string()];
        opts.swipe_left_3 = vec!["i3:swipe left".to_string()];
        opts.swipe_up_3 = vec!["i3:swipe up".to_string()];
        opts.swipe_down_3 = vec!["i3:swipe down".to_string()];

        // Create the expected commands (version + 4 swipes).
        let expected_commands = vec!["swipe right", "swipe left", "swipe up", "swipe down"];

        // Create the listener and the shared storage for the commands.
        let message_log = Arc::new(Mutex::new(vec![]));
        init_listener(Arc::clone(&message_log));

        // Trigger swipe in the 4 directions.
        let mut action_map: ActionMap = ActionController::new(&opts);
        action_map.populate_actions(&opts);
        action_map.receive_end_event(&10.0, &0.0);
        action_map.receive_end_event(&-10.0, &0.0);
        action_map.receive_end_event(&0.0, &10.0);
        action_map.receive_end_event(&0.0, &-10.0);

        // Assert over the expected messages.
        let messages = message_log.lock().unwrap();
        for (message, expected_command) in messages.iter().zip(expected_commands.iter()) {
            assert!(message == expected_command);
        }
    }

    #[test]
    /// Test the usage of the threshold argument.
    fn test_i3_swipe_below_threshold() {
        // Initialize the command line options.
        let mut opts: Opts = Opts::parse();
        opts.enabled_action_types = vec!["i3".to_string()];
        opts.swipe_right_3 = vec!["i3:swipe right".to_string()];
        opts.swipe_left_3 = vec!["i3:swipe left".to_string()];
        opts.threshold = 5.0;

        // Create the expected commands (version + 4 swipes).
        let expected_commands = vec!["swipe left"];

        // Create the listener and the shared storage for the commands.
        let message_log = Arc::new(Mutex::new(vec![]));
        init_listener(Arc::clone(&message_log));

        // Trigger right swipe below threshold, left above threshold.
        let mut action_map: ActionMap = ActionController::new(&opts);
        action_map.populate_actions(&opts);
        action_map.receive_end_event(&4.0, &0.0);
        action_map.receive_end_event(&-5.0, &0.0);

        // Assert over the expected messages.
        let messages = message_log.lock().unwrap();
        for (message, expected_command) in messages.iter().zip(expected_commands.iter()) {
            assert!(message == expected_command);
        }
    }

    #[test]
    ///Test graceful handling of unavailable i3 connection.
    fn test_i3_not_available() {
        // Initialize the command line options.
        let mut opts: Opts = Opts::parse();
        opts.enabled_action_types = vec!["i3".to_string(), "command".to_string()];
        opts.swipe_right_3 = vec![
            "i3:swipe right".to_string(),
            "command:touch /tmp/swipe-right".to_string(),
        ];

        // Create the action map.
        env::set_var("I3SOCK", "/tmp/non-existing-socket");
        let mut action_map: ActionMap = ActionController::new(&opts);
        action_map.populate_actions(&opts);

        // Assert that only the command action is created.
        assert!(action_map.swipe_right.len() == 1);
    }
}
