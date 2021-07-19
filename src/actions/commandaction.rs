//! Action for executing commands.

use super::{Action, ActionExt};
use shlex::split;
use std::process::Command;

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
}

impl ActionExt for CommandAction {
    fn new(command: String) -> CommandAction {
        CommandAction { command }
    }
}
