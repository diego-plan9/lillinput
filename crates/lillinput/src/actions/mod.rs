//! Traits for actions.
//!
//! Provides the interface for defining `Action`s that handle the different
//! `ActionEvents`.

pub mod commandaction;
pub mod controller;
pub mod errors;
pub mod i3action;

use std::collections::HashMap;
use std::fmt;

use crate::actions::errors::{ActionControllerError, ActionError};
use crate::events::ActionEvent;
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

/// Map between events and actions.
pub struct ActionMap {
    /// Minimum threshold for displacement changes.
    pub threshold: f64,
    /// Map between events and actions.
    pub actions: HashMap<ActionEvent, Vec<Box<dyn Action>>>,
}

/// Controller that connects events and actions.
pub trait ActionController {
    /// Receive the end of swipe gesture event.
    ///
    /// # Arguments
    ///
    /// * `self` - action controller.
    /// * `dx` - the current position in the `x` axis.
    /// * `dy` - the current position in the `y` axis.
    /// * `finger_count` - the number of fingers used for the gesture.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the processing of the end of swipe event resulted in
    /// failure or in no [`Action`]s invoked.
    fn receive_end_event(
        &mut self,
        dx: f64,
        dy: f64,
        finger_count: i32,
    ) -> Result<(), ActionControllerError>;

    /// Parse a swipe gesture end event into an action event.
    ///
    /// # Arguments
    ///
    /// * `self` - action controller.
    /// * `dx` - the current position in the `x` axis.
    /// * `dy` - the current position in the `y` axis.
    /// * `finger_count` - the number of fingers used for the gesture.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the processing of the swipe event did not result in a
    /// [`ActionEvent`].
    fn end_event_to_action_event(
        &mut self,
        dx: f64,
        dy: f64,
        finger_count: i32,
    ) -> Result<ActionEvent, ActionControllerError>;
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