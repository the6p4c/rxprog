use super::*;
use std::io;

struct NewBitRateSelection {
    bit_rate: u16,
    input_frequency: u16,
    clock_type_count: u8,
    multiplication_ratio_1: MultiplicationRatio,
    multiplication_ratio_2: MultiplicationRatio,
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
    type Error = Infallible;

    fn rx<T: io::Read>(&self, _p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        panic!("More complicated than tx/rx");
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
}
