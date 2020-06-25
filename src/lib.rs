//! # rxprog
//!
//! `rxprog` is a library for communicating with and programming devices such as the RX210 from
//! Renesas, and other devices which implement the "Boot Mode" protocol.
#![deny(missing_docs)]

use std::error;
use std::fmt;
use std::io;
use std::result;

/// Commands, and command execution
pub mod command;

/// Connection to a target device
pub mod target;

/// Interface wrapping a serial port to program a device
pub mod programmer;

/// A type for results generated when communicating with/programming a target
/// device
pub type Result<T> = result::Result<T, Error>;

/// Categories of errors that can occur when communicating with/programming a
/// target device
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    /// A connection to the target could not be established
    Connect,
    /// An error was returned by a command executed on the target
    Command(command::CommandError),
    /// An I/O error occurred
    Io(io::ErrorKind),
}

/// An error type for communication/programming operations
#[derive(Debug, PartialEq)]
pub struct Error {
    /// The kind of error that occurred
    pub kind: ErrorKind,
    /// A human-readable description of the error
    pub description: String,
}

impl Error {
    fn new<T: Into<String>>(kind: ErrorKind, description: T) -> Error {
        let description = description.into();
        Error { kind, description }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Error {
        Error::new(ErrorKind::Io(io_error.kind()), io_error.to_string())
    }
}

impl From<command::CommandError> for Error {
    fn from(command_error: command::CommandError) -> Error {
        Error::new(ErrorKind::Command(command_error), command_error.to_string())
    }
}
