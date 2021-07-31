//! Components for capturing and handling events.

use std::os::unix::io::{AsRawFd, RawFd};

use filedescriptor::{poll, pollfd, POLLIN};
use input::event::gesture::{GestureEvent, GestureEventCoordinates, GestureSwipeEvent};
use input::event::Event;
use input::Libinput;
use log::warn;

use super::actions::{ActionController, ActionMap};

pub mod libinput;

/// Process a single `GestureEvent`.
///
/// # Arguments
///
/// * `event` - a gesture event
/// * `dx` - the current position in the `x` axis
/// * `dy` - the current position in the `y` axis
/// * `action_map` - the action map that will process the event
fn process_event(event: GestureEvent, dx: &mut f64, dy: &mut f64, action_map: &mut ActionMap) {
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
            GestureSwipeEvent::End(_end_event) => {
                action_map.receive_end_event(dx, dy);
            }
        }
    }
}

/// Run the main loop for parsing the libinput events.
///
/// # Arguments
///
/// * `input` - the libinput object
/// * `action_map` - the action map that will process the event
pub fn main_loop(mut input: Libinput, action_map: &mut ActionMap) {
    // Variables for tracking the cursor position changes.
    let mut dx: f64 = 0.0;
    let mut dy: f64 = 0.0;

    // Use a raw file descriptor for polling.
    let raw_fd: RawFd = input.as_raw_fd();

    let mut poll_array = [pollfd {
        fd: raw_fd,
        events: POLLIN,
        revents: 0,
    }];

    loop {
        // Block until the descriptor is ready.
        if let Err(e) = poll(&mut poll_array, None) {
            warn!("{:?}", e);
        }

        input.dispatch().unwrap();
        for event in &mut input {
            if let Event::Gesture(gesture_event) = event {
                process_event(gesture_event, &mut dx, &mut dy, action_map);
            }
        }
    }
}
