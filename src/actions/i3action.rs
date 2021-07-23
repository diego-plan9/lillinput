//! Action for interacting with `i3`.

use super::Action;
use i3ipc::I3Connection;

use std::cell::RefCell;
use std::rc::Rc;

/// Action that executes `i3` commands.
pub struct I3Action {
    connection: Rc<RefCell<I3Connection>>,
    command: String,
}

/// Extended trait for construction new actions.
pub trait I3ActionExt {
    fn new(command: String, connection: Rc<RefCell<I3Connection>>) -> Self;
}

impl Action for I3Action {
    fn execute_command(&mut self) {
        // Perform the command, if specified.
        // TODO: capture result.
        Rc::clone(&self.connection)
            .borrow_mut()
            .run_command(&self.command)
            .unwrap();
    }
}

impl I3ActionExt for I3Action {
    fn new(command: String, connection: Rc<RefCell<I3Connection>>) -> Self {
        I3Action {
            connection,
            command,
        }
    }
}
