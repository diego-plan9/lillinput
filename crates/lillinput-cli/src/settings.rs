//! Functionality related to application settings and related tooling.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use crate::opts::{Opts, StringifiedAction};
use config::{Config, ConfigError, File, Map, Source, Value};
use i3ipc::I3Connection;
use lillinput::actions::commandaction::CommandAction;
use lillinput::actions::i3action::{I3Action, SharedConnection};
use lillinput::actions::{Action, ActionType};
use lillinput::events::ActionEvent;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, Config as LogConfig, Level, LevelFilter, TermLogger, TerminalMode};
use std::string::ToString;
use strum::IntoEnumIterator;

/// Application settings.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Settings {
    /// Level of verbosity.
    pub verbose: LevelFilter,
    /// `libinput` seat.
    pub seat: String,
    /// Enabled action types.
    pub enabled_action_types: Vec<String>,
    /// Minimum threshold for displacement changes.
    pub threshold: f64,
    /// List of action for each action event.
    pub actions: HashMap<String, Vec<StringifiedAction>>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            verbose: LevelFilter::Info,
            seat: "seat0".to_string(),
            enabled_action_types: vec![ActionType::I3.to_string()],
            threshold: 20.0,
            actions: HashMap::from([
                (
                    ActionEvent::ThreeFingerSwipeLeft.to_string(),
                    vec![StringifiedAction::new("i3", "workspace prev")],
                ),
                (
                    ActionEvent::ThreeFingerSwipeRight.to_string(),
                    vec![StringifiedAction::new("i3", "workspace next")],
                ),
                (ActionEvent::ThreeFingerSwipeUp.to_string(), vec![]),
                (ActionEvent::ThreeFingerSwipeDown.to_string(), vec![]),
                (ActionEvent::FourFingerSwipeLeft.to_string(), vec![]),
                (ActionEvent::FourFingerSwipeRight.to_string(), vec![]),
                (ActionEvent::FourFingerSwipeUp.to_string(), vec![]),
                (ActionEvent::FourFingerSwipeDown.to_string(), vec![]),
            ]),
        }
    }
}

/// Log entries emitted during [`setup_application()`].
struct LogEntry {
    /// Log level for the entry.
    level: Level,
    /// Message of the entry.
    message: String,
}

