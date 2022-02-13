//! Connect `libinput` gestures to `i3` and others.
//!
//! `lillinput` is a small for connecting `libinput` gestures into:
//! * commands for the `i3` tiling window manager IPC interface
//! * shell commands

use input::Libinput;

use clap::Parser;
use log::{error, info};
use strum::{Display, EnumString, EnumVariantNames, VariantNames};
use strum_macros::EnumIter;

mod actions;
use actions::{ActionController, ActionMap};

mod events;
use events::libinput::Interface;
use events::main_loop;

mod settings;
use settings::{setup_application, Settings};

#[cfg(test)]
mod test_utils;

/// Possible choices for action types.
#[derive(Display, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
enum ActionTypes {
    I3,
    Command,
}

/// High-level events that can trigger an action.
#[derive(Display, EnumIter, EnumString, EnumVariantNames, Eq, Hash, PartialEq)]
#[strum(serialize_all = "kebab_case")]
#[allow(clippy::enum_variant_names)]
pub enum ActionEvents {
    ThreeFingerSwipeLeft,
    ThreeFingerSwipeRight,
    ThreeFingerSwipeUp,
    ThreeFingerSwipeDown,
    FourFingerSwipeLeft,
    FourFingerSwipeRight,
    FourFingerSwipeUp,
    FourFingerSwipeDown,
}

/// Connect libinput gestures to i3 and others.
#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
pub struct Opts {
    /// Configuration file.
    #[clap(short, long)]
    config_file: Option<String>,
    /// Level of verbosity (additive, can be used up to 3 times)
    #[clap(short, long, parse(from_occurrences))]
    verbose: i64,
    /// libinput seat
    #[clap(short, long)]
    seat: Option<String>,
    /// enabled action types
    #[clap(short, long, possible_values = ActionTypes::VARIANTS)]
    enabled_action_types: Option<Vec<String>>,
    /// minimum threshold for displacement changes
    #[clap(short, long)]
    threshold: Option<f64>,
    /// actions the three-finger swipe left
    #[clap(long, validator = is_action_string)]
    three_finger_swipe_left: Option<Vec<String>>,
    /// actions the three-finger swipe right
    #[clap(long, validator = is_action_string)]
    three_finger_swipe_right: Option<Vec<String>>,
    /// actions the three-finger swipe up
    #[clap(long, validator = is_action_string)]
    three_finger_swipe_up: Option<Vec<String>>,
    /// actions the three-finger swipe down
    #[clap(long, validator = is_action_string)]
    three_finger_swipe_down: Option<Vec<String>>,
    /// actions the four-finger swipe left
    #[clap(long, validator = is_action_string)]
    four_finger_swipe_left: Option<Vec<String>>,
    /// actions the four-finger swipe right
    #[clap(long, validator = is_action_string)]
    four_finger_swipe_right: Option<Vec<String>>,
    /// actions the four-finger swipe up
    #[clap(long, validator = is_action_string)]
    four_finger_up_down: Option<Vec<String>>,
    /// actions the four-finger swipe down
    #[clap(long, validator = is_action_string)]
    four_finger_swipe_down: Option<Vec<String>>,
}

/// Validator for arguments that specify an action.
///
/// A string that specifies an action must conform to the following format:
/// {action choice}:{value}.
///
/// # Arguments
///
/// * `value` - argument to be parsed.
fn is_action_string(value: &str) -> Result<(), String> {
    let (action_type, _) = match value.split_once(':') {
        Some(v) => v,
        None => {
            return Err(format!(
                "The value does not conform to the action string pattern ({:?})",
                value
            ))
        }
    };

    match ActionTypes::VARIANTS.iter().any(|s| s == &action_type) {
        true => Ok(()),
        false => Err(format!(
            "The value does not start with a valid action ({:?})",
            ActionTypes::VARIANTS
        )),
    }
}

/// Main entry point.
fn main() {
    // Retrieve the application settings and setup logging.
    let opts: Opts = Opts::parse();
    let settings: Settings = setup_application(opts);

    // Create the action map controller.
    let mut action_map: ActionMap = ActionController::new(&settings);
    action_map.populate_actions(&settings);

    // Create the libinput object.
    let mut input = Libinput::new_with_udev(Interface {});
    input.udev_assign_seat(settings.seat.as_str()).unwrap();
    info!(
        "Assigned seat {:?} to the libinput context. Listening for events ...",
        settings.seat
    );

    // Start the main loop.
    if let Err(e) = main_loop(input, &mut action_map) {
        error!("Unhandled error during the main loop: {}", e.message)
    }
}

#[cfg(test)]
mod test {
    use clap::Parser;
    use crate::Opts;
    use crate::test_utils::{default_test_settings, init_listener};
    use crate::settings::{setup_application, Settings};
    use std::collections::HashMap;

    #[test]
    #[should_panic(expected = "The value does not conform to the action string pattern")]
    /// Test passing an action string as a parameter with invalid pattern.
    fn test_action_argument_invalid_pattern() {
        Opts::try_parse_from(&["lillinput", "--three-finger-swipe-left", "invalid"]).unwrap();
    }

    #[test]
    #[should_panic(expected = "The value does not start with a valid action")]
    /// Test passing an action string as a parameter with invalid pattern.
    fn test_action_argument_invalid_action_string() {
        Opts::try_parse_from(&["lillinput", "--three-finger-swipe-left", "invalid:bar"]).unwrap();
    }

    #[test]
    /// Test passing an action string as a parameter.
    fn test_action_argument_valid_action_string() {
        let opts: Opts = Opts::parse_from(&["lillinput", "--three-finger-swipe-left", "i3:foo"]);
        assert_eq!(opts.three_finger_swipe_left.unwrap(), vec![String::from("i3:foo")]);
    }

    #[test]
    #[should_panic(expected = "isn't a valid value for")]
    /// Test passing an invalid enabled action type as a parameter.
    fn test_enabled_action_types_argument_invalid() {
        Opts::try_parse_from(&["lillinput", "--enabled-action-types", "invalid"]).unwrap();
    }


}
