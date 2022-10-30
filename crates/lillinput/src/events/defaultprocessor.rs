//! Default [`Processor`] for events.

use crate::events::errors::{LibinputError, ProcessorError};
use crate::events::libinput::Interface;
use crate::events::{ActionEvent, Axis, FingerCount, Processor};

use std::os::unix::io::{AsRawFd, RawFd};

use filedescriptor::{pollfd, POLLIN};
use input::event::gesture::{
    GestureEvent, GestureEventCoordinates, GestureEventTrait, GestureSwipeEvent,
};
use input::Libinput;
use log::info;

/// Default [`Processor`] for events.
pub struct DefaultProcessor {
    /// Minimum threshold for displacement changes.
    pub threshold: f64,
    pub input: Libinput,
    pub poll_array: Vec<pollfd>,
}

impl DefaultProcessor {
    /// Return a new [`DefaultProcessor`].
    pub fn new(threshold: f64, seat_id: &str) -> Result<Self, LibinputError> {
        // Create the libinput context.
        let mut input = Libinput::new_with_udev(Interface {});
        input
            .udev_assign_seat(seat_id)
            .map_err(|_| LibinputError::SeatError)?;

        info!("Assigned seat {seat_id} to the libinput context.");
        // Use a raw file descriptor for polling.
        let raw_fd: RawFd = input.as_raw_fd();

        let mut poll_array = [pollfd {
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
    ) -> Result<(), ProcessorError> {
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
                    self.end_event_to_action_event(*dx, *dy, event.finger_count())?;
                }
                // GestureEvent::Swipe is non-exhaustive.
                other => return Err(ProcessorError::UnsupportedSwipeEvent(other)),
            }
        }

        Ok(())
    }

    fn end_event_to_action_event(
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
}
