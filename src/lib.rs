mod action;
mod command;
mod config;
mod keybinding;
mod keymatcher;
mod legend;
mod preview;

pub mod json;
pub mod terminal;

pub use action::Action;
pub use command::{ExternalCommand, ExternalCommandInput, ExternalCommandOutput, ShellCommand};
pub use config::Config;
pub use keybinding::{Keybinding, Keymap};
pub use keymatcher::display_key;
pub use legend::render_legend;
pub use preview::{FilePreview, FilePreviewPaneSpec, FilePreviewSpec};

fn io_error(cause: std::io::Error, message: &str) -> std::io::Error {
    std::io::Error::new(cause.kind(), format!("{message}: {cause}"))
}
