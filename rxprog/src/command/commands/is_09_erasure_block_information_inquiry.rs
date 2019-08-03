use std::ops::RangeInclusive;

use super::command_impl_prelude::*;

/// Requests information about the indivisible units of memory which support erasure operations
#[derive(Debug)]
pub struct ErasureBlockInformationInquiry {}

impl TransmitCommandData for ErasureBlockInformationInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x26,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for ErasureBlockInformationInquiry {
    type Response = Vec<RangeInclusive<u32>>;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, CommandError>> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u16>, NoError>::new(p, ResponseFirstByte::Byte(0x36));

        let data = reader.read_response()?.data;

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

        Ok(Ok(areas))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = ErasureBlockInformationInquiry {};
        let command_bytes = [0x26];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = ErasureBlockInformationInquiry {};
        let response_bytes = [
            0x36, 0x00, 0x11, 0x02, // Header
            0x10, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, // Area 1
            0x12, 0x34, 0x56, 0x78, 0x89, 0xAB, 0xCD, 0xEF, // Area 2
            0x85, // Checksum
        ];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(vec![0x10000000..=0x20000000, 0x12345678..=0x89ABCDEF])
        );
        assert!(is_script_complete(&mut p));
    }
}
