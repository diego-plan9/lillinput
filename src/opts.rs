//! Arguments and utils for the `lillinput` binary.

use crate::ActionTypes;
use clap::builder::{StringValueParser, TypedValueParser};
use clap::error::ErrorKind;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use strum::VariantNames;

/// Connect libinput gestures to i3 and others.
#[derive(Parser, Debug, Clone)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
pub struct Opts {
    /// Configuration file.
    #[clap(short, long)]
    pub config_file: Option<String>,
    /// Level of verbosity (additive, can be used up to 3 times)
    #[clap(flatten)]
    pub verbose: Verbosity<InfoLevel>,
    /// libinput seat
    #[clap(short, long)]
    pub seat: Option<String>,
    /// enabled action types
    #[clap(short, long, possible_values = ActionTypes::VARIANTS)]
    pub enabled_action_types: Option<Vec<String>>,
    /// minimum threshold for displacement changes
    #[clap(short, long)]
    pub threshold: Option<f64>,
    /// actions the three-finger swipe left
    #[clap(long, value_parser = ActionStringParser)]
    pub three_finger_swipe_left: Option<Vec<String>>,
    /// actions the three-finger swipe right
    #[clap(long, value_parser = ActionStringParser)]
    pub three_finger_swipe_right: Option<Vec<String>>,
    /// actions the three-finger swipe up
    #[clap(long, value_parser = ActionStringParser)]
    pub three_finger_swipe_up: Option<Vec<String>>,
    /// actions the three-finger swipe down
    #[clap(long, value_parser = ActionStringParser)]
    pub three_finger_swipe_down: Option<Vec<String>>,
    /// actions the four-finger swipe left
    #[clap(long, value_parser = ActionStringParser)]
    pub four_finger_swipe_left: Option<Vec<String>>,
    /// actions the four-finger swipe right
    #[clap(long, value_parser = ActionStringParser)]
    pub four_finger_swipe_right: Option<Vec<String>>,
    /// actions the four-finger swipe up
    #[clap(long, value_parser = ActionStringParser)]
    pub four_finger_swipe_up: Option<Vec<String>>,
    /// actions the four-finger swipe down
    #[clap(long, value_parser = ActionStringParser)]
    pub four_finger_swipe_down: Option<Vec<String>>,
}

/// Parser for arguments that specify an action.
///
/// A string that specifies an action must conform to the following format:
/// * `{action choice}:{value}`.
#[derive(Clone, Debug)]
struct ActionStringParser;

impl TypedValueParser for ActionStringParser {
    type Value = String;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let inner = StringValueParser::new();
        let value = inner.parse_ref(cmd, arg, value)?;

        match value.split_once(':') {
            None | Some((_, "") | ("", _)) => Err(clap::Error::raw(
                ErrorKind::ValueValidation,
                "The value does not conform to the action string pattern `{type}:{command}`",
            )),
            Some((action_type, _)) => {
                if ActionTypes::VARIANTS.iter().any(|s| s == &action_type) {
                    Ok(value)
                } else {
                    Err(clap::Error::raw(
                        ErrorKind::ValueValidation,
                        format!(
                            "The value does not start with a valid action ({:?})",
                            ActionTypes::VARIANTS
                        ),
                    ))
                }
            }
        }
    }
}
