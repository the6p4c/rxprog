use super::*;
use std::io;
use std::ops::RangeInclusive;

#[derive(Debug)]
pub struct ErasureBlockInformationInquiry {}

impl TransmitCommandData for ErasureBlockInformationInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x24,
            has_size_field: false,
            payload: vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ErasureBlockInformationInquiryResponse {
    pub areas: Vec<RangeInclusive<u32>>,
}

impl Receive for ErasureBlockInformationInquiry {
    type Response = ErasureBlockInformationInquiryResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let reader: ResponseReader<_, SizedResponse> = ResponseReader::new(
            p,
            ResponseFirstByte::Byte(0x34),
            ErrorResponseFirstByte::None,
        );

        let response = reader.read_response()?;

        Ok(match response {
            SizedResponse::Response(data) => {
                let area_count = data[0];

                let mut areas: Vec<RangeInclusive<u32>> = vec![];
                let mut remaining_data = &data[1..];
                for _ in 0..area_count {
                    let (area_data, new_remaining_data) = remaining_data.split_at(8);

                    let mut area_start_address_bytes = [0u8; 4];
                    area_start_address_bytes.copy_from_slice(&area_data[0..=3]);
                    let mut area_end_address_bytes = [0u8; 4];
                    area_end_address_bytes.copy_from_slice(&area_data[4..=7]);

                    let area_start_address = u32::from_be_bytes(area_start_address_bytes);
                    let area_end_address = u32::from_be_bytes(area_end_address_bytes);

                    // TODO: Check if inclusive
                    areas.push(area_start_address..=area_end_address);

                    remaining_data = new_remaining_data;
                }

                Ok(ErasureBlockInformationInquiryResponse { areas: areas })
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
        let cmd = ErasureBlockInformationInquiry {};
        let command_bytes = [0x24];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = ErasureBlockInformationInquiry {};
        let response_bytes = [
            0x34, 0x11, 0x02, // Header
            0x10, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, // Area 1
            0x12, 0x34, 0x56, 0x78, 0x89, 0xAB, 0xCD, 0xEF, // Area 2
            0x85, // Checksum
        ];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(ErasureBlockInformationInquiryResponse {
                areas: vec![0x10000000..=0x20000000, 0x12345678..=0x89ABCDEF],
            })
        );
        assert!(all_read(&mut p));
    }
}
