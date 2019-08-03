use std::fmt;
use std::io;
use std::num::Wrapping;

use crate::Result;

/// A command which can be sent to a device, and results in either a response or error
pub trait Command {
    /// Result of a successful command execution
    type Response;

    /// Executes the command on a device
    fn execute<T: io::Read + io::Write>(&self, p: &mut T) -> Result<Self::Response>;
}

pub trait Transmit {
    fn tx<T: io::Write>(&self, p: &mut T) -> Result<()>;
}

pub struct CommandData {
    pub opcode: u8,
    pub has_size_field: bool,
    pub payload: Vec<u8>,
}

impl CommandData {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        let payload = &self.payload;
        let payload_size = payload.len();

        bytes.push(self.opcode);

        if self.has_size_field {
            bytes.push(payload_size as u8);
        }

        bytes.extend(payload);

        if payload_size != 0 {
            let sum = bytes.iter().map(|x| Wrapping(*x)).sum::<Wrapping<u8>>().0;
            let checksum = !sum + 1;
            bytes.push(checksum);
        }

        bytes
    }
}

pub trait TransmitCommandData {
    fn command_data(&self) -> CommandData;
}

impl<T: TransmitCommandData> Transmit for T {
    fn tx<U: io::Write>(&self, p: &mut U) -> Result<()> {
        p.write(&self.command_data().bytes())?;
        p.flush()?;

        Ok(())
    }
}

pub trait Receive {
    type Response;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response>;
}

impl<T: Transmit + Receive> Command for T {
    type Response = T::Response;

    fn execute<U: io::Read + io::Write>(&self, p: &mut U) -> Result<Self::Response> {
        self.tx(p)?;
        self.rx(p)
    }
}

/// An error returned by a target in response to a command
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CommandError {
    /// Invalid address or area
    Address,
    /// Requested bit rate could not be selected within an acceptable margin of
    /// error
    BitRateSelection,
    /// Invalid block number
    BlockNumber,
    /// Checksum mismatch
    Checksum,
    /// Invalid clock mode
    ClockMode,
    /// Invalid data size (zero, too large, or calculated end address out of
    /// bounds)
    DataSize,
    /// Invalid device code
    DeviceCode,
    /// Error occurred during erasure (target or user initiated)
    Erasure,
    /// Supplied ID code did not match
    IDCodeMismatch,
    /// Input frequency out of range for selected clock mode
    InputFrequency,
    /// Multiplication ratio invalid for selected clock mode
    MultiplicationRatio,
    /// Calculated operating frequency out of range for clock
    OperatingFrequency,
    /// Error occurred during programming
    Programming,
    /// Failed to transition into programming/erasure state
    ProgrammingErasureStateTransition,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CommandError::Address => "invalid address/area",
                CommandError::BitRateSelection => "bit rate selection error too high",
                CommandError::BlockNumber => "invalid block number",
                CommandError::Checksum => "checksum mismatch",
                CommandError::ClockMode => "invalid clock mode",
                CommandError::DataSize => "invalid data size",
                CommandError::DeviceCode => "invalid device code",
                CommandError::Erasure => "erasure error",
                CommandError::IDCodeMismatch => "ID code mismatch",
                CommandError::InputFrequency => "input frequency out of range",
                CommandError::MultiplicationRatio => "invalid multiplication ratio",
                CommandError::OperatingFrequency => "calculated operating frequency out of range",
                CommandError::Programming => "programming error",
                CommandError::ProgrammingErasureStateTransition => {
                    "failed to transition into programming/erasure state"
                }
            }
        )
    }
}
