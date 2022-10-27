//! Action for executing commands.

use std::fmt;
use std::process::Command;

use crate::actions::errors::ActionError;
use crate::actions::{Action, ActionType};
use shlex::split;

/// Action that executes shell commands.
#[derive(Debug)]
pub struct CommandAction {
    /// Command to be executed in this action.
    command: String,
}

impl CommandAction {
    /// Create a new [`CommandAction`].
    ///
    /// # Arguments
    ///
    /// * `command` - shell command to be executed in this action.
    pub fn new(command: String) -> CommandAction {
        CommandAction { command }
    }
}

impl Action for CommandAction {
    fn execute_command(&mut self) -> Result<(), ActionError> {
        // Perform the command, if specified.
        let split_commands = split(&self.command).unwrap();
        Command::new(&split_commands[0])
            .args(&split_commands[1..])
            .output()
            .map(|_| ())
            .map_err(|e| ActionError::ExecutionError {
                type_: "command".into(),
                message: e.to_string(),
            })
    }

    fn fmt_command(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:<{}>", ActionType::Command, self.command)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::actions::{ActionController, ActionEvent, ActionMap};
    use crate::opts::StringifiedAction;
    use crate::settings::Settings;
    use crate::test_utils::default_test_settings;

    #[test]
    /// Test the triggering of commands for a single swipe action.
    fn test_command_single_action() {
        // File that will be touched.
        let expected_file = "/tmp/swipe-right";
        std::fs::remove_file(expected_file).ok();

        // Initialize the command line options.
        let mut settings: Settings = default_test_settings();
        settings.enabled_action_types = vec!["command".to_string()];
        settings.actions.insert(
            ActionEvent::ThreeFingerSwipeRight.to_string(),
            vec![StringifiedAction::new("command", "touch /tmp/swipe-right")],
        );

        // Trigger a swipe.
        let mut action_map: ActionMap = ActionMap::new(&settings);
        action_map.populate_actions(&settings);
        action_map.receive_end_event(10.0, 0.0, 3).ok();

        // Assert.
        assert!(Path::new(expected_file).exists());
        std::fs::remove_file(expected_file).ok();
    }
}
