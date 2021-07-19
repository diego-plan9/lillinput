//! Connect `libinput` gestures to `i3` and others.
//!
//! `lillinput` is a small utility written in Rust for connecting `libinput`
//! gestures into:
//! * commands for the `i3` tiling window manager IPC interface
//! * shell commands

use input::Libinput;

use clap::{AppSettings, Clap};
use strum::{Display, EnumString, EnumVariantNames, VariantNames};

mod actions;
use actions::{ActionController, ActionMap};

mod events;
use events::libinput::Interface;
use events::main_loop;

/// Possible choices for actions.
#[derive(Display, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
enum ActionChoices {
    I3,
    Command,
}

#[allow(clippy::enum_variant_names)]
enum ActionEvents {
    ThreeFingerSwipeLeft,
    ThreeFingerSwipeRight,
    ThreeFingerSwipeUp,
    ThreeFingerSwipeDown,
}

/// Connect libinput gestures to i3 and others.
#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// libinput seat.
    #[clap(short, long, default_value = "seat0")]
    seat: String,
    /// enabled actions.
    #[clap(short, long, default_value = "i3", possible_values = ActionChoices::VARIANTS)]
    enabled_actions: Vec<String>,
    /// minimum threshold for position changes.
    #[clap(short, long, default_value = "1.0")]
    threshold: f64,
    /// actions the three-finger swipe left
    #[clap(long, validator = is_action_string, default_value_if("enabled-actions", Some("i3"), "i3:workspace prev"))]
    swipe_left_3: Vec<String>,
    /// actions the three-finger swipe right
    #[clap(long, validator = is_action_string, default_value_if("enabled-actions", Some("i3"), "i3:workspace next"))]
    swipe_right_3: Vec<String>,
    /// actions the three-finger swipe up
    #[clap(long, validator = is_action_string)]
    swipe_up_3: Vec<String>,
    /// actions the three-finger swipe down
    #[clap(long, validator = is_action_string)]
    swipe_down_3: Vec<String>,
    /// allow passing nocapture as cargo test argument.
    /// TODO: handle more gracefully.
    #[cfg(test)]
    #[allow(dead_code)]
    #[clap(long)]
    nocapture: bool,
    /// allow passing test-threads as cargo test argument.
    /// TODO: handle more gracefully.
    #[cfg(test)]
    #[allow(dead_code)]
    #[clap(long, default_value = "1")]
    test_threads: u8,
}

/// Validator for arguments that specify an action.
///
/// A string that specifies an action must conform to the following format:
/// {action choice}:{value}.
fn is_action_string(value: &str) -> Result<(), String> {
    if ActionChoices::VARIANTS
        .iter()
        .any(|&i| value.starts_with(&(i.to_owned() + ":")))
    {
        return Ok(());
    }
    Err(format!(
        "The value does not start with a valid action ({:?})",
        ActionChoices::VARIANTS
    ))
}

/// Main entry point.
fn main() {
    let opts: Opts = Opts::parse();

    // Create the libinput object.
    let mut input = Libinput::new_with_udev(Interface {});
    input.udev_assign_seat(opts.seat.as_str()).unwrap();

    // Create the action map controller.
    let mut action_map: ActionMap = ActionController::new(&opts);
    action_map.populate_actions(&opts);

    // Start the main loop.
    main_loop(input, &mut action_map);
}