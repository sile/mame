mod action;
mod config;
mod keybinding;
mod keymatcher;

pub mod command;
pub mod json;
pub mod legend;
pub mod preview;
pub mod terminal;

pub mod fmt {
    pub fn display_key(key: tuinix::KeyInput) -> impl std::fmt::Display {
        crate::keymatcher::KeyMatcher::Literal(key)
    }
}

pub use action::Action;
pub use config::Config;
pub use keybinding::{Keybinding, Keymap};

fn io_error(cause: std::io::Error, message: &str) -> std::io::Error {
    std::io::Error::new(cause.kind(), format!("{message}: {cause}"))
}