/// Initialize logging, setting the logger and the verbosity.
///
/// # Arguments
///
/// * `verbosity` - verbosity level.
fn setup_logging(verbosity: LevelFilter) {
    TermLogger::init(
        verbosity,
        LogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
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
    let files = match opts.config_file.clone() {
        Some(filename) => vec![File::with_name(&filename).required(false)],
        None => vec![
            File::with_name("/etc/lillinput.toml").required(false),
            File::with_name(&config_home.into_os_string().into_string().unwrap()).required(false),
            File::with_name("./lillinput.toml").required(false),
        ],
    };

    // Special handling of the "verbose" flag. If no command line arguments
    // related to verbosity are passed, and the verbosity is specified in the
    // config files, use the config files value.
    let default_settings = Settings::default();
    let verbosity_override: Option<String> =
        if opts.verbose.log_level_filter() == default_settings.verbose {
            match Config::builder().add_source(files.clone()).build() {
                Ok(config) => config.get_string("verbose").ok(),
                Err(_) => None,
            }
        } else {
            None
        };

    // Parse the settings, defaulting in case of errors.
    let mut final_settings: Settings = match Config::builder()
        .add_source(Settings::default())
        .add_source(files)
        .add_source(opts)
        .set_override_option(String::from("verbose"), verbosity_override)
        .unwrap()
        .build()
    {
        Ok(merged_config) => match merged_config.try_deserialize::<Settings>() {
            Ok(merged_settings) => merged_settings,
            Err(e) => {
                log_entries.push(LogEntry {
                    level: Level::Warn,
                    message: format!(
                        "Unable to parse settings: {}. Reverting to default settings",
                        e
                    ),
                });
                Settings::default()
            }
        },
        Err(e) => {
            log_entries.push(LogEntry {
                level: Level::Warn,
                message: format!(
                    "Unable to parse settings: {}. Reverting to default settings",
                    e
                ),
            });
            Settings::default()
        }
    };

    // Prune action strings, removing the items that are malformed or using
    // not enabled action types.
    let enabled_action_types = final_settings.enabled_action_types.as_slice();
    for (key, value) in &mut final_settings.actions {
        let mut prune = false;
        // Check each action string, for debugging purposes.
        for entry in value.iter() {
            if !enabled_action_types.contains(&entry.type_) {
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
            value.retain(|x| enabled_action_types.contains(&x.type_));
        }
    }

    // Setup logging.
    if initialize_logging {
        setup_logging(final_settings.verbose);
    }

    // Log any pending error messages.
    for log_entry in &log_entries {
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

        m.insert(
            String::from("verbose"),
            Value::from(self.verbose.log_level_filter().to_string()),
        );
        self.seat
            .as_ref()
            .map(|x| m.insert(String::from("seat"), Value::from(x.clone())));
        self.enabled_action_types
            .as_ref()
            .map(|x| m.insert(String::from("enabled_action_types"), Value::from(x.clone())));
        self.threshold
            .as_ref()
            .map(|x| m.insert(String::from("threshold"), Value::from(*x)));

        for action_event in ActionEvent::iter() {
            let actions = self.get_actions_for_event(action_event);
            actions.map(|x| {
                m.insert(
                    String::from(&format!("actions.{}", action_event)),
                    Value::from(x.iter().map(ToString::to_string).collect::<Vec<String>>()),
                )
            });
        }

        Ok(m)
    }
}

impl Source for Settings {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let mut m = Map::new();

        m.insert(
            String::from("verbose"),
            Value::from(self.verbose.to_string()),
        );
        m.insert(String::from("seat"), Value::from(self.seat.clone()));
        m.insert(
            String::from("enabled_action_types"),
            Value::from(self.enabled_action_types.clone()),
        );
        m.insert(String::from("threshold"), Value::from(self.threshold));
        for (action_event, actions) in &self.actions {
            m.insert(
                String::from(&format!("actions.{}", action_event)),
                Value::from(
                    actions
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>(),
                ),
            );
        }

        Ok(m)
    }
}

/// Generate [`Action`]s from application settings.
///
/// # Arguments
///
/// * `settings` - application settings.
pub fn extract_action_map(
    settings: &Settings,
) -> (HashMap<ActionEvent, Vec<Box<dyn Action>>>, SharedConnection) {
    let mut action_map: HashMap<ActionEvent, Vec<Box<dyn Action>>> = HashMap::new();
    let connection = Rc::new(RefCell::new(None));
    let mut connection_exists = false;

    // Create the I3 connection if needed.
    if settings
        .actions
        .values()
        .flatten()
        .any(|s| s.type_ == ActionType::I3.to_string())
    {
        let new_connection = match I3Connection::connect() {
            Ok(mut conn) => {
                info!(
                    "i3: connection opened (with({:?})",
                    conn.get_version().unwrap().human_readable
                );
                connection_exists = true;

                Some(conn)
            }
            Err(error) => {
                warn!("i3: could not establish a connection: {:?}", error);
                None
            }
        };

        // Update the connection.
        let connection_option = &mut *connection.borrow_mut();
        *connection_option = new_connection;
    }

    // Populate the fields for each `ActionEvent`.
    for action_event in ActionEvent::iter() {
        if let Some(arguments) = settings.actions.get(&action_event.to_string()) {
            let mut actions_list: Vec<Box<dyn Action>> = vec![];

            for value in arguments.iter() {
                // Create the new actions.
                match ActionType::from_str(&value.type_) {
                    Ok(ActionType::Command) => {
                        actions_list.push(Box::new(CommandAction::new(value.command.clone())));
                    }
                    Ok(ActionType::I3) => {
                        if connection_exists {
                            actions_list.push(Box::new(I3Action::new(
                                value.command.clone(),
                                Rc::clone(&connection),
                            )));
                        } else {
                            warn!("Disabling action as i3 connection could not be established: {value}");
                        }
                    }
                    Err(_) => {
                        warn!("Unknown action type: '{}", value.type_);
                    }
                }
            }

            action_map.insert(action_event, actions_list);
        }
    }

    (action_map, connection)
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;
    use crate::test_utils::default_test_settings;
    use lillinput::controllers::defaultcontroller::DefaultController;

    use serial_test::serial;

    #[test]
    #[serial]
    ///Test graceful handling of unavailable i3 connection.
    fn test_i3_not_available() {
        // Initialize the command line options.
        let mut settings: Settings = default_test_settings();
        settings.enabled_action_types = vec!["i3".to_string()];
        settings.actions.insert(
            ActionEvent::ThreeFingerSwipeRight.to_string(),
            vec![
                StringifiedAction::new("i3", "swipe right"),
                StringifiedAction::new("command", "touch /tmp/swipe-right"),
            ],
        );

        // Create the controller.
        env::set_var("I3SOCK", "/tmp/non-existing-socket");
        let (actions, _) = extract_action_map(&settings);
        let action_map: DefaultController = DefaultController::new(settings.threshold, actions);

        // Assert that only the command action is created.
        assert_eq!(
            action_map
                .actions
                .get(&ActionEvent::ThreeFingerSwipeRight)
                .unwrap()
                .len(),
            1
        );
    }
}
