//! Functionality related to settings and other tooling.

use crate::{ActionTypes, Opts};

use config::{Config, ConfigError, File};
use log::warn;
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, Config as LogConfig, LevelFilter, TermLogger, TerminalMode};

/// Application settings.
#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    /// Level of verbosity.
    pub verbose: i64,
    /// libinput seat.
    pub seat: String,
    /// Enabled action types.
    pub enabled_action_types: Vec<String>,
    /// Minimum threshold for displacement changes.
    pub threshold: f64,
    /// Actions the three-finger swipe left.
    pub swipe_left_3: Vec<String>,
    /// Actions the three-finger swipe right.
    pub swipe_right_3: Vec<String>,
    /// Actions the three-finger swipe up.
    pub swipe_up_3: Vec<String>,
    /// Actions the three-finger swipe down.
    pub swipe_down_3: Vec<String>,
    /// Actions the four-finger swipe left.
    pub swipe_left_4: Vec<String>,
    /// Actions the four-finger swipe right.
    pub swipe_right_4: Vec<String>,
    /// Actions the four-finger swipe up.
    pub swipe_up_4: Vec<String>,
    /// Actions the four-finger swipe down.
    pub swipe_down_4: Vec<String>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            verbose: 0,
            seat: "seat0".to_string(),
            enabled_action_types: vec![ActionTypes::I3.to_string()],
            threshold: 20.0,
            swipe_left_3: vec!["i3:workspace prev".to_string()],
            swipe_right_3: vec!["i3:workspace next".to_string()],
            swipe_up_3: vec![],
            swipe_down_3: vec![],
            swipe_left_4: vec![],
            swipe_right_4: vec![],
            swipe_up_4: vec![],
            swipe_down_4: vec![],
        }
    }
}

/// Initialize logging, setting the logger and the verbosity.
///
/// # Arguments
///
/// * `verbosity` - verbosity level.
pub fn setup_logging(verbosity: i64) {
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

/// Setup the application logging and return the application settings.
///
/// The application settings are merged from:
/// 1. Configuration file
/// 2. Command line arguments
///
/// # Arguments
///
/// * `opts` - command line arguments.
pub fn setup_application(opts: Opts) -> Settings {
    // Determine the config files to use: unless an specific file is provided
    // from the CLI option, use the default files.
    let config_file = opts.config_file.clone();
    let files: Vec<String> = match config_file {
        Some(filename) => vec![filename],
        None => vec!["/etc/lillinput.toml".to_string()],
    };

    // Prepare the default settings and options.
    let default_settings = Settings::default();
    let mut default_config = Config::default();

    default_config.set_default("verbose", 0).ok();
    default_config.set_default("seat", "seat0".to_string()).ok();
    default_config.set_default("enabled_action_types", vec![ActionTypes::I3.to_string()]).ok();
    default_config
        .set_default("enabled_action_types", vec![ActionTypes::I3.to_string()])
        .ok();
    default_config.set_default("threshold", 20.0).ok();
    default_config.set_default("swipe_left_3", vec!["i3:workspace prev".to_string()]).ok();
    default_config.set_default("swipe_right_3", vec!["i3:workspace next".to_string()]).ok();
    default_config.set_default::<Vec<String>>("swipe_up_3", vec![]).ok();
    default_config.set_default::<Vec<String>>("swipe_down_3", vec![]).ok();
    default_config.set_default::<Vec<String>>("swipe_left_4", vec![]).ok();
    default_config.set_default::<Vec<String>>("swipe_right_4", vec![]).ok();
    default_config.set_default::<Vec<String>>("swipe_up_4", vec![]).ok();
    default_config.set_default::<Vec<String>>("swipe_down_4", vec![]).ok();
    default_config
        .set_default("swipe_left_3", vec!["i3:workspace prev".to_string()])
        .ok();
    default_config
        .set_default("swipe_right_3", vec!["i3:workspace next".to_string()])
        .ok();
    default_config
        .set_default::<Vec<String>>("swipe_up_3", vec![])
        .ok();
    default_config
        .set_default::<Vec<String>>("swipe_down_3", vec![])
        .ok();
    default_config
        .set_default::<Vec<String>>("swipe_left_4", vec![])
        .ok();
    default_config
        .set_default::<Vec<String>>("swipe_right_4", vec![])
        .ok();
    default_config
        .set_default::<Vec<String>>("swipe_up_4", vec![])
        .ok();
    default_config
        .set_default::<Vec<String>>("swipe_down_4", vec![])
        .ok();

    // Initialize the variables to keep track of config.
    let final_settings: Settings;
    let mut config_file_errors: Vec<ConfigError> = Vec::new();

    // Start a config with the default options.
    let mut config = Config::default();
    match config.merge(default_config) {
        Ok(_) => (),
        Err(e) => config_file_errors.push(e),
    }

    // Merge the config files.
    for filename in files {
        match Config::default().with_merged(File::with_name(&filename)) {
            Ok(c) => config = c,
            Err(e) => config_file_errors.push(e),
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
    if opts.swipe_left_3.is_some() {
        config.set("swipe_left_3", opts.swipe_left_3).ok();
    }
    if opts.swipe_right_3.is_some() {
        config.set("swipe_right_3", opts.swipe_right_3).ok();
    }
    if opts.swipe_up_3.is_some() {
        config.set("swipe_up_3", opts.swipe_up_3).ok();
    }
    if opts.swipe_down_3.is_some() {
        config.set("swipe_down_3", opts.swipe_down_3).ok();
    }
    if opts.swipe_left_4.is_some() {
        config.set("swipe_left_4", opts.swipe_left_4).ok();
    }
    if opts.swipe_right_4.is_some() {
        config.set("swipe_right_4", opts.swipe_right_4).ok();
    }
    if opts.swipe_up_4.is_some() {
        config.set("swipe_up_4", opts.swipe_up_4).ok();
    }
    if opts.swipe_down_4.is_some() {
        config.set("swipe_down_4", opts.swipe_down_4).ok();
    }

    // Finalize the config, determining which Settings to use. In case of
    // errors, revert to the default settings.
    match config.try_into::<Settings>() {
        Ok(merged_settings) => final_settings = merged_settings,
        Err(e) => {
            config_file_errors.push(e);
            final_settings = default_settings
        }
    }

    // Setup logging.
    setup_logging(final_settings.verbose);

    // Log any pending error messages.
    for e in config_file_errors.iter() {
        warn!("Unable to parse config file: {}", e);
    }

    // Return the final settings.
    final_settings
}
