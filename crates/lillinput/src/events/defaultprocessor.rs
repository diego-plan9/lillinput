//! Default [`Processor`] for events.

use crate::events::errors::{LibinputError, ProcessorError};
use crate::events::libinput::Interface;
use crate::events::{ActionEvent, FingerCount, Processor};

use std::f64::consts::PI;
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
    /// Whether positive displacement on the `X` axis should be interpreted as
    /// "left".
    pub invert_x: bool,
    /// Whether positive displacement on the `Y` axis should be interpreted as
    /// "up".
    pub invert_y: bool,
}

impl DefaultProcessor {
    /// Return a new [`DefaultProcessor`].
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum threshold for displacement changes.
    /// * `seat_id` - `libinput` seat id.
    /// * `invert_x` - Whether positive displacement on the `X` axis should be
    ///   interpreted as "left".
    /// * `invert_y` - Whether positive displacement on the `Y` axis should be
    ///   interpreted as "up".
    ///
    /// # Errors
    ///
    /// Return `Err` if the `libinput` initialization failed.
    pub fn new(
        threshold: f64,
        seat_id: &str,
        invert_x: bool,
        invert_y: bool,
    ) -> Result<Self, LibinputError> {
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
            invert_x,
            invert_y,
        })
    }
}

impl Default for DefaultProcessor {
    fn default() -> Self {
        DefaultProcessor::new(5.0, "seat0", false, false).unwrap()
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
        /// Return the octant for the given displacement.
        ///
        /// # Arguments
        ///
        /// * `x` - the final position in the `x` axis.
        /// * `y` - the final position in the `y` axis.
        ///
        /// # Returns
        ///
        /// The octant the displacement is closest to in the `X-Y` coordinates,
        /// with `0` being the left direction and increasing clock-wise.
        fn get_event_octant(dx: f64, dy: f64) -> i8 {
            // Get the angle, scaled to `[0..1]`.
            let mut angle = -dy.atan2(-dx);
            if angle < 0.0 {
                angle += 2.0 * PI;
            };
            angle /= 2.0 * PI;

            // Get the octant, rounding the angle to the nearest possible of
            // the `8` (determined by the number of `ActionEvents` directions.
            #[allow(clippy::cast_possible_truncation)]
            let mut octant = (angle * 8.0).round() as i8;
            if octant == 8 {
                // Wrap to the initial direction.
                octant = 0;
            }

            octant
        }

        // Determine finger count.
        let finger_count_as_enum = FingerCount::try_from(finger_count)?;

        // Discard displacements below threshold.
        if (dx.powi(2) + dy.powi(2)).sqrt() < self.threshold {
            return Err(ProcessorError::DisplacementBelowThreshold(self.threshold));
        };

        // Determine the `ActionEvent` for the event.
        if self.invert_x {
            dx = -dx;
        }
        if self.invert_y {
            dy = -dy;
        }
        Ok(match (get_event_octant(dx, dy), finger_count_as_enum) {
            (0, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeLeft,
            (1, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeLeftUp,
            (2, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeUp,
            (3, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeRightUp,
            (4, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeRight,
            (5, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeRightDown,
            (6, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeDown,
            (7, FingerCount::ThreeFinger) => ActionEvent::ThreeFingerSwipeLeftDown,

            (0, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeLeft,
            (1, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeLeftUp,
            (2, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeUp,
            (3, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeRightUp,
            (4, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeRight,
            (5, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeRightDown,
            (6, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeDown,
            (7, FingerCount::FourFinger) => ActionEvent::FourFingerSwipeLeftDown,
            (_, _) => todo!(),
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
    use crate::events::{ActionEvent, Processor, ProcessorError};
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
        let mut processor = DefaultProcessor::default();

        // Trigger right swipe with supported (3) fingers count.
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRight);

        // Trigger right swipe with supported (4) fingers count.
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 4);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRight);

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
        let mut processor = DefaultProcessor::default();

        // Trigger swipe below threshold.
        let action_event = processor._end_event_to_action_event(4.99, 0.0, 3);
        #[allow(clippy::no_effect_underscore_binding)]
        let _expected_err = ProcessorError::DisplacementBelowThreshold(5.0);
        assert!(matches!(action_event, Err(_expected_err)));

        // Trigger swipe above threshold.
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.is_ok());
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRight);
        std::fs::remove_file(socket_file.path().file_name().unwrap()).ok();
    }

    #[test]
    #[serial]
    /// Test the handling of different directions.
    fn test_multiple_directions() {
        // Create the listener and the shared storage for the commands.
        let message_log = Arc::new(Mutex::new(vec![]));
        let socket_file = init_listener(Arc::clone(&message_log));

        // Initialize the processor.
        let mut processor = DefaultProcessor::default();

        let s = 5.0f64.powi(2) / 2.0;
        // Trigger swipes for directions at the edges of the threshold.
        let action_event = processor._end_event_to_action_event(-5.0, 0.0, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeLeft);
        let action_event = processor._end_event_to_action_event(-s, -s, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeLeftUp);
        let action_event = processor._end_event_to_action_event(0.0, -5.0, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeUp);
        let action_event = processor._end_event_to_action_event(s, -s, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRightUp);
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRight);
        let action_event = processor._end_event_to_action_event(s, s, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRightDown);
        let action_event = processor._end_event_to_action_event(0.0, 5.0, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeDown);
        let action_event = processor._end_event_to_action_event(-s, s, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeLeftDown);

        let action_event = processor._end_event_to_action_event(-5.0, 0.0, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeLeft);
        let action_event = processor._end_event_to_action_event(-s, -s, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeLeftUp);
        let action_event = processor._end_event_to_action_event(0.0, -5.0, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeUp);
        let action_event = processor._end_event_to_action_event(s, -s, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRightUp);
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRight);
        let action_event = processor._end_event_to_action_event(s, s, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRightDown);
        let action_event = processor._end_event_to_action_event(0.0, 5.0, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeDown);
        let action_event = processor._end_event_to_action_event(-s, s, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeLeftDown);

        std::fs::remove_file(socket_file.path().file_name().unwrap()).ok();
    }

    #[test]
    #[serial]
    /// Test the handling of different inverted directions.
    fn test_multiple_directions_inverted() {
        // Create the listener and the shared storage for the commands.
        let message_log = Arc::new(Mutex::new(vec![]));
        let socket_file = init_listener(Arc::clone(&message_log));

        // Initialize the processor.
        let mut processor = DefaultProcessor {
            invert_x: true,
            invert_y: true,
            ..Default::default()
        };

        let s = 5.0f64.powi(2) / 2.0;
        // Trigger swipes for directions at the edges of the threshold.
        let action_event = processor._end_event_to_action_event(5.0, 0.0, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeLeft);
        let action_event = processor._end_event_to_action_event(s, s, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeLeftUp);
        let action_event = processor._end_event_to_action_event(0.0, 5.0, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeUp);
        let action_event = processor._end_event_to_action_event(-s, s, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRightUp);
        let action_event = processor._end_event_to_action_event(-5.0, 0.0, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRight);
        let action_event = processor._end_event_to_action_event(-s, -s, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeRightDown);
        let action_event = processor._end_event_to_action_event(0.0, -5.0, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeDown);
        let action_event = processor._end_event_to_action_event(s, -s, 3);
        assert!(action_event.unwrap() == ActionEvent::ThreeFingerSwipeLeftDown);

        let action_event = processor._end_event_to_action_event(5.0, 0.0, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeLeft);
        let action_event = processor._end_event_to_action_event(s, s, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeLeftUp);
        let action_event = processor._end_event_to_action_event(0.0, 5.0, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeUp);
        let action_event = processor._end_event_to_action_event(-s, s, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRightUp);
        let action_event = processor._end_event_to_action_event(-5.0, 0.0, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRight);
        let action_event = processor._end_event_to_action_event(-s, -s, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeRightDown);
        let action_event = processor._end_event_to_action_event(0.0, -5.0, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeDown);
        let action_event = processor._end_event_to_action_event(s, -s, 4);
        assert!(action_event.unwrap() == ActionEvent::FourFingerSwipeLeftDown);

        std::fs::remove_file(socket_file.path().file_name().unwrap()).ok();
    }
}
