#[cfg(test)]
use std::collections::HashMap;

use crate::settings::Settings;
use simplelog::LevelFilter;

/// Return an `Settings` with default test arguments.
pub fn default_test_settings() -> Settings {
    Settings {
        enabled_action_types: vec![],
        actions: HashMap::new(),
        threshold: 5.0,
        seat: "seat0".to_string(),
        verbose: LevelFilter::Info,
    }
}
