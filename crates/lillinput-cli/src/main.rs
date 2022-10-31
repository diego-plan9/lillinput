//! Application for connecting `libinput` gestures to `i3` and others
//!
//! `lillinput` is a small for utility for connecting `libinput` gestures to:
//! * commands for the `i3` tiling window manager `IPC` interface
//! * shell commands
//!
//! This crate provides the command-line application. See also the
//! [`lillinput`] crate for the underlying library.
//!
//! # Configuring the swipe actions
//!
//! Each `--{number}-finger-swipe-{direction}` argument accepts one or several
//! "actions", in the form `{type}:{command}`. For example, the following
//! invocation specifies two actions for the "three finger swipe up" gesture:
//! moving to the next workspace in `i3`, and creating a file.
//!
//! ```bash
//! $ lillinput -e i3 -e command --three-finger-swipe-up "i3:workspace next" --three-finger-swipe-up "command:touch /tmp/myfile"
//! ```
//!
//! Currently, the available action types are `i3` and `command`.
//!
//! ### Using a configuration file
//!
//! The configuration from the application can be read from a configuration file.
//! By default, the following sources will be read in order:
//!
//! 1. `/etc/lillinput.toml`
//! 2. `${XDG_HOME}/lillinput/lillinput.toml`
//! 3. `${CWD}/lillinput.toml`
//!
//! Alternatively, a different file can be specified via the `--config-file`
//! argument. The configuration files can be partial (as in declaring just
//! specific options rather than the full range of options), and each option can be
//! overridden individually by later config files or command line arguments,
//! falling back to their default values if not provided.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown
)]

pub mod opts;
pub mod settings;

use crate::opts::Opts;
use crate::settings::{extract_action_map, setup_application, Settings};
use lillinput::controllers::{Controller, DefaultController};
use lillinput::events::DefaultProcessor;

use clap::Parser;
use log::{error, info};
use std::process;

#[cfg(test)]
mod test_utils;

/// Main entry point.
pub fn main() {
    // Retrieve the application settings and setup logging.
    let opts = Opts::parse();
    let settings = match setup_application(opts, true) {
        Ok(settings) => settings,
        Err(e) => {
            error!("Unable to process settings: {e}. Attempting to proceed with defaults ...");
            Settings::default()
        }
    };

    // Create the Processor.
    let processor = match DefaultProcessor::new(settings.threshold, &settings.seat) {
        Ok(processor) => processor,
        Err(e) => {
            error!("Unable to initialize: {e}");
            process::exit(1);
        }
    };

    // Create the controller.
    let (actions, _) = extract_action_map(&settings);
    let mut controller: DefaultController = DefaultController::new(Box::new(processor), actions);

    // Start the main loop.
    info!("Listening for events ...");
    if let Err(e) = controller.run() {
        error!("Unhandled error during the main loop: {}", e);
        process::exit(1);
    }
}
