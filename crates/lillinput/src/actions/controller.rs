//! Controller for actions.

use std::collections::HashMap;
use std::convert::TryFrom;

use crate::actions::errors::ActionControllerError;
use crate::actions::{Action, ActionController, ActionEvent, ActionMap};
use itertools::Itertools;
use log::{debug, info, warn};
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

impl ActionMap {
    /// Return a new [`ActionMap`].
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum threshold for displacement changes.
    /// * `actions` - List of action for each action event.
    pub fn new(threshold: f64, actions: HashMap<ActionEvent, Vec<Box<dyn Action>>>) -> Self {
        let action_map = ActionMap { threshold, actions };

        info!(
            "Action controller started: {}",
            action_map._get_status_info()
        );
        action_map
    }

    /// Return the status of the controller in printable form.
    fn _get_status_info(&self) -> String {
        // Print information.
        for action_event in ActionEvent::iter() {
            debug!(
                " * {}: {}",
                action_event,
                self.actions.get(&action_event).unwrap().iter().format(", ")
            );
        }
        let three_finger_counts: String = ActionEvent::iter()
            .take(4)
            .map(|x| format!("{:?}/", self.actions.get(&x).unwrap().len()))
            .collect();
        let four_finger_counts: String = ActionEvent::iter()
            .skip(4)
            .map(|x| format!("{:?}/", self.actions.get(&x).unwrap().len()))
            .collect();
        format!(
            "{}, {} actions enabled",
            &three_finger_counts.as_str()[0..three_finger_counts.len() - 1],
            &four_finger_counts.as_str()[0..four_finger_counts.len() - 1],
        )
    }
}

impl ActionController for ActionMap {
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

    fn end_event_to_action_event(
        &mut self,
        mut dx: f64,
        mut dy: f64,
        finger_count: i32,
    ) -> Result<ActionEvent, ActionControllerError> {
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
            (Axis::X, true, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeRight,
            (Axis::X, false, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeLeft,
            (Axis::X, true, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeRight,
            (Axis::X, false, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeLeft,
            (Axis::Y, true, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeUp,
            (Axis::Y, false, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeDown,
            (Axis::Y, true, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeUp,
            (Axis::Y, false, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeDown,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::actions::controller::{ActionController, ActionEvent, ActionMap};
    use crate::actions::errors::ActionControllerError;
    use crate::extract_action_map;
    use crate::test_utils::default_test_settings;
    use crate::Settings;

    #[test]
    /// Test the handling of an event `finger_count` argument.
    fn test_parse_finger_count() {
        // Initialize the command line options and controller.
        let settings: Settings = default_test_settings();
        let (actions, _) = extract_action_map(&settings);
        let mut action_map: ActionMap = ActionMap::new(settings.threshold, actions);

        // Trigger right swipe with supported (3) fingers count.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRight,);

        // Trigger right swipe with supported (4) fingers count.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 4);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRight,);

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
        let (actions, _) = extract_action_map(&settings);
        let mut action_map: ActionMap = ActionMap::new(settings.threshold, actions);

        // Trigger swipe below threshold.
        let action_event = action_map.end_event_to_action_event(4.99, 0.0, 3);
        assert_eq!(
            action_event,
            Err(ActionControllerError::DisplacementBelowThreshold(5.0))
        );

        // Trigger swipe above threshold.
        let action_event = action_map.end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRight,);
    }
}
