//! Functionality related to settings and other tooling.

use std::collections::HashMap;

use crate::{ActionEvents, ActionTypes, Opts};
use config::{Config, ConfigError, File, Map, Source, Value};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, Config as LogConfig, Level, LevelFilter, TermLogger, TerminalMode};

/// Application settings.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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
    let files = match config_file {
        Some(filename) => vec![File::with_name(&filename).required(false)],
        None => vec![
            File::with_name(&"/etc/lillinput.toml".to_string()).required(false),
            File::with_name(&config_home.into_os_string().into_string().unwrap()).required(false),
            File::with_name(&"./lillinput.toml".to_string()).required(false),
        ],
    };

    // Prepare the default settings and options.
    let default_settings = Settings::default();
    let mut default_config = Config::default();
    match default_config.merge(default_settings.clone()) {
        Ok(_) => (),
        Err(e) => log_entries.push(LogEntry {
            level: Level::Warn,
            message: format!("Unable to parse default config: {}", e),
        }),
    }

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
    match config.merge(files) {
        Ok(c) => {
            log_entries.push(LogEntry {
                level: Level::Info,
                message: "Read config file".to_string(),
            });
            config = c.clone()
        }
        Err(e) => log_entries.push(LogEntry {
            level: Level::Warn,
            message: format!("Unable to parse config file: {}", e),
        }),
    }

    // Add the CLI options.
    match config.merge(opts) {
        Ok(_) => (),
        Err(e) => log_entries.push(LogEntry {
            level: Level::Warn,
            message: format!("Unable to parse default config: {}", e),
        }),
    }

    // Finalize the config, determining which Settings to use. In case of
    // errors, revert to the default settings.
    let mut final_settings: Settings = match config.try_deserialize::<Settings>() {
        Ok(merged_settings) => merged_settings,
        Err(e) => {
            log_entries.push(LogEntry {
                level: Level::Warn,
                message: format!(
                    "Unable to parse settings: {}. Reverting to default settings",
                    e
                ),
            });
            default_settings
        }
    };

    // Prune action strings, removing the items that are malformed or using
    // not enabled action types.
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

impl Source for Opts {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let mut m = Map::new();

        m.insert(String::from("verbose"), Value::from(self.verbose));
        self.seat
            .as_ref()
            .map(|x| m.insert(String::from("seat"), Value::from(x.clone())));
        self.enabled_action_types
            .as_ref()
            .map(|x| m.insert(String::from("enabled_action_types"), Value::from(x.clone())));
        self.threshold
            .as_ref()
            .map(|x| m.insert(String::from("threshold"), Value::from(*x)));
        self.three_finger_swipe_left.as_ref().map(|x| {
            m.insert(
                String::from(&format!("actions.{}", ActionEvents::ThreeFingerSwipeLeft)),
                Value::from(x.clone()),
            )
        });
        self.three_finger_swipe_right.as_ref().map(|x| {
            m.insert(
                String::from(&format!("actions.{}", ActionEvents::ThreeFingerSwipeRight)),
                Value::from(x.clone()),
            )
        });
        self.three_finger_swipe_up.as_ref().map(|x| {
            m.insert(
                String::from(&format!("actions.{}", ActionEvents::ThreeFingerSwipeUp)),
                Value::from(x.clone()),
            )
        });
        self.three_finger_swipe_down.as_ref().map(|x| {
            m.insert(
                String::from(&format!("actions.{}", ActionEvents::ThreeFingerSwipeDown)),
                Value::from(x.clone()),
            )
        });
        self.four_finger_swipe_left.as_ref().map(|x| {
            m.insert(
                String::from(&format!("actions.{}", ActionEvents::FourFingerSwipeLeft)),
                Value::from(x.clone()),
            )
        });
        self.four_finger_swipe_right.as_ref().map(|x| {
            m.insert(
                String::from(&format!("actions.{}", ActionEvents::FourFingerSwipeRight)),
                Value::from(x.clone()),
            )
        });
        self.four_finger_swipe_up.as_ref().map(|x| {
            m.insert(
                String::from(&format!("actions.{}", ActionEvents::FourFingerSwipeUp)),
                Value::from(x.clone()),
            )
        });
        self.four_finger_swipe_down.as_ref().map(|x| {
            m.insert(
                String::from(&format!("actions.{}", ActionEvents::FourFingerSwipeDown)),
                Value::from(x.clone()),
            )
        });

        Ok(m)
    }
}

impl Source for Settings {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let mut m = Map::new();

        m.insert(String::from("verbose"), Value::from(self.verbose));
        m.insert(String::from("seat"), Value::from(self.seat.clone()));
        m.insert(
            String::from("enabled_action_types"),
            Value::from(self.enabled_action_types.clone()),
        );
        m.insert(String::from("threshold"), Value::from(self.threshold));
        for (action_event, actions) in self.actions.iter() {
            m.insert(
                String::from(&format!("actions.{}", action_event)),
                Value::from(actions.clone()),
            );
        }

        Ok(m)
    }
}
