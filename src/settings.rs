//! Functionality related to settings and other tooling.

use crate::{ActionTypes, Opts};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

/// Application settings.
pub struct Settings {
    /// Level of verbosity.
    pub verbose: u8,
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
            swipe_down_4: vec![]
        }
    }
}

impl From<Opts> for Settings {
    fn from(opts: Opts) -> Self {
        Settings {
            verbose: opts.verbose,
            seat: opts.seat.unwrap_or(Settings::default().seat),
            enabled_action_types: opts.enabled_action_types.unwrap_or(Settings::default().enabled_action_types),
            threshold: opts.threshold.unwrap_or(Settings::default().threshold),
            swipe_left_3: opts.swipe_left_3.unwrap_or(Settings::default().swipe_left_3),
            swipe_right_3: opts.swipe_right_3.unwrap_or(Settings::default().swipe_right_3),
            swipe_up_3: opts.swipe_up_3.unwrap_or(Settings::default().swipe_up_3),
            swipe_down_3: opts.swipe_down_3.unwrap_or(Settings::default().swipe_down_3),
            swipe_left_4: opts.swipe_left_4.unwrap_or(Settings::default().swipe_left_4),
            swipe_right_4: opts.swipe_right_4.unwrap_or(Settings::default().swipe_right_4),
            swipe_up_4: opts.swipe_up_4.unwrap_or(Settings::default().swipe_up_4),
            swipe_down_4: opts.swipe_down_4.unwrap_or(Settings::default().swipe_down_4),
        }
    }
}

/// Initialize logging, setting the logger and the verbosity.
///
/// # Arguments
///
/// * `verbosity` - verbosity level.
pub fn setup_logging(verbosity: u8) {
    let log_level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    TermLogger::init(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
