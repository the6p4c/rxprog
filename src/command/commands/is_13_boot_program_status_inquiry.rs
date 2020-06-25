use super::command_impl_prelude::*;

/// Requests the current status of the device
#[derive(Debug)]
pub struct BootProgramStatusInquiry {}

impl TransmitCommandData for BootProgramStatusInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x4F,
            has_size_field: false,
            payload: vec![],
        }
    }
}

/// The current status of the device
#[derive(Debug, PartialEq)]
pub enum BootProgramStatus {
    /// Waiting to be issued a `DeviceSelection` command
    WaitingForDeviceSelection,
    /// Waiting to be issued a `ClockModeSelection` command
    WaitingForClockModeSelection,
    /// Waiting to be issued a `NewBitRateSelection` command
    WaitingForBitRateSelection,
    /// Waiting to be issued a `ProgrammingErasureStateTransition` command
    WaitingForTransitionToProgrammingErasureCommandWait,
    /// Busy erasing user area or user boot area
    ErasingUserAreaAndUserBootArea,
    /// Waiting to be issued a valid programming/erasure command wait command
    WaitingForProgrammingErasureCommand,
    /// Waiting to be issued a `X256ByteProgramming` command
    WaitingForProgrammingData,
    /// Waiting to be issued a `BlockErasure` command
    WaitingForErasureBlockSpecification,
}

impl From<u8> for BootProgramStatus {
    fn from(item: u8) -> Self {
        match item {
            0x11 => BootProgramStatus::WaitingForDeviceSelection,
            0x12 => BootProgramStatus::WaitingForClockModeSelection,
            0x13 => BootProgramStatus::WaitingForBitRateSelection,
            0x1F => BootProgramStatus::WaitingForTransitionToProgrammingErasureCommandWait,
            0x31 => BootProgramStatus::ErasingUserAreaAndUserBootArea,
            0x3F => BootProgramStatus::WaitingForProgrammingErasureCommand,
            0x4F => BootProgramStatus::WaitingForProgrammingData,
            0x5F => BootProgramStatus::WaitingForErasureBlockSpecification,
            _ => panic!("Invalid status code"),
        }
    }
}

/// Last error encountered
#[derive(Debug, PartialEq)]
pub enum BootProgramError {
    /// No errors
    NoError,
    /// Command checksum validation failed
    Checksum,
    /// Invalid device code
    IncorrectDeviceCode,
    /// Invalid clock mode
    IncorrectClockMode,
    /// Bit rate could not be selected within an acceptable margin of error
    BitRateSelection,
    /// Input frequency out of bounds
    InputFrequency,
    /// Multiplication ratio not supported by clock mode
    MultiplicationRatio,
    /// Operating frequency after scaling not supported
    OperatingFrequency,
    /// Incorrect block number
    BlockNumber,
    /// Invalid address
    Address,
    /// Invalid data read size
    DataSize,
    /// Failed to complete ID code mismatch triggered erasure
    Erasure,
    /// Failed to complete erasure
    IncompleteErasure,
    /// Failed to complete programming
    Programming,
    /// Unknown
    Selection,
    /// Invalid command opcode
    Command,
    /// Bitrate could not be selected within an acceptable margin of error
    BitRateAdjustmentConfirmation,
}

impl From<u8> for BootProgramError {
    fn from(item: u8) -> Self {
        match item {
            0x00 => BootProgramError::NoError,
            0x11 => BootProgramError::Checksum,
            0x21 => BootProgramError::IncorrectDeviceCode,
            0x22 => BootProgramError::IncorrectClockMode,
            0x24 => BootProgramError::BitRateSelection,
            0x25 => BootProgramError::InputFrequency,
            0x26 => BootProgramError::MultiplicationRatio,
            0x27 => BootProgramError::OperatingFrequency,
            0x29 => BootProgramError::BlockNumber,
            0x2A => BootProgramError::Address,
            0x2B => BootProgramError::DataSize,
            0x51 => BootProgramError::Erasure,
            0x52 => BootProgramError::IncompleteErasure,
            0x53 => BootProgramError::Programming,
            0x54 => BootProgramError::Selection,
            0x80 => BootProgramError::Command,
            0xFF => BootProgramError::BitRateAdjustmentConfirmation,
            _ => panic!("Invalid error code"),
        }
    }
}

/// Response to a `BootProgramStatusInquiry`
#[derive(Debug, PartialEq)]
pub struct BootProgramStatusInquiryResponse {
    /// Current device status
    pub status: BootProgramStatus,
    /// Last error
    pub error: BootProgramError,
}

impl Receive for BootProgramStatusInquiry {
    type Response = BootProgramStatusInquiryResponse;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x5F));

        let data = reader.read_response()?.data;

        let status = BootProgramStatus::from(data[0]);
        let error = BootProgramError::from(data[1]);

        Ok(BootProgramStatusInquiryResponse {
            status: status,
            error: error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
        let cmd = BootProgramStatusInquiry {};
        let command_bytes = [0x4F];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = BootProgramStatusInquiry {};
        let response_bytes = [0x5F, 0x02, 0x13, 0x24, 0x68];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(
            response,
            Ok(BootProgramStatusInquiryResponse {
                status: BootProgramStatus::WaitingForBitRateSelection,
                error: BootProgramError::BitRateSelection,
            })
        );
        assert!(is_script_complete(&mut p));
    }
}
