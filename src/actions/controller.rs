//! Controller for actions.

use crate::Settings;
use super::commandaction::CommandAction;
use super::i3action::{I3Action, I3ActionExt};
use super::{Action, ActionController, ActionEvents, ActionExt, ActionMap, ActionTypes};

use i3ipc::I3Connection;
use itertools::Itertools;
use log::{debug, info, warn};

use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;
use std::str::FromStr;

use strum::IntoEnumIterator;

/// Possible choices for finger count.
enum FingerCount {
    ThreeFinger = 3,
    FourFinger = 4,
}

impl TryFrom<i32> for FingerCount {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(FingerCount::ThreeFinger),
            4 => Ok(FingerCount::FourFinger),
            _ => Err(()),
        }
    }
}

// Axis of a swipe action.
enum Axis {
    X,
    Y,
}

impl ActionController for ActionMap {
    fn new(settings: &Settings) -> Self {
        // Create the I3 connection if needed.
        let connection = match settings
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
            threshold: settings.threshold,
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

    fn populate_actions(&mut self, settings: &Settings) {
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

        // Populate the fields for each `ActionEvent`, printing debug info in the process.
        for action_event in ActionEvents::iter() {
            let (settings_field, self_field) = match action_event {
                ActionEvents::ThreeFingerSwipeLeft => (&settings.swipe_left_3, &mut self.swipe_left_3),
                ActionEvents::ThreeFingerSwipeRight => {
                    (&settings.swipe_right_3, &mut self.swipe_right_3)
                }
                ActionEvents::ThreeFingerSwipeUp => (&settings.swipe_up_3, &mut self.swipe_up_3),
                ActionEvents::ThreeFingerSwipeDown => (&settings.swipe_down_3, &mut self.swipe_down_3),
                ActionEvents::FourFingerSwipeLeft => (&settings.swipe_left_4, &mut self.swipe_left_4),
                ActionEvents::FourFingerSwipeRight => {
                    (&settings.swipe_right_4, &mut self.swipe_right_4)
                }
                ActionEvents::FourFingerSwipeUp => (&settings.swipe_up_4, &mut self.swipe_up_4),
                ActionEvents::FourFingerSwipeDown => (&settings.swipe_down_4, &mut self.swipe_down_4),
            };

            parse_action_list(settings_field, self_field, &self.connection);
            debug!(" * {}: {}", action_event, self_field.iter().format(", "));
        }

        // Print information.
        info!(
            "Action controller started: {:?}/{:?}/{:?}/{:?} actions enabled",
            self.swipe_left_3.len(),
            self.swipe_right_3.len(),
            self.swipe_up_3.len(),
            self.swipe_down_3.len(),
        );
    }

    fn end_event_to_action_event(
        &mut self,
        dx: &f64,
        dy: &f64,
        finger_count: i32,
    ) -> Option<ActionEvents> {
        // Avoid acting if the displacement is below the threshold.
        if dx.abs() < self.threshold && dy.abs() < self.threshold {
            debug!("Received end event below threshold, discarding");
            return None;
        }

        // Determine finger count and avoid acting if the number of fingers is not supported.
        let finger_count_as_enum = match FingerCount::try_from(finger_count) {
            Ok(count) => count,
            Err(_) => {
                debug!("Received end event with unsupported finger count, discarding");
                return None;
            }
        };

        // Determine the axis and direction.
        let (axis, positive) = match dx.abs() > dy.abs() {
            true => (Axis::X, dx > &0.0),
            false => (Axis::Y, dy > &0.0),
        };

        // Determine the command for the event.
        match (axis, positive, finger_count_as_enum) {
            (Axis::X, true, FingerCount::ThreeFinger) => Some(ActionEvents::ThreeFingerSwipeRight),
            (Axis::X, false, FingerCount::ThreeFinger) => Some(ActionEvents::ThreeFingerSwipeLeft),
            (Axis::X, true, FingerCount::FourFinger) => Some(ActionEvents::FourFingerSwipeRight),
            (Axis::X, false, FingerCount::FourFinger) => Some(ActionEvents::FourFingerSwipeLeft),
            (Axis::Y, true, FingerCount::ThreeFinger) => Some(ActionEvents::ThreeFingerSwipeUp),
            (Axis::Y, false, FingerCount::ThreeFinger) => Some(ActionEvents::ThreeFingerSwipeDown),
            (Axis::Y, true, FingerCount::FourFinger) => Some(ActionEvents::FourFingerSwipeUp),
            (Axis::Y, false, FingerCount::FourFinger) => Some(ActionEvents::FourFingerSwipeDown),
        }
    }

    fn receive_end_event(&mut self, dx: &f64, dy: &f64, finger_count: i32) {
        let action_event = self.end_event_to_action_event(dx, dy, finger_count);

        // Invoke actions.
        let actions = match action_event {
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
            action_event.unwrap(),
            actions.len()
        );

        for action in actions.iter_mut() {
            action.execute_command();
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ActionController, ActionEvents, ActionMap, Settings};
    use crate::test_utils::default_test_settings;

    #[test]
    /// Test the handling of an event `finger_count` argument.
    fn test_parse_finger_count() {
        // Initialize the command line options and controller.
        let settings: Settings = default_test_settings();
        let mut action_map: ActionMap = ActionController::new(&settings);

        // Trigger right swipe with supported (3) fingers count.
        let action_event = action_map.end_event_to_action_event(&5.0, &0.0, 3);
        assert_eq!(action_event.is_some(), true);
        assert_eq!(
            action_event.unwrap() == ActionEvents::ThreeFingerSwipeRight,
            true
        );

        // Trigger right swipe with supported (4) fingers count.
        let action_event = action_map.end_event_to_action_event(&5.0, &0.0, 4);
        assert_eq!(action_event.is_some(), true);
        assert_eq!(
            action_event.unwrap() == ActionEvents::FourFingerSwipeRight,
            true
        );

        // Trigger right swipe with unsupported (5) fingers count.
        let action_event = action_map.end_event_to_action_event(&5.0, &0.0, 5);
        assert_eq!(action_event.is_none(), true);
    }

    #[test]
    /// Test the handling of an event `threshold` argument.
    fn test_parse_threshold() {
        // Initialize the command line options and controller.
        let settings: Settings = default_test_settings();
        let mut action_map: ActionMap = ActionController::new(&settings);

        // Trigger swipe below threshold.
        let action_event = action_map.end_event_to_action_event(&4.99, &0.0, 3);
        assert_eq!(action_event.is_none(), true);

        // Trigger swipe above threshold.
        let action_event = action_map.end_event_to_action_event(&5.0, &0.0, 3);
        assert_eq!(action_event.is_some(), true);
        assert_eq!(
            action_event.unwrap() == ActionEvents::ThreeFingerSwipeRight,
            true
        );
    }
}
