use std::io::{self, Read};

use crate::command::Command;

/// Chip operating modes which can be entered after a reset
pub enum OperatingMode {
    /// Executes main user code
    SingleChip,
    /// Executes the in-ROM boot program, which provides the boot mode interface
    Boot,
    /// Executes the user bootloader
    UserBoot,
}

/// Functionality required to communicate with a target device. `io::Read` and
/// `io::Write` traits should expose the underlying serial connection.
pub trait Target: io::Read + io::Write {
    /// Clears both read and write buffers of the underlying serial port
    fn clear_buffers(&mut self) -> io::Result<()>;

    /// Sets the baud rate of the underlying serial port
    fn set_baud_rate(&mut self, baud_rate: u32) -> io::Result<()>;

    /// Returns the number of bytes available to be read from the underlying
    /// serial port
    fn bytes_to_read(&mut self) -> io::Result<u32>;

    /// Resets the target into the specified operating mode. Implementation
    /// unrestricted: can do anything from automatically resetting the target
    /// through the debug adapter, to asking the user to do it manually.
    fn reset_into(&mut self, operating_mode: OperatingMode);
}

/// Implements target communication with the `serialport` crate. Prompts the
/// user to perform manual resets.
pub struct SerialTarget {
    p: Box<dyn serialport::SerialPort>,
}

impl SerialTarget {
    /// Creates a new target from the specified serial port
    pub fn new(p: Box<dyn serialport::SerialPort>) -> SerialTarget {
        SerialTarget { p }
    }
}

impl Target for SerialTarget {
    fn clear_buffers(&mut self) -> io::Result<()> {
        Ok(self.p.clear(serialport::ClearBuffer::All)?)
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> io::Result<()> {
        Ok(self.p.set_baud_rate(baud_rate)?)
    }

    fn bytes_to_read(&mut self) -> io::Result<u32> {
        Ok(self.p.bytes_to_read()?)
    }

    fn reset_into(&mut self, operating_mode: OperatingMode) {
        let operating_mode_str = match operating_mode {
            OperatingMode::SingleChip => "single-chip",
            OperatingMode::Boot => "boot",
            OperatingMode::UserBoot => "user boot",
        };

        println!("The selected debug adapter does not support automatic reset. Please reset the target into {} mode and press ENTER.", operating_mode_str);

        io::stdin().read_exact(&mut [0u8]).unwrap();

        println!("Continuing...");
    }
}

impl io::Read for SerialTarget {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.p.read(buf)
    }
}

impl io::Write for SerialTarget {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.p.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.p.flush()
    }
}
