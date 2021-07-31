//! Controller for actions.

use super::commandaction::CommandAction;
use super::i3action::{I3Action, I3ActionExt};
use super::{Action, ActionController, ActionEvents, ActionExt, ActionMap, ActionTypes, Opts};

use i3ipc::I3Connection;
use log::{info, warn};

use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

impl ActionController for ActionMap {
    fn new(opts: &Opts) -> Self {
        // Create the I3 connection if needed.
        let connection = match opts
            .enabled_action_types
            .contains(&ActionTypes::I3.to_string())
        {
            true => match I3Connection::connect() {
                Ok(mut conn) => {
                    info!(
                        "i3: connection opened (with({:?})",
                        conn.get_version().unwrap().human_readable
                    );
                    Some(Rc::new(RefCell::new(conn)))
                }
                Err(error) => {
                    info!("i3: could not establish a connection: {:?}", error);
                    None
                }
            },
            false => None,
        };

        ActionMap {
            threshold: opts.threshold,
            connection,
            swipe_left: vec![],
            swipe_right: vec![],
            swipe_up: vec![],
            swipe_down: vec![],
        }
    }

    fn populate_actions(&mut self, opts: &Opts) {
        /// Add actions to a destination vector.
        ///
        /// # Arguments
        ///
        /// * `arguments` - list of command line arguments
        /// * `destination` - vector to append actions to
        /// * `connection` - optional i3 connection
        fn parse_action_list(
            arguments: &[String],
            destination: &mut Vec<Box<dyn Action>>,
            connection: &Option<Rc<RefCell<I3Connection>>>,
        ) {
            for value in arguments.iter() {
                // Split the arguments, in the form "{type}:{value}".
                let mut splitter = value.splitn(2, ':');
                let action_type = splitter.next().unwrap();
                let action_value = splitter.next().unwrap();

                // Create new actions and add them to the controller.
                match ActionTypes::from_str(action_type) {
                    Ok(ActionTypes::Command) => {
                        destination.push(Box::new(CommandAction::new(action_value.to_string())));
                    }
                    Ok(ActionTypes::I3) => match connection {
                        Some(conn) => {
                            destination.push(Box::new(I3Action::new(
                                action_value.to_string(),
                                Rc::clone(conn),
                            )));
                        }
                        None => {
                            warn!("ignoring i3 action, as the i3 connection could not be set.")
                        }
                    },
                    Err(_) => {}
                }
            }
        }

        parse_action_list(&opts.swipe_left_3, &mut self.swipe_left, &self.connection);
        parse_action_list(&opts.swipe_right_3, &mut self.swipe_right, &self.connection);
        parse_action_list(&opts.swipe_up_3, &mut self.swipe_up, &self.connection);
        parse_action_list(&opts.swipe_down_3, &mut self.swipe_down, &self.connection);

        // Print information.
        info!(
            "Action controller started: {:?}/{:?}/{:?}/{:?} actions enabled",
            self.swipe_left.len(),
            self.swipe_right.len(),
            self.swipe_up.len(),
            self.swipe_down.len(),
        );
    }

    #[allow(clippy::collapsible_else_if)]
    fn receive_end_event(&mut self, dx: &f64, dy: &f64) {
        // Avoid acting if the displacement is below the threshold.
        if dx.abs() < self.threshold && dy.abs() < self.threshold {
            return;
        }

        // Determine the command for the event.
        let command: ActionEvents;
        if dx.abs() > dy.abs() {
            if dx > &0.0 {
                command = ActionEvents::ThreeFingerSwipeRight
            } else {
                command = ActionEvents::ThreeFingerSwipeLeft
            }
        } else {
            if dy > &0.0 {
                command = ActionEvents::ThreeFingerSwipeUp
            } else {
                command = ActionEvents::ThreeFingerSwipeDown
            }
        }

        // Invoke actions.
        match command {
            ActionEvents::ThreeFingerSwipeLeft => {
                for action in self.swipe_left.iter_mut() {
                    action.execute_command();
                }
            }
            ActionEvents::ThreeFingerSwipeRight => {
                for action in self.swipe_right.iter_mut() {
                    action.execute_command();
                }
            }
            ActionEvents::ThreeFingerSwipeUp => {
                for action in self.swipe_up.iter_mut() {
                    action.execute_command();
                }
            }
            ActionEvents::ThreeFingerSwipeDown => {
                for action in self.swipe_down.iter_mut() {
                    action.execute_command();
                }
            }
        }
    }
}
