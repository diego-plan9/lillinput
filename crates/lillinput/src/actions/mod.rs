//! Building blocks for actions.

pub mod commandaction;
pub mod errors;
pub mod i3action;

use std::fmt;

use crate::actions::errors::ActionError;
use strum::{Display, EnumString, EnumVariantNames};

/// Possible choices for action types.
#[derive(Display, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
pub enum ActionType {
    /// Action for interacting with `i3`.
    I3,
    /// Action for executing commands.
    Command,
}

/// Handler for a single action triggered by an event.
pub trait Action: std::fmt::Debug {
    /// Execute the command for this action.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the execution of the command was not successful.
    fn execute_command(&mut self) -> Result<(), ActionError>;
    /// Format the contents of the action as a [`String`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if the action cannot be formatted as a [`String`].
    fn fmt_command(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl fmt::Display for dyn Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Delegate on the structs specific `fmt` implementation.
        self.fmt_command(f)
    }
}
