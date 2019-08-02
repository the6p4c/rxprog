use std::io;
use std::ops::RangeInclusive;
use std::thread;
use std::time;

use crate::command::{self, Command};
use crate::target::{OperatingMode, Target};

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
    target: Box<dyn Target>,
}

impl Programmer {
    /// Creates a new programmer connected to the provided serial port
    pub fn new(target: Box<dyn Target>) -> Programmer {
        Programmer { target }
    }

    /// Attempts to make an initial connection to the device
    pub fn connect(mut self) -> io::Result<Result<ProgrammerConnected, ConnectError>> {
        self.target.reset_into(OperatingMode::Boot);

        self.target.clear_buffers();

        for baud_rate in &[9600, 4800, 2400, 1200, 0] {
            if *baud_rate == 0 {
                return Ok(Err(ConnectError::NoResponse));
            }

            self.target.set_baud_rate(*baud_rate)?;

            let mut attempts = 0;
            while self.target.bytes_to_read()? < 1 && attempts < 30 {
                self.target.write(&[0x00])?;
                thread::sleep(time::Duration::from_millis(10));

                attempts += 1;
            }

            if self.target.bytes_to_read()? >= 1 {
                break;
            }
        }

        let mut response1 = [0u8; 1];
        self.target.read_exact(&mut response1)?;
        let response1 = response1[0];

        if response1 != 0x00 {
            return Ok(Err(ConnectError::BadResponse));
        }

        self.target.write(&[0x55])?;

        let mut response2 = [0u8; 1];
        self.target.read_exact(&mut response2)?;
        let response2 = response2[0];

        Ok(match response2 {
            0xE6 => Ok(ProgrammerConnected {
                target: self.target,
            }),
            0xFF => Err(ConnectError::Failed),
            _ => Err(ConnectError::BadResponse),
        })
    }
}

/// A programmer connected to a device
pub struct ProgrammerConnected {
    target: Box<dyn Target>,
}

impl ProgrammerConnected {
    /// Retrieve a list of devices supported by the target
    pub fn supported_devices(&mut self) -> io::Result<Vec<command::data::SupportedDevice>> {
        let cmd = command::commands::SupportedDeviceInquiry {};
        let devices = cmd.execute(&mut self.target)?.unwrap();

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
        let response = cmd.execute(&mut self.target)?;

        Ok(match response {
            Ok(()) => Ok(ProgrammerConnectedDeviceSelected {
                target: self.target,
            }),
            Err(x) => Err(x),
        })
    }
}

/// A programmer connected to a device, with a device selected
pub struct ProgrammerConnectedDeviceSelected {
    target: Box<dyn Target>,
}

impl ProgrammerConnectedDeviceSelected {
    /// Retrieve a list of supported clock modes
    pub fn clock_modes(&mut self) -> io::Result<Vec<u8>> {
        let cmd = command::commands::ClockModeInquiry {};
        let clock_modes = cmd.execute(&mut self.target)?.unwrap();

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
        let response = cmd.execute(&mut self.target)?;

        Ok(match response {
            Ok(()) => Ok(ProgrammerConnectedClockModeSelected {
                target: self.target,
            }),
            Err(x) => Err(x),
        })
    }
}

/// A programmer connected to a device, with a clock mode selected
pub struct ProgrammerConnectedClockModeSelected {
    target: Box<dyn Target>,
}

impl ProgrammerConnectedClockModeSelected {
    /// Retrieve a list of multiplication ratios supported by each clock
    pub fn multiplication_ratios(
        &mut self,
    ) -> io::Result<Vec<Vec<command::data::MultiplicationRatio>>> {
        let cmd = command::commands::MultiplicationRatioInquiry {};
        let multiplication_ratios = cmd.execute(&mut self.target)?.unwrap();

        Ok(multiplication_ratios)
    }

    /// Retrive the operating frequency range of each clock
    pub fn operating_frequencies(&mut self) -> io::Result<Vec<RangeInclusive<u16>>> {
        let cmd = command::commands::OperatingFrequencyInquiry {};
        let operating_frequencies = cmd.execute(&mut self.target)?.unwrap();

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
        let response = cmd.execute(&mut self.target)?;

        Ok(match response {
            Ok(()) => {
                let baud_rate: u32 = (bit_rate * 100).into();
                self.target.set_baud_rate(baud_rate)?;

                let cmd = command::commands::NewBitRateSelectionConfirmation {};
                let _response = cmd.execute(&mut self.target)?;

                Ok(ProgrammerConnectedNewBitRateSelected {
                    target: self.target,
                })
            }
            Err(x) => Err(x),
        })
    }
}

