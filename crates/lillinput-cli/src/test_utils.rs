#[cfg(test)]
use std::collections::HashMap;

use crate::settings::Settings;
use lillinput::events::ActionEvent;
use simplelog::LevelFilter;

/// Return an `Settings` with default test arguments.
pub fn default_test_settings() -> Settings {
    Settings {
        enabled_action_types: vec![],
        actions: HashMap::from([
            (ActionEvent::ThreeFingerSwipeLeft.to_string(), vec![]),
            (ActionEvent::ThreeFingerSwipeLeftUp.to_string(), vec![]),
            (ActionEvent::ThreeFingerSwipeUp.to_string(), vec![]),
            (ActionEvent::ThreeFingerSwipeRightUp.to_string(), vec![]),
            (ActionEvent::ThreeFingerSwipeRight.to_string(), vec![]),
            (ActionEvent::ThreeFingerSwipeRightDown.to_string(), vec![]),
            (ActionEvent::ThreeFingerSwipeDown.to_string(), vec![]),
            (ActionEvent::ThreeFingerSwipeLeftDown.to_string(), vec![]),
            (ActionEvent::FourFingerSwipeLeft.to_string(), vec![]),
            (ActionEvent::FourFingerSwipeLeftUp.to_string(), vec![]),
            (ActionEvent::FourFingerSwipeUp.to_string(), vec![]),
            (ActionEvent::FourFingerSwipeRightUp.to_string(), vec![]),
            (ActionEvent::FourFingerSwipeRight.to_string(), vec![]),
            (ActionEvent::FourFingerSwipeRightDown.to_string(), vec![]),
            (ActionEvent::FourFingerSwipeDown.to_string(), vec![]),
            (ActionEvent::FourFingerSwipeLeftDown.to_string(), vec![]),
        ]),
        threshold: 5.0,
        seat: "seat0".to_string(),
        verbose: LevelFilter::Info,
    }
}
