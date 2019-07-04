use super::*;
use std::io;

struct NewBitRateSelection {
    bit_rate: u16,
    input_frequency: u16,
    clock_type_count: u8,
    multiplication_ratio_1: MultiplicationRatio,
    multiplication_ratio_2: MultiplicationRatio,
}

#[derive(Debug, PartialEq)]
enum NewBitRateSelectionError {
    Checksum,
    BitRateSelection,
    InputFrequency,
    MultiplicationRatio,
    OperatingFrequency,
}

impl TransmitCommandData for NewBitRateSelection {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x3F,
            has_size_field: true,
            payload: {
                let mut payload = vec![];
                // TODO: Check endianness
                payload.extend_from_slice(&self.bit_rate.to_le_bytes());
                payload.extend_from_slice(&self.input_frequency.to_le_bytes());
                payload.push(self.clock_type_count);
                payload.push(self.multiplication_ratio_1.into());
                payload.push(self.multiplication_ratio_2.into());
                payload
            },
        }
    }
}

impl Receive for NewBitRateSelection {
    type Response = ();
    type Error = NewBitRateSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let reader: ResponseReader<_, SimpleResponse> = ResponseReader::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorResponseFirstByte::Byte(0xBF),
        );

        let response = reader.read_response()?;

        Ok(match response {
            SimpleResponse::Response(_) => Ok(()),
            SimpleResponse::Error(error) => match error {
                0x11 => Err(NewBitRateSelectionError::Checksum),
                0x24 => Err(NewBitRateSelectionError::BitRateSelection),
                0x25 => Err(NewBitRateSelectionError::InputFrequency),
                0x26 => Err(NewBitRateSelectionError::MultiplicationRatio),
                0x27 => Err(NewBitRateSelectionError::OperatingFrequency),
                _ => panic!("Unknown error code"),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = NewBitRateSelection {
            bit_rate: 0x00C0,
            input_frequency: 0x04E2,
            clock_type_count: 0x02,
            multiplication_ratio_1: MultiplicationRatio::MultiplyBy(4),
            multiplication_ratio_2: MultiplicationRatio::DivideBy(2),
        };
        let command_bytes = [0x3F, 0x07, 0xC0, 0x00, 0xE2, 0x04, 0x02, 0x04, 0xFE, 0x10];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = NewBitRateSelection {
            bit_rate: 0x00C0,
            input_frequency: 0x04E2,
            clock_type_count: 0x02,
            multiplication_ratio_1: MultiplicationRatio::MultiplyBy(4),
            multiplication_ratio_2: MultiplicationRatio::DivideBy(2),
        };
        let response_bytes = [0x06];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = NewBitRateSelection {
            bit_rate: 0x00C0,
            input_frequency: 0x04E2,
            clock_type_count: 0x02,
            multiplication_ratio_1: MultiplicationRatio::MultiplyBy(4),
            multiplication_ratio_2: MultiplicationRatio::DivideBy(2),
        };
        let response_bytes = [0xBF, 0x24];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(NewBitRateSelectionError::BitRateSelection));
        assert!(all_read(&mut p));
    }
}