/// A programmer connected to a device, after a new bit rate has been selected
pub struct ProgrammerConnectedNewBitRateSelected {
    target: Box<dyn Target>,
}

impl ProgrammerConnectedNewBitRateSelected {
    /// Retrieves the regions which comprise the user boot area
    pub fn user_boot_area(&mut self) -> io::Result<Vec<RangeInclusive<u32>>> {
        let cmd = command::commands::UserBootAreaInformationInquiry {};
        let response = cmd.execute(&mut self.target)?.unwrap();

        Ok(response)
    }

    /// Retrieves the regions which comprise the user area
    pub fn user_area(&mut self) -> io::Result<Vec<RangeInclusive<u32>>> {
        let cmd = command::commands::UserAreaInformationInquiry {};
        let response = cmd.execute(&mut self.target)?.unwrap();

        Ok(response)
    }

    /// Retrieves the blocks which can be erased
    pub fn erasure_block(&mut self) -> io::Result<Vec<RangeInclusive<u32>>> {
        let cmd = command::commands::ErasureBlockInformationInquiry {};
        let response = cmd.execute(&mut self.target)?.unwrap();

        Ok(response)
    }

    /// Transitions into the programming/erasure wait state
    pub fn programming_erasure_state_transition(
        mut self,
    ) -> io::Result<ProgrammerConnectedProgrammingErasureState> {
        let cmd = command::commands::ProgrammingErasureStateTransition {};
        let response = cmd.execute(&mut self.target)?.unwrap();

        match response {
            command::commands::IDCodeProtectionStatus::Disabled => {
                Ok(ProgrammerConnectedProgrammingErasureState {
                    target: self.target,
                })
            }
            command::commands::IDCodeProtectionStatus::Enabled => {
                panic!("Support for ID codes not implemented")
            }
        }
    }
}

/// A programmer connected to a device, waiting for programming selection commands
pub struct ProgrammerConnectedProgrammingErasureState {
    target: Box<dyn Target>,
}

impl ProgrammerConnectedProgrammingErasureState {
    /// Selects the user area and data area for programming
    pub fn program_user_or_data_area(mut self) -> io::Result<ProgrammerConnectedWaitingForData> {
        let cmd = command::commands::UserDataAreaProgrammingSelection {};
        let response = cmd.execute(&mut self.target)?.unwrap();

        Ok(ProgrammerConnectedWaitingForData {
            target: self.target,
        })
    }

    /// Read `size` bytes of memory starting from `start_address`
    pub fn read_memory(
        &mut self,
        area: command::data::MemoryArea,
        start_address: u32,
        size: u32,
    ) -> io::Result<Result<Vec<u8>, command::commands::MemoryReadError>> {
        let cmd = command::commands::MemoryRead {
            area,
            start_address,
            size,
        };

        Ok(cmd.execute(&mut self.target)?)
    }
}

/// A programmer connected to a device, waiting for data to be programmed into the selected area
pub struct ProgrammerConnectedWaitingForData {
    target: Box<dyn Target>,
}

impl ProgrammerConnectedWaitingForData {
    /// Writes a block of data to the device
    pub fn program_block(
        &mut self,
        address: u32,
        data: [u8; 256],
    ) -> io::Result<Result<(), command::commands::X256ByteProgrammingError>> {
        let cmd = command::commands::X256ByteProgramming {
            address: address,
            data: data,
        };

        cmd.execute(&mut self.target)
    }

    /// Finishes programming
    pub fn end(mut self) -> io::Result<ProgrammerConnectedProgrammingErasureState> {
        let cmd = command::commands::X256ByteProgramming {
            address: 0xFFFFFFFF,
            data: [0u8; 256],
        };
        let response = cmd.execute(&mut self.target)?;

        match response {
            Ok(()) => Ok(ProgrammerConnectedProgrammingErasureState {
                target: self.target,
            }),
            Err(_) => panic!("End programming should not give error!"),
        }
    }
}
