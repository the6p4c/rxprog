use std::convert::Infallible;
use std::io;

use super::command::*;
use super::reader::*;

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

#[derive(Debug, PartialEq)]
pub enum BootProgramStatus {
    WaitingForDeviceSelection,
    WaitingForClockModeSelection,
    WaitingForBitRateSelection,
    WaitingForTransitionToProgrammingErasureCommandWait,
    ErasingUserAreaAndUserBootArea,
    WaitingForProgrammingErasureCommand,
    WaitingForProgrammingData,
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

#[derive(Debug, PartialEq)]
pub enum BootProgramError {
    NoError,
    Checksum,
    IncorrectDeviceCode,
    IncorrectClockMode,
    BitRateSelection,
    InputFrequency,
    MultiplicationRatio,
    OperatingFrequency,
    BlockNumber,
    Address,
    DataSize,
    Erasure,
    IncompleteErasure,
    Programming,
    Selection,
    Command,
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

#[derive(Debug, PartialEq)]
pub struct BootProgramStatusInquiryResponse {
    pub status: BootProgramStatus,
    pub error: BootProgramError,
}

impl Receive for BootProgramStatusInquiry {
    type Response = BootProgramStatusInquiryResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x5F));

        let data = reader.read_response()?.data;

        let status = BootProgramStatus::from(data[0]);
        let error = BootProgramError::from(data[1]);

        Ok(Ok(BootProgramStatusInquiryResponse {
            status: status,
            error: error,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_util::is_script_complete;

    #[test]
    fn test_tx() -> io::Result<()> {
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

        let response = cmd.rx(&mut p).unwrap();

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
