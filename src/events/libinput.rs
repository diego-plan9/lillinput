//! Components for interacting with `libinput`.

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};
use std::path::Path;

use input::{Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use log::info;
use thiserror::Error;

/// Struct for `libinput` interface.
struct Interface;

impl LibinputInterface for Interface {
    #[allow(clippy::bad_bit_mask)]
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(IntoRawFd::into_raw_fd)
            .map_err(|err| err.raw_os_error().unwrap())
    }

    fn close_restricted(&mut self, fd: RawFd) {
        unsafe {
            File::from_raw_fd(fd);
        }
    }
}

/// Errors raised during `libinput` initialization.
#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("error while assigning seat to the libinput context")]
    SeatError,
}

/// Return an initialized `libinput` context.
///
/// # Arguments
///
/// * `seat_id` - the identifier of the seat.
pub fn initialize_context(seat_id: &str) -> Result<Libinput, InitializationError> {
    // Create the libinput context.
    let mut input = Libinput::new_with_udev(Interface {});
    if input.udev_assign_seat(seat_id).is_err() {
        return Err(InitializationError::SeatError);
    }

    info!("Assigned seat {seat_id} to the libinput context.");
    Ok(input)
}
