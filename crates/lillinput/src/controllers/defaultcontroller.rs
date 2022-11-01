//! Default [`Controller`] for actions.

use std::collections::HashMap;

use crate::actions::Action;
use crate::controllers::errors::ControllerError;
use crate::controllers::Controller;
use crate::events::defaultprocessor::DefaultProcessor;
use crate::events::{ActionEvent, Processor};

use itertools::Itertools;
use log::{debug, info, warn};
use strum::IntoEnumIterator;

/// Controller that maps between events and actions.
pub struct DefaultController {
    /// Processor for events.
    pub processor: Box<dyn Processor>,
    /// Map between events and actions.
    pub actions: HashMap<ActionEvent, Vec<Box<dyn Action>>>,
}

impl DefaultController {
    /// Return a new [`DefaultController`].
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum threshold for displacement changes.
    /// * `actions` - List of action for each action event.
    #[must_use]
    pub fn new(
        processor: Box<dyn Processor>,
        actions: HashMap<ActionEvent, Vec<Box<dyn Action>>>,
    ) -> Self {
        let controller = DefaultController { processor, actions };
        controller._log_status_info();

        controller
    }

    /// Return the status of the controller in printable form.
    fn _log_status_info(&self) {
        // Print information.
        for action_event in ActionEvent::iter() {
            debug!(
                " * {}: {}",
                action_event,
                self.actions
                    .get(&action_event)
                    .unwrap_or(&vec![])
                    .iter()
                    .format(", ")
            );
        }
        let three_finger_counts: String = ActionEvent::iter()
            .take(8)
            .map(|x| format!("{:?}/", self.actions.get(&x).unwrap_or(&vec![]).len()))
            .collect();
        let four_finger_counts: String = ActionEvent::iter()
            .skip(8)
            .map(|x| format!("{:?}/", self.actions.get(&x).unwrap_or(&vec![]).len()))
            .collect();
        info!(
            "{}, {} actions enabled",
            &three_finger_counts.as_str()[0..three_finger_counts.len() - 1],
            &four_finger_counts.as_str()[0..four_finger_counts.len() - 1],
        );
    }
}

impl Default for DefaultController {
    fn default() -> Self {
        DefaultController::new(Box::new(DefaultProcessor::default()), HashMap::new())
    }
}

impl Controller for DefaultController {
    fn process_action_event(&mut self, action_event: ActionEvent) -> Result<(), ControllerError> {
        // Invoke actions.
        let actions = self
            .actions
            .get_mut(&action_event)
            .ok_or(ControllerError::NoActionsRegistered(action_event))?;

        debug!(
            "Received end event: {}, triggering {} actions",
            action_event,
            actions.len()
        );

        for action in actions.iter_mut() {
            match action.execute_command() {
                Ok(_) => (),
                Err(e) => warn!("Error execution action {action}: {}", e),
            }
        }

        Ok(())
    }

    fn run(&mut self) -> Result<(), ControllerError> {
        // Variables for tracking the cursor position changes.
        let mut dx: f64 = 0.0;
        let mut dy: f64 = 0.0;

        loop {
            let events = self.processor.dispatch(&mut dx, &mut dy)?;

            for event in events {
                match self.process_action_event(event) {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("Discarding event: {}", e);
                    }
                }
            }
        }
    }
}
