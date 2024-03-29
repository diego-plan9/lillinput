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
    #[must_use]
    pub fn new(command: String) -> CommandAction {
        CommandAction { command }
    }
}

impl Action for CommandAction {
    fn execute_command(&mut self) -> Result<(), ActionError> {
        // Perform the command, if specified.
        let split_commands = split(&self.command).ok_or(ActionError::ExecutionError {
            type_: "command".into(),
            message: format!("Unable to parse command: {}", self.command),
        })?;
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

    use super::CommandAction;
    use crate::actions::Action;
    use crate::controllers::{Controller, DefaultController};
    use crate::events::ActionEvent;
    use serial_test::serial;

    #[test]
    #[serial]
    /// Test the triggering of commands for a single swipe action.
    fn test_command_single_action() {
        // File that will be touched.
        let expected_file = "/tmp/swipe-right";
        std::fs::remove_file(expected_file).ok();

        // Create the controller.
        let actions_list: Vec<Box<dyn Action>> = vec![Box::new(CommandAction::new(
            "touch /tmp/swipe-right".into(),
        ))];
        let mut controller = DefaultController::default();
        controller
            .actions
            .insert(ActionEvent::ThreeFingerSwipeRight, actions_list);

        // Trigger a swipe.
        controller
            .process_action_event(ActionEvent::ThreeFingerSwipeRight)
            .ok();

        // Assert.
        assert!(Path::new(expected_file).exists());
        std::fs::remove_file(expected_file).ok();
    }
}
