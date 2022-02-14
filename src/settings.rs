//! Functionality related to settings and other tooling.

use crate::{ActionEvents, ActionTypes, Opts};

use config::{Config, File};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, Config as LogConfig, Level, LevelFilter, TermLogger, TerminalMode};

use std::collections::HashMap;

/// Application settings.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Settings {
    /// Level of verbosity.
    pub verbose: i64,
    /// libinput seat.
    pub seat: String,
    /// Enabled action types.
    pub enabled_action_types: Vec<String>,
    /// Minimum threshold for displacement changes.
    pub threshold: f64,
    /// List of action for each action event.
    pub actions: HashMap<String, Vec<String>>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            verbose: 0,
            seat: "seat0".to_string(),
            enabled_action_types: vec![ActionTypes::I3.to_string()],
            threshold: 20.0,
            actions: HashMap::from([
                (
                    ActionEvents::ThreeFingerSwipeLeft.to_string(),
                    vec!["i3:workspace prev".to_string()],
                ),
                (
                    ActionEvents::ThreeFingerSwipeRight.to_string(),
                    vec!["i3:workspace next".to_string()],
                ),
                (ActionEvents::ThreeFingerSwipeUp.to_string(), vec![]),
                (ActionEvents::ThreeFingerSwipeDown.to_string(), vec![]),
                (ActionEvents::FourFingerSwipeLeft.to_string(), vec![]),
                (ActionEvents::FourFingerSwipeRight.to_string(), vec![]),
                (ActionEvents::FourFingerSwipeUp.to_string(), vec![]),
                (ActionEvents::FourFingerSwipeDown.to_string(), vec![]),
            ]),
        }
    }
}

// Log entries emitted during setup_application.
struct LogEntry {
    level: Level,
    message: String,
}

