mod action;
mod config;
mod keybinding;
mod keymatcher;
mod legend;
mod preview;

pub mod command;
pub mod json;
pub mod terminal;

pub mod fmt {
    pub fn display_key(key: tuinix::KeyInput) -> impl std::fmt::Display {
        crate::keymatcher::KeyMatcher::Literal(key)
    }
}

pub use action::Action;
pub use config::Config;
pub use keybinding::{Keybinding, Keymap};
pub use legend::render_legend;
pub use preview::{FilePreview, FilePreviewPaneSpec, FilePreviewSpec};

fn io_error(cause: std::io::Error, message: &str) -> std::io::Error {
    std::io::Error::new(cause.kind(), format!("{message}: {cause}"))
}
