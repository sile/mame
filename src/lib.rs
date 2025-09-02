//! This crate provides common building blocks to build TUI (Terminal User Interface) applications.
#![warn(missing_docs)]

mod binding;
mod matcher;

pub mod action;
pub mod command;
pub mod fmt;
pub mod json;
pub mod legend;
pub mod preview;
pub mod terminal;

fn io_error(cause: std::io::Error, message: &str) -> std::io::Error {
    std::io::Error::new(cause.kind(), format!("{message}: {cause}"))
}
