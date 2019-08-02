//! # rxprog
//!
//! `rxprog` is a library for communicating with and programming devices such as the RX210 from
//! Renesas, and other devices which implement the "Boot Mode" protocol.
#![deny(missing_docs)]

extern crate serialport;

/// Commands, and command execution
pub mod command;

/// Connection to a target device
pub mod target;

/// Interface wrapping a serial port to program a device
pub mod programmer;
