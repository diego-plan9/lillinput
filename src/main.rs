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

mod actions;
mod events;
mod opts;
mod settings;

use crate::actions::{ActionController, ActionMap, ActionTypes};
use crate::events::ActionEvents;
use crate::opts::Opts;
use clap::Parser;
use events::libinput::initialize_context;
use events::main_loop;
use log::{error, info};
use settings::{setup_application, Settings};
use std::process;

#[cfg(test)]
mod test_utils;

/// Main entry point.
fn main() {
    // Retrieve the application settings and setup logging.
    let opts: Opts = Opts::parse();
    let settings: Settings = setup_application(opts, true);

    // Create the action map controller.
    let mut action_map: ActionMap = ActionController::new(&settings);
    action_map.populate_actions(&settings);

    // Create the libinput object.
    let input = initialize_context(&settings.seat).unwrap_or_else(|e| {
        error!("Unable to initialize libinput: {e}");
        process::exit(1);
    });

    // Start the main loop.
    info!("Listening for events ...");
    if let Err(e) = main_loop(input, &mut action_map) {
        error!("Unhandled error during the main loop: {}", e);
        process::exit(1);
    }
}
