//! Controller for actions.

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use std::str::FromStr;

use crate::actions::commandaction::CommandAction;
use crate::actions::i3action::{I3Action, I3ActionExt};
use crate::actions::{Action, ActionController, ActionEvents, ActionExt, ActionMap, ActionTypes};
use crate::Settings;
use i3ipc::I3Connection;
use itertools::Itertools;
use log::{debug, info, warn};
use std::convert::TryInto;
use strum::IntoEnumIterator;

/// Possible choices for finger count.
enum FingerCount {
    /// Three fingers.
    ThreeFinger = 3,
    /// Four fingers.
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

/// Axis of a swipe action.
enum Axis {
    /// Horizontal (`X`) axis.
    X,
    /// Vertical (`Y`) axis.
    Y,
}

impl ActionController for ActionMap {
    fn new(settings: &Settings) -> Self {
        // Create the I3 connection if needed.
        let connection = if settings
            .enabled_action_types
            .contains(&ActionTypes::I3.to_string())
        {
            match I3Connection::connect() {
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
            }
        } else {
            None
        };

        let default_actions: [(ActionEvents, Vec<_>); 8] = ActionEvents::iter()
            .map(|x| (x, Vec::new()))
            .collect::<Vec<(ActionEvents, Vec<_>)>>()
            .try_into()
            .unwrap();

        ActionMap {
            threshold: settings.threshold,
            connection,
            actions: HashMap::from(default_actions),
        }
    }

    fn populate_actions(&mut self, settings: &Settings) {
        /// Convert an stringified action list into individual actions.
        ///
        /// # Arguments
        ///.
        /// * `arguments` - list of command line arguments.
        /// * `connection` - optional i3 connection.
        fn parse_action_list(
            arguments: &[String],
            connection: &Option<Rc<RefCell<I3Connection>>>,
        ) -> Vec<Box<dyn Action>> {
            let mut actions_list: Vec<Box<dyn Action>> = vec![];

            for value in arguments.iter() {
                // Split the arguments, in the form "{type}:{value}".
                let (action_type, action_value) = value.split_once(':').unwrap();

                // Create the new actions.
                match ActionTypes::from_str(action_type) {
                    Ok(ActionTypes::Command) => {
                        actions_list.push(Box::new(CommandAction::new(action_value.to_string())));
                    }
                    Ok(ActionTypes::I3) => match connection {
                        Some(conn) => {
                            actions_list.push(Box::new(I3Action::new(
                                action_value.to_string(),
                                Rc::clone(conn),
                            )));
                        }
                        None => {
                            warn!("ignoring i3 action, as the i3 connection could not be set.");
                        }
                    },
                    Err(_) => {
                        warn!("Unknown action type: '{}", action_type);
                    }
                }
            }

            actions_list
        }

        // Populate the fields for each `ActionEvent`.
        for action_event in ActionEvents::iter() {
            if let Some(arguments) = settings.actions.get(&action_event.to_string()) {
                let parsed_actions = parse_action_list(arguments, &self.connection);
                self.actions.insert(action_event, parsed_actions);
            }
        }

        // Print information.
        for action_event in ActionEvents::iter() {
            debug!(
                " * {}: {}",
                action_event,
                self.actions.get(&action_event).unwrap().iter().format(", ")
            );
        }
        let three_finger_counts: String = ActionEvents::iter()
            .take(4)
            .map(|x| format!("{:?}/", self.actions.get(&x).unwrap().len()))
            .collect();
        let four_finger_counts: String = ActionEvents::iter()
            .skip(4)
            .map(|x| format!("{:?}/", self.actions.get(&x).unwrap().len()))
            .collect();
        info!(
            "Action controller started: {}, {} actions enabled",
            &three_finger_counts.as_str()[0..three_finger_counts.len() - 1],
            &four_finger_counts.as_str()[0..four_finger_counts.len() - 1],
        );
    }

    fn end_event_to_action_event(
        &mut self,
        dx: f64,
        dy: f64,
        finger_count: i32,
    ) -> Option<ActionEvents> {
        // Avoid acting if the displacement is below the threshold.
        if dx.abs() < self.threshold && dy.abs() < self.threshold {
            debug!("Received end event below threshold, discarding");
            return None;
        }

        // Determine finger count and avoid acting if the number of fingers is not supported.
        let finger_count_as_enum = if let Ok(count) = FingerCount::try_from(finger_count) {
            count
        } else {
            debug!("Received end event with unsupported finger count, discarding");
            return None;
        };

        // Determine the axis and direction.
        let (axis, positive) = if dx.abs() > dy.abs() {
            (Axis::X, dx > 0.0)
        } else {
            (Axis::Y, dy > 0.0)
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

    fn receive_end_event(&mut self, dx: f64, dy: f64, finger_count: i32) {
        let action_event = self.end_event_to_action_event(dx, dy, finger_count);

        // Invoke actions.
        let actions = match action_event {
            Some(ref event) => self.actions.get_mut(event).unwrap(),
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
    use crate::actions::controller::{ActionController, ActionEvents, ActionMap, Settings};
    use crate::test_utils::default_test_settings;

    #[test]
    /// Test the handling of an event `finger_count` argument.
    fn test_parse_finger_count() {
        // Initialize the command line options and controller.
        let settings: Settings = default_test_settings();
        let mut action_map: ActionMap = ActionController::new(&settings);

        // Trigger right swipe with supported (3) fingers count.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_some());
        assert!(action_event.unwrap() == ActionEvents::ThreeFingerSwipeRight,);

        // Trigger right swipe with supported (4) fingers count.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 4);
        assert!(action_event.is_some());
        assert!(action_event.unwrap() == ActionEvents::FourFingerSwipeRight,);

        // Trigger right swipe with unsupported (5) fingers count.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 5);
        assert!(action_event.is_none());
    }

    #[test]
    /// Test the handling of an event `threshold` argument.
    fn test_parse_threshold() {
        // Initialize the command line options and controller.
        let settings: Settings = default_test_settings();
        let mut action_map: ActionMap = ActionController::new(&settings);

        // Trigger swipe below threshold.
        let action_event = action_map.end_event_to_action_event(4.99, 0.0, 3);
        assert!(action_event.is_none());

        // Trigger swipe above threshold.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_some());
        assert!(action_event.unwrap() == ActionEvents::ThreeFingerSwipeRight,);
    }
}
