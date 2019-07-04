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

impl Command for NewBitRateSelection {
    type Response = ();
    type Error = NewBitRateSelectionError;

    fn execute<T: io::Read + io::Write>(
        &self,
        p: &mut T,
    ) -> io::Result<Result<Self::Response, Self::Error>> {
        let cd = CommandData {
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
        };

        p.write(&cd.bytes())?;
        p.flush()?;

        let mut b1 = [0u8; 1];
        p.read_exact(&mut b1)?;
        let b1 = b1[0];

        match b1 {
            0x06 => {
                // TODO: Actually wait and set new bit rate

                p.write(&[0x06])?;
                p.flush()?;

                let mut b2 = [0u8; 1];
                p.read_exact(&mut b2)?;
                let b2 = b2[0];

                assert_eq!(b2, 0x06);

                Ok(Ok(()))
            }
            0xBF => {
                let mut error = [0u8; 1];
                p.read_exact(&mut error)?;
                let error = error[0];

                match error {
                    0x11 => Ok(Err(NewBitRateSelectionError::Checksum)),
                    0x24 => Ok(Err(NewBitRateSelectionError::BitRateSelection)),
                    0x25 => Ok(Err(NewBitRateSelectionError::InputFrequency)),
                    0x26 => Ok(Err(NewBitRateSelectionError::MultiplicationRatio)),
                    0x27 => Ok(Err(NewBitRateSelectionError::OperatingFrequency)),
                    _ => panic!("Unknown error code"),
                }
            }
            _ => panic!("Invalid response received"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success() -> io::Result<()> {
        let cmd = NewBitRateSelection {
            bit_rate: 0x00C0,
            input_frequency: 0x04E2,
            clock_type_count: 0x02,
            multiplication_ratio_1: MultiplicationRatio::MultiplyBy(4),
            multiplication_ratio_2: MultiplicationRatio::DivideBy(2),
        };
        let command_bytes = [
            0x3F, 0x07, 0xC0, 0x00, 0xE2, 0x04, 0x02, 0x04, 0xFE, 0x10, // Command
            0x06, // Confirmation
        ];
        let response_bytes = [0x06, 0x06];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.execute(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);
        assert_eq!(response, Ok(()));
        assert!(all_read(&mut p));

        Ok(())
    }

    #[test]
    fn test_fail() -> io::Result<()> {
        let cmd = NewBitRateSelection {
            bit_rate: 0x00C0,
            input_frequency: 0x04E2,
            clock_type_count: 0x02,
            multiplication_ratio_1: MultiplicationRatio::MultiplyBy(4),
            multiplication_ratio_2: MultiplicationRatio::DivideBy(2),
        };
        let command_bytes = [0x3F, 0x07, 0xC0, 0x00, 0xE2, 0x04, 0x02, 0x04, 0xFE, 0x10];
        let response_bytes = [0xBF, 0x25];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.execute(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);
        assert_eq!(response, Err(NewBitRateSelectionError::InputFrequency));
        assert!(all_read(&mut p));

        Ok(())
    }
}
