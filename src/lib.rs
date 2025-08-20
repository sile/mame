mod action;
mod config;
mod json;
mod keybinding;
mod keymatcher;
mod terminal;
// TODO: mod legend;
// TODO: mod error_dialog or notification_area;
mod external_command;

pub use action::Action;
pub use config::Config;
pub use external_command::ExternalCommand;
pub use json::LoadJsonFileError;
pub use keybinding::{Keybinding, Keymap};
pub use terminal::{UnicodeCharWidthEstimator, UnicodeTerminalFrame, char_cols, str_cols};

// TODO: delete
pub use keymatcher::KeyMatcher;
