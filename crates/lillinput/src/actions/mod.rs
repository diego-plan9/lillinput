//! Traits for actions.
//!
//! Provides the interface for defining `Action`s that handle the different
//! `ActionEvents`.

pub mod commandaction;
pub mod controller;
pub mod errors;
pub mod i3action;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::actions::errors::{ActionControllerError, ActionError};
use crate::events::ActionEvent;
use i3ipc::I3Connection;
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
    threshold: f64,
    /// Optional `i3` RPC connection.
    connection: Option<Rc<RefCell<I3Connection>>>,
    /// Map between events and actions.
    actions: HashMap<ActionEvent, Vec<Box<dyn Action>>>,
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
    fn execute_command(&mut self) -> Result<(), ActionError>;
    /// Format the contents of the action as a String.
    fn fmt_command(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl fmt::Display for dyn Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Delegate on the structs specific `fmt` implementation.
        self.fmt_command(f)
    }
}
