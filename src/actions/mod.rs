//! Traits for actions.
//!
//! Provides the interface for defining `Action`s that handle the different
//! `ActionEvents`.

pub mod commandaction;
pub mod controller;
pub mod i3action;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::{ActionEvents, ActionTypes, Settings};
use i3ipc::I3Connection;

/// Map between events and actions.
pub struct ActionMap {
    /// Minimum threshold for displacement changes.
    threshold: f64,
    /// Optional `i3` RPC connection.
    connection: Option<Rc<RefCell<I3Connection>>>,
    /// Map between events and actions.
    actions: HashMap<ActionEvents, Vec<Box<dyn Action>>>,
}

/// Controller that connects events and actions.
pub trait ActionController {
    /// Create a new [`ActionController`].
    ///
    /// # Arguments
    ///
    /// * `settings` - application settings.
    fn new(settings: &Settings) -> Self;

    /// Create the individual actions used by this controller.
    ///
    /// Parse the command line arguments and add the individual actions to
    /// the internal structures in this controller.
    ///
    /// # Arguments
    ///
    /// * `self` - action controller.
    /// * `settings` - application settings.
    fn populate_actions(&mut self, settings: &Settings);

    /// Receive the end of swipe gesture event.
    ///
    /// # Arguments
    ///
    /// * `self` - action controller.
    /// * `dx` - the current position in the `x` axis.
    /// * `dy` - the current position in the `y` axis.
    /// * `finger_count` - the number of fingers used for the gesture.
    fn receive_end_event(&mut self, dx: f64, dy: f64, finger_count: i32);

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
    ) -> Option<ActionEvents>;
}

/// Handler for a single action triggered by an event.
pub trait Action: std::fmt::Debug {
    /// Execute the command for this action.
    fn execute_command(&mut self);
    /// Format the contents of the action as a String.
    fn fmt_command(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

/// Extended trait for construction new actions.
pub trait ActionExt {
    /// Create a new [`ActionExt`].
    ///
    /// # Arguments
    ///
    /// * `command` - the command to be executed in this action.
    fn new(command: String) -> Self;
}

impl fmt::Display for dyn Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Delegate on the structs specific `fmt` implementation.
        self.fmt_command(f)
    }
}
