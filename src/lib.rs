//! This crate provides common building blocks to build TUI (Terminal User Interface) applications.
#![warn(missing_docs)]

mod keybinding;
mod keymatcher;

pub mod action;
pub mod command;
pub mod json;
pub mod legend;
pub mod preview;
pub mod terminal;
pub mod fmt;

fn io_error(cause: std::io::Error, message: &str) -> std::io::Error {
    std::io::Error::new(cause.kind(), format!("{message}: {cause}"))
}
