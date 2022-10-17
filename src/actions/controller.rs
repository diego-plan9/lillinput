//! Controller for actions.

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use std::str::FromStr;

use crate::actions::commandaction::CommandAction;
use crate::actions::errors::ActionControllerError;
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
    type Error = ActionControllerError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(FingerCount::ThreeFinger),
            4 => Ok(FingerCount::FourFinger),
            _ => Err(ActionControllerError::UnsupportedFingerCount(value)),
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
        mut dx: f64,
        mut dy: f64,
        finger_count: i32,
    ) -> Result<ActionEvents, ActionControllerError> {
        // Determine finger count.
        let finger_count_as_enum = FingerCount::try_from(finger_count)?;

        // Trim displacements according to threshold.
        dx = if dx.abs() < self.threshold { 0.0 } else { dx };
        dy = if dy.abs() < self.threshold { 0.0 } else { dy };
        if dx == 0.0 && dy == 0.0 {
            return Err(ActionControllerError::DisplacementBelowThreshold(
                self.threshold,
            ));
        }

        // Determine the axis and direction.
        let (axis, positive) = if dx.abs() > dy.abs() {
            (Axis::X, dx > 0.0)
        } else {
            (Axis::Y, dy > 0.0)
        };

        // Determine the command for the event.
        Ok(match (axis, positive, finger_count_as_enum) {
            (Axis::X, true, FingerCount::ThreeFinger) => ActionEvents::ThreeFingerSwipeRight,
            (Axis::X, false, FingerCount::ThreeFinger) => ActionEvents::ThreeFingerSwipeLeft,
            (Axis::X, true, FingerCount::FourFinger) => ActionEvents::FourFingerSwipeRight,
            (Axis::X, false, FingerCount::FourFinger) => ActionEvents::FourFingerSwipeLeft,
            (Axis::Y, true, FingerCount::ThreeFinger) => ActionEvents::ThreeFingerSwipeUp,
            (Axis::Y, false, FingerCount::ThreeFinger) => ActionEvents::ThreeFingerSwipeDown,
            (Axis::Y, true, FingerCount::FourFinger) => ActionEvents::FourFingerSwipeUp,
            (Axis::Y, false, FingerCount::FourFinger) => ActionEvents::FourFingerSwipeDown,
        })
    }

    fn receive_end_event(
        &mut self,
        dx: f64,
        dy: f64,
        finger_count: i32,
    ) -> Result<(), ActionControllerError> {
        let action_event = self.end_event_to_action_event(dx, dy, finger_count)?;

        // Invoke actions.
        let actions = self
            .actions
            .get_mut(&action_event)
            .ok_or(ActionControllerError::NoActionsRegistered(action_event))?;

        debug!(
            "Received end event: {}, triggering {} actions",
            action_event,
            actions.len()
        );

        for action in actions.iter_mut() {
            match action.execute_command() {
                Ok(_) => (),
                Err(e) => warn!("{}", e),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::actions::controller::{ActionController, ActionEvents, ActionMap, Settings};
    use crate::actions::errors::ActionControllerError;
    use crate::test_utils::default_test_settings;

    #[test]
    /// Test the handling of an event `finger_count` argument.
    fn test_parse_finger_count() {
        // Initialize the command line options and controller.
        let settings: Settings = default_test_settings();
        let mut action_map: ActionMap = ActionController::new(&settings);

        // Trigger right swipe with supported (3) fingers count.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvents::ThreeFingerSwipeRight,);

        // Trigger right swipe with supported (4) fingers count.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 4);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvents::FourFingerSwipeRight,);

        // Trigger right swipe with unsupported (5) fingers count.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 5);
        assert!(action_event.is_err());
        assert_eq!(
            action_event,
            Err(ActionControllerError::UnsupportedFingerCount(5))
        );
    }

    #[test]
    /// Test the handling of an event `threshold` argument.
    fn test_parse_threshold() {
        // Initialize the command line options and controller.
        let settings: Settings = default_test_settings();
        let mut action_map: ActionMap = ActionController::new(&settings);

        // Trigger swipe below threshold.
        let action_event = action_map.end_event_to_action_event(4.99, 0.0, 3);
        assert_eq!(
            action_event,
            Err(ActionControllerError::DisplacementBelowThreshold(5.0))
        );

        // Trigger swipe above threshold.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvents::ThreeFingerSwipeRight,);
    }
}
