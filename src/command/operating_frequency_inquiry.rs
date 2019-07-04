use super::*;
use std::io;

struct OperatingFrequencyInquiry {}

#[derive(Debug, PartialEq)]
struct OperatingFrequencyRange {
    minimum_frequency: u16,
    maximum_frequency: u16,
}

#[derive(Debug, PartialEq)]
struct OperatingFrequencyInquiryResponse {
    clock_types: Vec<OperatingFrequencyRange>,
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
        let reader: ResponseReader<_, SizedResponse> = ResponseReader::new(
            p,
            ResponseFirstByte::Byte(0x33),
            ErrorResponseFirstByte::None,
        );

        let response = reader.read_response()?;

        Ok(match response {
            SizedResponse::Response(data) => {
                let clock_type_count = data[0];

                let mut clock_types: Vec<OperatingFrequencyRange> = vec![];
                let mut remaining_data = &data[1..];
                for _ in 0..clock_type_count {
                    let mut minimum_frequency_bytes = [0u8; 2];
                    minimum_frequency_bytes.copy_from_slice(&remaining_data[0..=1]);
                    let mut maximum_frequency_bytes = [0u8; 2];
                    maximum_frequency_bytes.copy_from_slice(&remaining_data[2..=3]);

                    clock_types.push(OperatingFrequencyRange {
                        // TODO: Check endianness
                        minimum_frequency: u16::from_le_bytes(minimum_frequency_bytes),
                        maximum_frequency: u16::from_le_bytes(maximum_frequency_bytes),
                    });

                    remaining_data = &remaining_data[4..];
                }

                Ok(OperatingFrequencyInquiryResponse {
                    clock_types: clock_types,
                })
            }
            SizedResponse::Error(_) => panic!("Error should not ocurr"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = OperatingFrequencyInquiry {};
        let command_bytes = [0x23];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = OperatingFrequencyInquiry {};
        let response_bytes = [
            0x33, 0x09, 0x02, // Header
            0xE8, 0x03, 0xD0, 0x07, // Clock type 1
            0x64, 0x00, 0x10, 0x27, // Clock type 2
            0x65, // Checksum
        ];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

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
        assert!(all_read(&mut p));
    }
}
