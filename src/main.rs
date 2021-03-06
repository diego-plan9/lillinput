//! Connect `libinput` gestures to `i3` and others.
//!
//! `lillinput` is a small for connecting `libinput` gestures into:
//! * commands for the `i3` tiling window manager IPC interface
//! * shell commands

mod actions;
mod events;
mod settings;

use actions::{ActionController, ActionMap};
use clap::Parser;
use events::libinput::Interface;
use events::main_loop;
use input::Libinput;
use log::{error, info};
use settings::{setup_application, Settings};
use strum::{Display, EnumString, EnumVariantNames, VariantNames};
use strum_macros::EnumIter;

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
#[derive(Parser, Debug, Clone)]
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
    four_finger_swipe_up: Option<Vec<String>>,
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
    let settings: Settings = setup_application(opts, true);

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
    use crate::settings::{setup_application, Settings};
    use crate::test_utils::default_test_settings;
    use crate::{ActionEvents, ActionTypes, Opts};
    use clap::Parser;
    use std::env;
    use std::fs::{create_dir, File};
    use std::io::Write;
    use tempfile::Builder;

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
        assert_eq!(
            opts.three_finger_swipe_left.unwrap(),
            vec![String::from("i3:foo")]
        );
    }

    #[test]
    #[should_panic(expected = "InvalidValue")]
    /// Test passing an invalid enabled action type as a parameter.
    fn test_enabled_action_types_argument_invalid() {
        Opts::try_parse_from(&["lillinput", "--enabled-action-types", "invalid"]).unwrap();
    }

    #[test]
    /// Test conversion of `Opts` to `Settings`.
    fn test_opts_to_settings() {
        let opts: Opts = Opts::parse_from(&[
            "lillinput",
            "--config-file",
            "nonexisting.file",
            "--seat",
            "some.seat",
            "--verbose",
            "--verbose",
            "--enabled-action-types",
            "i3",
            "--threshold",
            "20",
            "--three-finger-swipe-left",
            "command:bar",
            "--three-finger-swipe-left",
            "i3:3left",
            "--three-finger-swipe-right",
            "i3:3right",
            "--three-finger-swipe-up",
            "i3:3up",
            "--three-finger-swipe-down",
            "i3:3down",
            "--four-finger-swipe-left",
            "i3:4left",
            "--four-finger-swipe-right",
            "i3:4right",
            "--four-finger-swipe-up",
            "i3:4up",
            "--four-finger-swipe-down",
            "i3:4down",
        ]);
        let converted_settings: Settings = setup_application(opts, false);

        // Build expected settings:
        // * config file should be not passed and have no effect on settings.
        // * the "command:bar" action should be removed, as "command" is not enabled.
        // * actions should use the enum representations, and contain the passed values.
        let mut expected_settings = default_test_settings();
        expected_settings.verbose = 2;
        expected_settings.seat = String::from("some.seat");
        expected_settings.enabled_action_types = vec![ActionTypes::I3.to_string()];
        expected_settings.threshold = 20.0;
        expected_settings.actions.insert(
            ActionEvents::ThreeFingerSwipeLeft.to_string(),
            vec![String::from("i3:3left")],
        );
        expected_settings.actions.insert(
            ActionEvents::ThreeFingerSwipeRight.to_string(),
            vec![String::from("i3:3right")],
        );
        expected_settings.actions.insert(
            ActionEvents::ThreeFingerSwipeUp.to_string(),
            vec![String::from("i3:3up")],
        );
        expected_settings.actions.insert(
            ActionEvents::ThreeFingerSwipeDown.to_string(),
            vec![String::from("i3:3down")],
        );
        expected_settings.actions.insert(
            ActionEvents::FourFingerSwipeLeft.to_string(),
            vec![String::from("i3:4left")],
        );
        expected_settings.actions.insert(
            ActionEvents::FourFingerSwipeRight.to_string(),
            vec![String::from("i3:4right")],
        );
        expected_settings.actions.insert(
            ActionEvents::FourFingerSwipeUp.to_string(),
            vec![String::from("i3:4up")],
        );
        expected_settings.actions.insert(
            ActionEvents::FourFingerSwipeDown.to_string(),
            vec![String::from("i3:4down")],
        );

        assert_eq!(converted_settings, expected_settings);
    }

    #[test]
    /// Test using a config file.
    fn test_config_file() {
        let mut file = Builder::new().suffix(".toml").tempfile().unwrap();
        let file_path = String::from(file.path().to_str().unwrap());

        writeln!(
            file,
            r#"
verbose = 0
seat = "some.seat"
threshold = 42.0
enabled_action_types = ["i3"]

[actions]
three-finger-swipe-right = ["i3:foo"]
three-finger-swipe-left = []
three-finger-swipe-up = []
three-finger-swipe-down = []
four-finger-swipe-right = ["i3:bar", "command:baz"]
four-finger-swipe-left = []
four-finger-swipe-up = []
four-finger-swipe-down = []
"#
        )
        .unwrap();

        let opts: Opts = Opts::parse_from(&["lillinput", "--config-file", &file_path]);
        let converted_settings: Settings = setup_application(opts, false);

        // Build expected settings:
        // * values should be read from the config file.
        // * the "command:bar" action should be removed, as "command" is not enabled.
        // * actions should use the enum representations, and contain the passed values.
        let mut expected_settings = default_test_settings();
        expected_settings.verbose = 0;
        expected_settings.seat = String::from("some.seat");
        expected_settings.enabled_action_types = vec![ActionTypes::I3.to_string()];
        expected_settings.threshold = 42.0;
        expected_settings.actions.insert(
            ActionEvents::ThreeFingerSwipeRight.to_string(),
            vec![String::from("i3:foo")],
        );
        expected_settings.actions.insert(
            ActionEvents::FourFingerSwipeRight.to_string(),
            vec![String::from("i3:bar")],
        );

        assert_eq!(converted_settings, expected_settings);
    }

    #[test]
    /// Test using a config file from the default set (at `XDG_CONFIG_HOME`).
    fn test_config_file_from_xdg_config_home() {
        // Create a temporary dir.
        let tmp_dir = Builder::new().prefix("lillinput-conf").tempdir().unwrap();

        // Create the config dir ("temp/lillinput"), and tweak the xdg env var.
        create_dir(tmp_dir.path().join("lillinput")).unwrap();
        env::set_var("XDG_CONFIG_HOME", tmp_dir.path());

        // Populate the config file.
        let config_home_file_path = tmp_dir.path().join("lillinput").join("lillinput.toml");
        let mut config_home_file = File::create(config_home_file_path).unwrap();
        writeln!(
            config_home_file,
            r#"
verbose = 0
seat = "some.seat"
threshold = 42.0
enabled_action_types = ["i3"]

[actions]
three-finger-swipe-right = ["i3:foo"]
three-finger-swipe-left = []
three-finger-swipe-up = []
three-finger-swipe-down = []
four-finger-swipe-right = ["i3:bar", "command:baz"]
four-finger-swipe-left = []
four-finger-swipe-up = []
four-finger-swipe-down = []
"#
        )
        .unwrap();

        let opts: Opts = Opts::parse_from(&["lillinput"]);
        let converted_settings: Settings = setup_application(opts, false);

        // Build expected settings:
        // * values should be read from the home config file.
        // * the "command:bar" action should be removed, as "command" is not enabled.
        // * actions should use the enum representations, and contain the passed values.
        let mut expected_settings = default_test_settings();
        expected_settings.verbose = 0;
        expected_settings.seat = String::from("some.seat");
        expected_settings.enabled_action_types = vec![ActionTypes::I3.to_string()];
        expected_settings.threshold = 42.0;
        expected_settings.actions.insert(
            ActionEvents::ThreeFingerSwipeRight.to_string(),
            vec![String::from("i3:foo")],
        );
        expected_settings.actions.insert(
            ActionEvents::FourFingerSwipeRight.to_string(),
            vec![String::from("i3:bar")],
        );

        assert_eq!(converted_settings, expected_settings);
    }

    #[test]
    /// Test overriding options from a config file with options from CLI.
    fn test_config_overriding() {
        let mut file = Builder::new().suffix(".toml").tempfile().unwrap();
        let file_path = String::from(file.path().to_str().unwrap());

        writeln!(
            file,
            r#"
seat = "seat.from.config"
threshold = 42.0

[actions]
three-finger-swipe-right = ["i3:right_from_config"]
three-finger-swipe-left = ["i3:left_from_config"]
"#
        )
        .unwrap();

        let opts: Opts = Opts::parse_from(&[
            "lillinput",
            "--config-file",
            &file_path,
            "--threshold",
            "99.9",
            "--three-finger-swipe-left",
            "i3:left_from_cli",
        ]);
        let converted_settings: Settings = setup_application(opts, false);

        // Build expected settings:
        // * values should be merged from:
        //   1. default values.
        //   2. custom config file.
        //   3. cli arguments.
        let mut expected_settings = Settings {
            // `seat` from config file.
            seat: String::from("seat.from.config"),
            // `threshold` from CLI.
            threshold: 99.9,
            ..Default::default()
        };

        // `three-finger-swipe-right` from config file.
        expected_settings.actions.insert(
            ActionEvents::ThreeFingerSwipeRight.to_string(),
            vec![String::from("i3:right_from_config")],
        );
        // `three-finger-swipe-left` from CLI.
        expected_settings.actions.insert(
            ActionEvents::ThreeFingerSwipeLeft.to_string(),
            vec![String::from("i3:left_from_cli")],
        );

        assert_eq!(converted_settings, expected_settings);
    }
}
