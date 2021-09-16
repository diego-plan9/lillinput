//! Controller for actions.

use super::commandaction::CommandAction;
use super::i3action::{I3Action, I3ActionExt};
use super::{Action, ActionController, ActionEvents, ActionExt, ActionMap, ActionTypes, Opts};

use i3ipc::I3Connection;
use itertools::Itertools;
use log::{debug, info, warn};

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
            swipe_left_3: vec![],
            swipe_right_3: vec![],
            swipe_up_3: vec![],
            swipe_down_3: vec![],
            swipe_left_4: vec![],
            swipe_right_4: vec![],
            swipe_up_4: vec![],
            swipe_down_4: vec![],
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

        parse_action_list(&opts.swipe_left_3, &mut self.swipe_left_3, &self.connection);
        parse_action_list(
            &opts.swipe_right_3,
            &mut self.swipe_right_3,
            &self.connection,
        );
        parse_action_list(&opts.swipe_up_3, &mut self.swipe_up_3, &self.connection);
        parse_action_list(&opts.swipe_down_3, &mut self.swipe_down_3, &self.connection);
        parse_action_list(&opts.swipe_left_4, &mut self.swipe_left_4, &self.connection);
        parse_action_list(
            &opts.swipe_right_4,
            &mut self.swipe_right_4,
            &self.connection,
        );
        parse_action_list(&opts.swipe_up_4, &mut self.swipe_up_4, &self.connection);
        parse_action_list(&opts.swipe_down_4, &mut self.swipe_down_4, &self.connection);

        // Print information.
        info!(
            "Action controller started: {:?}/{:?}/{:?}/{:?} actions enabled",
            self.swipe_left_3.len(),
            self.swipe_right_3.len(),
            self.swipe_up_3.len(),
            self.swipe_down_3.len(),
        );

        // Print detailed information about actions.
        debug!(
            " * {}: {}",
            ActionEvents::ThreeFingerSwipeLeft,
            self.swipe_left_3.iter().format(", ")
        );
        debug!(
            " * {}: {}",
            ActionEvents::ThreeFingerSwipeRight,
            self.swipe_right_3.iter().format(", ")
        );
        debug!(
            " * {}: {}",
            ActionEvents::ThreeFingerSwipeUp,
            self.swipe_up_3.iter().format(", ")
        );
        debug!(
            " * {}: {}",
            ActionEvents::ThreeFingerSwipeDown,
            self.swipe_down_3.iter().format(", ")
        );
    }

    #[allow(clippy::collapsible_else_if)]
    fn end_event_to_action_event(&mut self, dx: &f64, dy: &f64, finger_count: i32) -> Option<ActionEvents> {
        // Avoid acting if the displacement is below the threshold.
        if dx.abs() < self.threshold && dy.abs() < self.threshold {
            debug!("Received end event below threshold, discarding");
            return None
        }
        // Avoid acting if the number of fingers is not supported.
        if finger_count != 3 && finger_count != 4 {
            debug!("Received end event with unsupported finger count, discarding");
            return None
        }

        // Determine the command for the event.
        let command: ActionEvents;
        if dx.abs() > dy.abs() {
            if dx > &0.0 {
                if finger_count == 3 {
                    command = ActionEvents::ThreeFingerSwipeRight
                } else {
                    command = ActionEvents::FourFingerSwipeRight
                }
            } else {
                if finger_count == 3 {
                    command = ActionEvents::ThreeFingerSwipeLeft
                } else {
                    command = ActionEvents::FourFingerSwipeLeft
                }
            }
        } else {
            if dy > &0.0 {
                if finger_count == 3 {
                    command = ActionEvents::ThreeFingerSwipeUp
                } else {
                    command = ActionEvents::FourFingerSwipeUp
                }
            } else {
                if finger_count == 3 {
                    command = ActionEvents::ThreeFingerSwipeDown
                } else {
                    command = ActionEvents::FourFingerSwipeDown
                }
            }
        }

        return Some(command)
    }


    fn receive_end_event(&mut self, dx: &f64, dy: &f64, finger_count: i32) {
        let command = self.end_event_to_action_event(dx, dy, finger_count);

        // Invoke actions.
        let actions = match command {
            Some(ActionEvents::ThreeFingerSwipeLeft) => &mut self.swipe_left_3,
            Some(ActionEvents::ThreeFingerSwipeRight) => &mut self.swipe_right_3,
            Some(ActionEvents::ThreeFingerSwipeUp) => &mut self.swipe_up_3,
            Some(ActionEvents::ThreeFingerSwipeDown) => &mut self.swipe_down_3,
            Some(ActionEvents::FourFingerSwipeLeft) => &mut self.swipe_left_4,
            Some(ActionEvents::FourFingerSwipeRight) => &mut self.swipe_right_4,
            Some(ActionEvents::FourFingerSwipeUp) => &mut self.swipe_up_4,
            Some(ActionEvents::FourFingerSwipeDown) => &mut self.swipe_down_4,
            None => return,
        };

        debug!(
            "Received end event: {}, triggering {} actions",
            command.unwrap(),
            actions.len()
        );

        for action in actions.iter_mut() {
            action.execute_command();
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ActionController, ActionEvents, ActionMap, Opts};
    use clap::Clap;

    #[test]
    /// Test the handling of an event `finger_count` parameter.
    fn test_parse_finger_count() {
        // Initialize the command line options.
        let mut opts: Opts = Opts::parse();
        opts.threshold = 5.0;
        let mut action_map: ActionMap = ActionController::new(&opts);

        // Trigger right swipe with supported (3) fingers count.
        let action_event = action_map.end_event_to_action_event(&5.0, &0.0, 3);
        assert_eq!(action_event.is_some(), true);
        assert_eq!(action_event.unwrap() == ActionEvents::ThreeFingerSwipeRight, true);

        // Trigger right swipe with supported (4) fingers count.
        let action_event = action_map.end_event_to_action_event(&5.0, &0.0, 4);
        assert_eq!(action_event.is_some(), true);
        assert_eq!(action_event.unwrap() == ActionEvents::FourFingerSwipeRight, true);

        // Trigger right swipe with unsupported (5) fingers count.
        let action_event = action_map.end_event_to_action_event(&5.0, &0.0, 5);
        assert_eq!(action_event.is_none(), true);
    }

}
