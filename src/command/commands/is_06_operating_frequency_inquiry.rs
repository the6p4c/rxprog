use std::ops::RangeInclusive;

use super::command_impl_prelude::*;

/// Requests the valid frequency range of each clock
#[derive(Debug)]
pub struct OperatingFrequencyInquiry {}

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
    type Response = Vec<RangeInclusive<u16>>;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x33));

        let data = reader.read_response()?.data;

        let clock_type_count = data[0];

        let mut clock_types: Vec<RangeInclusive<u16>> = vec![];
        let mut remaining_data = &data[1..];
        for _ in 0..clock_type_count {
            let (clock_type_data, new_remaining_data) = remaining_data.split_at(4);

            let mut minimum_frequency_bytes = [0u8; 2];
            minimum_frequency_bytes.copy_from_slice(&clock_type_data[0..=1]);
            let mut maximum_frequency_bytes = [0u8; 2];
            maximum_frequency_bytes.copy_from_slice(&clock_type_data[2..=3]);

            let minimum_frequency = u16::from_be_bytes(minimum_frequency_bytes);
            let maximum_frequency = u16::from_be_bytes(maximum_frequency_bytes);

            clock_types.push(minimum_frequency..=maximum_frequency);

            remaining_data = &new_remaining_data;
        }

        Ok(clock_types)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
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

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(vec![1000..=2000, 100..=10000]));
        assert!(is_script_complete(&mut p));
    }
}
