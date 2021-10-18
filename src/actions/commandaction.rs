//! Action for executing commands.

use super::{Action, ActionExt, ActionTypes};
use log::warn;
use shlex::split;

use std::fmt;
use std::process::Command;

/// Action that executes shell commands.
pub struct CommandAction {
    command: String,
}

impl Action for CommandAction {
    fn execute_command(&mut self) {
        // Perform the command, if specified.
        let split_commands = split(&self.command).unwrap();
        match Command::new(&split_commands[0])
            .args(&split_commands[1..])
            .output()
        {
            Ok(_) => (),
            Err(e) => warn!("command: command execution resulted in error: {:?}", e),
        }
    }

    fn fmt_command(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:<{}>", ActionTypes::Command, self.command)
    }
}

impl ActionExt for CommandAction {
    fn new(command: String) -> CommandAction {
        CommandAction { command }
    }
}

#[cfg(test)]
mod test {
    use crate::actions::{ActionController, ActionMap, Settings};
    use crate::test_utils::default_test_settings;
    use std::path::Path;

    #[test]
    /// Test the triggering of commands for a single swipe action.
    fn test_command_single_action() {
        // File that will be touched .
        let expected_file = "/tmp/swipe-right";
        std::fs::remove_file(expected_file).ok();

        // Initialize the command line options.
        let mut settings: Settings = default_test_settings();
        settings.enabled_action_types = vec!["command".to_string()];
        settings.swipe_right_3 = vec!["command:touch /tmp/swipe-right".to_string()];

        // Trigger a swipe.
        let mut action_map: ActionMap = ActionController::new(&settings);
        action_map.populate_actions(&settings);
        action_map.receive_end_event(&10.0, &0.0, 3);

        // Assert.
        assert!(Path::new(expected_file).exists());
        std::fs::remove_file(expected_file).ok();
    }
}
