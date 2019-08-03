mod command;

/// Boot mode commands
pub mod commands;
/// Data types used by commands
pub mod data;
mod reader;

#[cfg(test)]
mod test_util;

pub use command::Command;

/// Prelude module providing basic data types required to implement a command.
/// Intended to be glob imported.
mod command_impl_prelude {
    pub use std::convert::Infallible;
    pub use std::io;

    pub use super::command::*;
    pub use super::data::*;
    pub use super::reader::*;
}
