//! Traits for actions.
//!
//! Provides the interface for defining `Action`s that handle the different
//! events.

pub mod commandaction;
pub mod controller;
pub mod i3action;

use super::{ActionChoices, ActionEvents, Opts};
use i3ipc::I3Connection;

use std::cell::RefCell;
use std::rc::Rc;

/// Maps between events and actions.
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

/// Action handler for events.
pub trait Action {
    /// Execute the command for this action.
    fn execute_command(&mut self);
}

/// Extended trait for action handler for events.
pub trait ActionExt {
    /// Return a new action.
    fn new(command: String) -> Self;
}
