//! Default [`Processor`] for events.

use crate::events::errors::{LibinputError, ProcessorError};
use crate::events::libinput::Interface;
use crate::events::{ActionEvent, Axis, FingerCount, Processor};

use std::os::unix::io::{AsRawFd, RawFd};

use filedescriptor::{poll, pollfd, POLLIN};
use input::event::gesture::{
    GestureEvent, GestureEventCoordinates, GestureEventTrait, GestureSwipeEvent,
};
use input::event::Event;
use input::Libinput;
use log::{debug, info};

/// Default [`Processor`] for events.
pub struct DefaultProcessor {
    /// Minimum threshold for displacement changes.
    pub threshold: f64,
    /// Libinput context.
    pub input: Libinput,
    /// File descriptor poll structure.
    pub poll_array: Vec<pollfd>,
}

impl DefaultProcessor {
    /// Return a new [`DefaultProcessor`].
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum threshold for displacement changes.
    /// * `seat_id` - `libinput` seat id.
    ///
    /// # Errors
    ///
    /// Return `Err` if the `libinput` initialization failed.
    pub fn new(threshold: f64, seat_id: &str) -> Result<Self, LibinputError> {
        // Create the libinput context.
        let mut input = Libinput::new_with_udev(Interface {});
        input
            .udev_assign_seat(seat_id)
            .map_err(|_| LibinputError::SeatError)?;

        info!("Assigned seat {seat_id} to the libinput context.");
        // Use a raw file descriptor for polling.
        let raw_fd: RawFd = input.as_raw_fd();

        let poll_array = [pollfd {
            fd: raw_fd,
            events: POLLIN,
            revents: 0,
        }]
        .to_vec();

        Ok(DefaultProcessor {
            threshold,
            input,
            poll_array,
        })
    }
}

impl Processor for DefaultProcessor {
    fn process_event(
        &mut self,
        event: GestureEvent,
        dx: &mut f64,
        dy: &mut f64,
    ) -> Result<Option<ActionEvent>, ProcessorError> {
        if let GestureEvent::Swipe(event) = event {
            match event {
                GestureSwipeEvent::Begin(_begin_event) => {
                    (*dx) = 0.0;
                    (*dy) = 0.0;
                }
                GestureSwipeEvent::Update(update_event) => {
                    (*dx) += update_event.dx();
                    (*dy) += update_event.dy();
                }
                GestureSwipeEvent::End(ref _end_event) => {
                    return match self._end_event_to_action_event(*dx, *dy, event.finger_count()) {
                        Ok(event) => Ok(Some(event)),
                        Err(e) => Err(e),
                    };
                }
                // GestureEvent::Swipe is non-exhaustive.
                other => return Err(ProcessorError::UnsupportedSwipeEvent(other)),
            }
        }

        Ok(None)
    }

    fn _end_event_to_action_event(
        &mut self,
        mut dx: f64,
        mut dy: f64,
        finger_count: i32,
    ) -> Result<ActionEvent, ProcessorError> {
        // Determine finger count.
        let finger_count_as_enum = FingerCount::try_from(finger_count)?;

        // Trim displacements according to threshold.
        dx = if dx.abs() < self.threshold { 0.0 } else { dx };
        dy = if dy.abs() < self.threshold { 0.0 } else { dy };
        if dx == 0.0 && dy == 0.0 {
            return Err(ProcessorError::DisplacementBelowThreshold(self.threshold));
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

    fn dispatch(&mut self, dx: &mut f64, dy: &mut f64) -> Result<Vec<ActionEvent>, LibinputError> {
        // Block until the descriptor is ready.
        poll(&mut self.poll_array, None)?;

        // Dispatch, bubbling up in case of an error.
        self.input.dispatch()?;

        let mut action_events = Vec::new();
        let events: Vec<Event> = (&mut self.input).collect();

        for event in events {
            if let Event::Gesture(gesture_event) = event {
                let result = self.process_event(gesture_event, dx, dy);

                match result {
                    Err(e) => {
                        debug!("Discarding event: {}", e);
                    }
                    Ok(None) => {}
                    Ok(Some(action_event)) => action_events.push(action_event),
                }
            }
        }

        Ok(action_events)
    }
}

#[cfg(test)]
mod test {
    use super::DefaultProcessor;
    use crate::events::errors::ProcessorError;
    use crate::events::ActionEvent;
    use crate::events::Processor;
    use crate::test_utils::init_listener;

    use std::sync::{Arc, Mutex};

    use serial_test::serial;

    #[test]
    #[serial]
    /// Test the handling of an event `finger_count` argument.
    fn test_parse_finger_count() {
        // Create the listener and the shared storage for the commands.
        let message_log = Arc::new(Mutex::new(vec![]));
        let socket_file = init_listener(Arc::clone(&message_log));

        // Initialize the processor.
        let mut processor = DefaultProcessor::new(5.0, "seat0").unwrap();

        // Trigger right swipe with supported (3) fingers count.
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRight,);

        // Trigger right swipe with supported (4) fingers count.
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 4);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRight,);

        // Trigger right swipe with unsupported (5) fingers count.
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 5);
        assert!(action_event.is_err());
        assert!(matches!(
            action_event,
            Err(ProcessorError::UnsupportedFingerCount(5))
        ));
        std::fs::remove_file(socket_file.path().file_name().unwrap()).ok();
    }

    #[test]
    #[serial]
    /// Test the handling of an event `threshold` argument.
    fn test_parse_threshold() {
        // Create the listener and the shared storage for the commands.
        let message_log = Arc::new(Mutex::new(vec![]));
        let socket_file = init_listener(Arc::clone(&message_log));

        // Initialize the processor.
        let mut processor = DefaultProcessor::new(5.0, "seat0").unwrap();

        // Trigger swipe below threshold.
        let action_event = processor._end_event_to_action_event(4.99, 0.0, 3);
        #[allow(clippy::no_effect_underscore_binding)]
        let _expected_err = ProcessorError::DisplacementBelowThreshold(5.0);
        assert!(matches!(action_event, Err(_expected_err)));

        // Trigger swipe above threshold.
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRight,);
        std::fs::remove_file(socket_file.path().file_name().unwrap()).ok();
    }
}
