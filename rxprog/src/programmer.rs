use std::io;
use std::ops::RangeInclusive;
use std::thread;
use std::time;

use crate::command::{self, Command};

/// Error encountered when attempting to make an initial connection to a device
#[derive(Debug)]
pub enum ConnectError {
    /// The device did not respond
    NoResponse,
    /// The device responded with an unknown response
    BadResponse,
    /// The device responded with a failure code
    Failed,
}

/// A programmer connected to a device, through a serial port
pub struct Programmer {
    p: Box<dyn serialport::SerialPort>,
}

impl Programmer {
    /// Creates a new programmer connected to the provided serial port
    pub fn new(p: Box<dyn serialport::SerialPort>) -> Programmer {
        Programmer { p: p }
    }

    /// Attempts to make an initial connection to the device
    pub fn connect(mut self) -> io::Result<Result<ProgrammerConnected, ConnectError>> {
        self.p.clear(serialport::ClearBuffer::All)?;

        for baud_rate in &[9600, 4800, 2400, 1200, 0] {
            if *baud_rate == 0 {
                return Ok(Err(ConnectError::NoResponse));
            }

            self.p.set_baud_rate(*baud_rate)?;

            let mut attempts = 0;
            while self.p.bytes_to_read()? < 1 && attempts < 30 {
                self.p.write(&[0x00])?;
                thread::sleep(time::Duration::from_millis(10));

                attempts += 1;
            }

            if self.p.bytes_to_read()? >= 1 {
                break;
            }
        }

        let mut response1 = [0u8; 1];
        self.p.read_exact(&mut response1)?;
        let response1 = response1[0];

        if response1 != 0x00 {
            return Ok(Err(ConnectError::BadResponse));
        }

        self.p.write(&[0x55])?;

        let mut response2 = [0u8; 1];
        self.p.read_exact(&mut response2)?;
        let response2 = response2[0];

        Ok(match response2 {
            0xE6 => Ok(ProgrammerConnected(self)),
            0xFF => Err(ConnectError::Failed),
            _ => Err(ConnectError::BadResponse),
        })
    }
}

/// A programmer connected to a device
pub struct ProgrammerConnected(Programmer);

impl ProgrammerConnected {
    fn execute<T: command::Command>(
        &mut self,
        cmd: &T,
    ) -> io::Result<Result<T::Response, T::Error>> {
        cmd.execute(&mut self.0.p)
    }

    /// Retrieve a list of devices supported by the target
    pub fn supported_devices(&mut self) -> io::Result<Vec<command::data::SupportedDevice>> {
        let cmd = command::commands::SupportedDeviceInquiry {};
        let devices = self.execute(&cmd)?.unwrap();

        Ok(devices)
    }

    /// Selects a device
    pub fn select_device(
        mut self,
        device_code: &String,
    ) -> io::Result<
        Result<ProgrammerConnectedDeviceSelected, command::commands::DeviceSelectionError>,
    > {
        let cmd = command::commands::DeviceSelection {
            device_code: device_code.clone(),
        };
        let response = self.execute(&cmd)?;

        Ok(match response {
            Ok(()) => Ok(ProgrammerConnectedDeviceSelected(self)),
            Err(x) => Err(x),
        })
    }
}

/// A programmer connected to a device, with a device selected
pub struct ProgrammerConnectedDeviceSelected(ProgrammerConnected);

impl ProgrammerConnectedDeviceSelected {
    /// Retrieve a list of supported clock modes
    pub fn clock_modes(&mut self) -> io::Result<Vec<u8>> {
        let cmd = command::commands::ClockModeInquiry {};
        let clock_modes = self.0.execute(&cmd)?.unwrap();

        Ok(clock_modes)
    }

    /// Selects a clock mode
    pub fn select_clock_mode(
        mut self,
        clock_mode: u8,
    ) -> io::Result<
        Result<ProgrammerConnectedClockModeSelected, command::commands::ClockModeSelectionError>,
    > {
        let cmd = command::commands::ClockModeSelection { mode: clock_mode };
        let response = self.0.execute(&cmd)?;

        Ok(match response {
            Ok(()) => Ok(ProgrammerConnectedClockModeSelected(self.0)),
            Err(x) => Err(x),
        })
    }
}

/// A programmer connected to a device, with a clock mode selected
pub struct ProgrammerConnectedClockModeSelected(ProgrammerConnected);

impl ProgrammerConnectedClockModeSelected {
    /// Retrieve a list of multiplication ratios supported by each clock
    pub fn multiplication_ratios(
        &mut self,
    ) -> io::Result<Vec<Vec<command::data::MultiplicationRatio>>> {
        let cmd = command::commands::MultiplicationRatioInquiry {};
        let multiplication_ratios = self.0.execute(&cmd)?.unwrap();

        Ok(multiplication_ratios)
    }

    /// Retrive the operating frequency range of each clock
    pub fn operating_frequencies(&mut self) -> io::Result<Vec<RangeInclusive<u16>>> {
        let cmd = command::commands::OperatingFrequencyInquiry {};
        let operating_frequencies = self.0.execute(&cmd)?.unwrap();

        Ok(operating_frequencies)
    }

    /// Sets a new bit rate for the device connection
    pub fn set_new_bit_rate(
        mut self,
        bit_rate: u16,
        input_frequency: u16,
        multiplication_ratios: Vec<command::data::MultiplicationRatio>,
    ) -> io::Result<
        Result<ProgrammerConnectedNewBitRateSelected, command::commands::NewBitRateSelectionError>,
    > {
        let cmd = command::commands::NewBitRateSelection {
            bit_rate: bit_rate,
            input_frequency: input_frequency,
            multiplication_ratios: multiplication_ratios,
        };
        let response = self.0.execute(&cmd)?;

        Ok(match response {
            Ok(()) => {
                let baud_rate: u32 = (bit_rate * 100).into();
                (self.0).0.p.set_baud_rate(baud_rate)?;

                let cmd = command::commands::NewBitRateSelectionConfirmation {};
                let _response = self.0.execute(&cmd)?;

                Ok(ProgrammerConnectedNewBitRateSelected(self.0))
            }
            Err(x) => Err(x),
        })
    }
}

/// A programmer connected to a device, after a new bit rate has been selected
pub struct ProgrammerConnectedNewBitRateSelected(ProgrammerConnected);
