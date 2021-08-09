//! Action for executing commands.

use super::{Action, ActionExt, ActionTypes};
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
        // TODO: capture result gracefully.
        Command::new(&split_commands[0])
            .args(&split_commands[1..])
            .output()
            .expect("Failed to execute command");
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