/// Initialize logging, setting the logger and the verbosity.
///
/// # Arguments
///
/// * `verbosity` - verbosity level.
fn setup_logging(verbosity: i64) {
    let log_level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    TermLogger::init(
        log_level,
        LogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}

/// Check if an action string is valid and with an enabled action type.
///
/// A string that specifies an action must conform to the following format:
/// {action choice}:{value}.
/// and {action choice} needs to be in enabled_action_types.
///
/// # Arguments
///
/// * `value` - argument to be parsed.
/// * `enabled_action_types` - slice of enabled action types.
fn is_enabled_action_string(action_string: &str, enabled_action_types: &[String]) -> bool {
    match action_string.split_once(':') {
        Some((action, _)) => enabled_action_types.iter().any(|s| s == action),
        None => false,
    }
}

/// Setup the application logging and return the application settings.
///
/// The application settings are merged from:
/// 1. Configuration file
/// 2. Command line arguments
///
/// # Arguments
///
/// * `opts` - command line arguments.
/// * `initialize_logging` - if `true`, initialize logging.
pub fn setup_application(opts: Opts, initialize_logging: bool) -> Settings {
    // Initialize the variables to keep track of config.
    let mut final_settings: Settings;
    let mut log_entries: Vec<LogEntry> = Vec::new();

    // Determine the config files to use: unless an specific file is provided
    // from the CLI option, use the default files:
    // * /etc
    // * XDG_CONFIG_HOME/lillinput
    // * cwd
    let mut config_home = xdg::BaseDirectories::with_prefix("lillinput")
        .unwrap()
        .get_config_home();
    config_home.push("lillinput.toml");
    let config_file = opts.config_file.clone();
    let files: Vec<String> = match config_file {
        Some(filename) => vec![filename],
        None => vec![
            "/etc/lillinput.toml".to_string(),
            config_home.into_os_string().into_string().unwrap(),
            "./lillinput.toml".to_string(),
        ],
    };

    // Prepare the default settings and options.
    let default_settings = Settings::default();
    let mut default_config = Config::default();

    default_config.set_default("verbose", 0).ok();
    default_config.set_default("seat", "seat0".to_string()).ok();
    default_config
        .set_default("enabled_action_types", vec![ActionTypes::I3.to_string()])
        .ok();
    default_config.set_default("threshold", 20.0).ok();
    let actions: HashMap<String, Vec<String>> = HashMap::from([
        (
            ActionEvents::ThreeFingerSwipeLeft.to_string(),
            vec!["i3:workspace prev".to_string()],
        ),
        (
            ActionEvents::ThreeFingerSwipeRight.to_string(),
            vec!["i3:workspace next".to_string()],
        ),
        (ActionEvents::ThreeFingerSwipeUp.to_string(), vec![]),
        (ActionEvents::ThreeFingerSwipeDown.to_string(), vec![]),
        (ActionEvents::FourFingerSwipeLeft.to_string(), vec![]),
        (ActionEvents::FourFingerSwipeRight.to_string(), vec![]),
        (ActionEvents::FourFingerSwipeUp.to_string(), vec![]),
        (ActionEvents::FourFingerSwipeDown.to_string(), vec![]),
    ]);
    default_config.set_default("actions", actions).ok();

    // Start a config with the default options.
    let mut config = Config::default();
    match config.merge(default_config) {
        Ok(_) => (),
        Err(e) => log_entries.push(LogEntry {
            level: Level::Warn,
            message: format!("Unable to parse default config: {}", e),
        }),
    }

    // Merge the config files.
    for filename in files {
        match Config::default().with_merged(File::with_name(&filename)) {
            Ok(c) => {
                log_entries.push(LogEntry {
                    level: Level::Info,
                    message: format!("Read config file '{}'", filename),
                });
                config = c
            }
            Err(e) => log_entries.push(LogEntry {
                level: Level::Warn,
                message: format!("Unable to parse config file '{}': {}", filename, e),
            }),
        };
    }

    // Add the CLI options.
    config.set("verbose", opts.verbose).ok();
    if opts.seat.is_some() {
        config.set("seat", opts.seat).ok();
    }
    if opts.enabled_action_types.is_some() {
        config
            .set("enabled_action_types", opts.enabled_action_types)
            .ok();
    }
    if opts.threshold.is_some() {
        config.set("threshold", opts.threshold).ok();
    }
    if let Some(values) = opts.three_finger_swipe_left {
        config
            .set(
                &format!("actions.{}", ActionEvents::ThreeFingerSwipeLeft),
                values,
            )
            .ok();
    }
    if let Some(values) = opts.three_finger_swipe_right {
        config
            .set(
                &format!("actions.{}", ActionEvents::ThreeFingerSwipeRight),
                values,
            )
            .ok();
    }
    if let Some(values) = opts.three_finger_swipe_up {
        config
            .set(
                &format!("actions.{}", ActionEvents::ThreeFingerSwipeUp),
                values,
            )
            .ok();
    }
    if let Some(values) = opts.three_finger_swipe_down {
        config
            .set(
                &format!("actions.{}", ActionEvents::ThreeFingerSwipeDown),
                values,
            )
            .ok();
    }
    if let Some(values) = opts.four_finger_swipe_left {
        config
            .set(
                &format!("actions.{}", ActionEvents::FourFingerSwipeLeft),
                values,
            )
            .ok();
    }
    if let Some(values) = opts.four_finger_swipe_right {
        config
            .set(
                &format!("actions.{}", ActionEvents::FourFingerSwipeRight),
                values,
            )
            .ok();
    }
    if let Some(values) = opts.four_finger_up_down {
        config
            .set(
                &format!("actions.{}", ActionEvents::FourFingerSwipeUp),
                values,
            )
            .ok();
    }
    if let Some(values) = opts.four_finger_swipe_down {
        config
            .set(
                &format!("actions.{}", ActionEvents::FourFingerSwipeDown),
                values,
            )
            .ok();
    }

    // Finalize the config, determining which Settings to use. In case of
    // errors, revert to the default settings.
    match config.try_into::<Settings>() {
        Ok(merged_settings) => final_settings = merged_settings,
        Err(e) => {
            log_entries.push(LogEntry {
                level: Level::Warn,
                message: format!(
                    "Unable to parse settings: {}. Reverting to default settings",
                    e
                ),
            });
            final_settings = default_settings
        }
    }

    // Prune action strings, removing the items that are malformed or using
    // not enaled action types.
    let enabled_action_types = final_settings.enabled_action_types.as_slice();
    for (key, value) in final_settings.actions.iter_mut() {
        let mut prune = false;
        // Check each action string, for debugging purposes.
        for entry in value.iter() {
            if !is_enabled_action_string(entry, enabled_action_types) {
                log_entries.push(LogEntry {
                    level: Level::Warn,
                    message: format!(
                        "Removing malformed or disabled action in {}: {}",
                        key, entry
                    ),
                });
                prune = true;
            }
        }

        if prune {
            value.retain(|x| is_enabled_action_string(x, enabled_action_types));
        }
    }

    // Setup logging.
    if initialize_logging {
        setup_logging(final_settings.verbose);
    }

    // Log any pending error messages.
    for log_entry in log_entries.iter() {
        match log_entry.level {
            Level::Info => info!("{}", log_entry.message),
            Level::Warn => warn!("{}", log_entry.message),
            _ => (),
        }
    }

    // Return the final settings.
    final_settings
}
