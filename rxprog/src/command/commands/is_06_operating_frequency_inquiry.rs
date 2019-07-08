use std::convert::Infallible;
use std::io;

use super::command::*;
use super::reader::*;

/// Requests the valid frequency range of each clock
#[derive(Debug)]
pub struct OperatingFrequencyInquiry {}

#[derive(Debug, PartialEq)]
pub struct OperatingFrequencyRange {
    /// The clock's minimum frequency
    pub minimum_frequency: u16,
    /// The clock's maximum frequency
    pub maximum_frequency: u16,
}

/// Response to a `OperatingFrequencyInquiry`
#[derive(Debug, PartialEq)]
pub struct OperatingFrequencyInquiryResponse {
    pub clock_types: Vec<OperatingFrequencyRange>,
}

impl TransmitCommandData for OperatingFrequencyInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x23,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for OperatingFrequencyInquiry {
    type Response = OperatingFrequencyInquiryResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x33));

        let data = reader.read_response()?.data;

        let clock_type_count = data[0];

        let mut clock_types: Vec<OperatingFrequencyRange> = vec![];
        let mut remaining_data = &data[1..];
        for _ in 0..clock_type_count {
            let (clock_type_data, new_remaining_data) = remaining_data.split_at(4);

            let mut minimum_frequency_bytes = [0u8; 2];
            minimum_frequency_bytes.copy_from_slice(&clock_type_data[0..=1]);
            let mut maximum_frequency_bytes = [0u8; 2];
            maximum_frequency_bytes.copy_from_slice(&clock_type_data[2..=3]);

            clock_types.push(OperatingFrequencyRange {
                minimum_frequency: u16::from_be_bytes(minimum_frequency_bytes),
                maximum_frequency: u16::from_be_bytes(maximum_frequency_bytes),
            });

            remaining_data = &new_remaining_data;
        }

        Ok(Ok(OperatingFrequencyInquiryResponse {
            clock_types: clock_types,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = OperatingFrequencyInquiry {};
        let command_bytes = [0x23];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = OperatingFrequencyInquiry {};
        let response_bytes = [
            0x33, 0x09, 0x02, // Header
            0x03, 0xE8, 0x07, 0xD0, // Clock type 1
            0x00, 0x64, 0x27, 0x10, // Clock type 2
            0x65, // Checksum
        ];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(OperatingFrequencyInquiryResponse {
                clock_types: vec![
                    OperatingFrequencyRange {
                        minimum_frequency: 1000,
                        maximum_frequency: 2000
                    },
                    OperatingFrequencyRange {
                        minimum_frequency: 100,
                        maximum_frequency: 10000
                    },
                ],
            })
        );
        assert!(is_script_complete(&mut p));
    }
}
