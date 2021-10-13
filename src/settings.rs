//! Functionality related to settings and other tooling.

use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

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
