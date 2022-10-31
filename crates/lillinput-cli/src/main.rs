//! Connect `libinput` gestures to `i3` and others.
//!
//! `lillinput` is a small for connecting `libinput` gestures into:
//! * commands for the `i3` tiling window manager IPC interface
//! * shell commands

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
