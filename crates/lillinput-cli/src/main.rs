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

mod opts;
mod settings;

use crate::opts::Opts;
use crate::settings::{extract_action_map, setup_application};
use lillinput::controllers::defaultcontroller::DefaultController;
use lillinput::events::libinput::initialize_context;
use lillinput::events::main_loop;

use clap::Parser;
use log::{error, info};
use std::process;

#[cfg(test)]
mod test_utils;

/// Main entry point.
fn main() {
    // Retrieve the application settings and setup logging.
    let opts = Opts::parse();
    let settings = setup_application(opts, true);

    // Prepare the action map.
    let (actions, _) = extract_action_map(&settings);

    // Create the controller.
    let mut controller: DefaultController = DefaultController::new(settings.threshold, actions);

    // Create the libinput object.
    let input = initialize_context(&settings.seat).unwrap_or_else(|e| {
        error!("Unable to initialize libinput: {e}");
        process::exit(1);
    });

    // Start the main loop.
    info!("Listening for events ...");
    if let Err(e) = main_loop(input, &mut controller) {
        error!("Unhandled error during the main loop: {}", e);
        process::exit(1);
    }
}
