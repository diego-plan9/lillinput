//! Utilities for the command line application.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use crate::Settings;
use i3ipc::I3Connection;
use lillinput::actions::commandaction::CommandAction;
use lillinput::actions::i3action::{I3Action, SharedConnection};
use lillinput::actions::{Action, ActionType};
use lillinput::events::ActionEvent;
use log::{info, warn};
use strum::IntoEnumIterator;

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

    use crate::actions::ActionMap;
    use crate::events::ActionEvent;
    use crate::extract_action_map;
    use crate::opts::StringifiedAction;
    use crate::settings::Settings;
    use crate::test_utils::default_test_settings;

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
        let action_map: ActionMap = ActionMap::new(settings.threshold, actions);

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
