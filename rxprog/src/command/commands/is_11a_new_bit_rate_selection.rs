use super::command_impl_prelude::*;

/// Selects a new bit rate for the programmer connection. Must be followed by a
/// `NewBitRateSelectionConfirmation`.
#[derive(Debug)]
pub struct NewBitRateSelection {
    /// New bit rate in bps / 100
    pub bit_rate: u16,
    /// Device input frequency in MHz * 100
    pub input_frequency: u16,
    /// Clock multiplication ratios
    pub multiplication_ratios: Vec<MultiplicationRatio>,
}

impl TransmitCommandData for NewBitRateSelection {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x3F,
            has_size_field: true,
            payload: {
                let mut payload = vec![];
                payload.extend_from_slice(&self.bit_rate.to_be_bytes());
                payload.extend_from_slice(&self.input_frequency.to_be_bytes());
                payload.push(self.multiplication_ratios.len() as u8);
                payload.extend(self.multiplication_ratios.iter().map(|&x| u8::from(x)));
                payload
            },
        }
    }
}

/// Error preventing successful bit rate selection
#[derive(Debug, PartialEq)]
pub enum NewBitRateSelectionError {
    /// Command checksum validation failed
    Checksum,
    /// Bit rate could not be selected within an acceptable margin of error
    BitRateSelection,
    /// Input frequency out of bounds
    InputFrequency,
    /// Multiplication ratio not supported by clock mode
    MultiplicationRatio,
    /// Operating frequency after scaling not supported
    OperatingFrequency,
}

impl Receive for NewBitRateSelection {
    type Response = ();
    type Error = NewBitRateSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0xBF),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(_) => Ok(()),
            Err(error_code) => match error_code {
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
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = NewBitRateSelection {
            bit_rate: 0x00C0,
            input_frequency: 0x04E2,
            multiplication_ratios: vec![
                MultiplicationRatio::MultiplyBy(4),
                MultiplicationRatio::DivideBy(2),
            ],
        };
        let command_bytes = [0x3F, 0x07, 0x00, 0xC0, 0x04, 0xE2, 0x02, 0x04, 0xFE, 0x10];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = NewBitRateSelection {
            bit_rate: 0x00C0,
            input_frequency: 0x04E2,
            multiplication_ratios: vec![
                MultiplicationRatio::MultiplyBy(4),
                MultiplicationRatio::DivideBy(2),
            ],
        };
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = NewBitRateSelection {
            bit_rate: 0x00C0,
            input_frequency: 0x04E2,
            multiplication_ratios: vec![
                MultiplicationRatio::MultiplyBy(4),
                MultiplicationRatio::DivideBy(2),
            ],
        };
        let response_bytes = [0xBF, 0x24];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(NewBitRateSelectionError::BitRateSelection));
        assert!(is_script_complete(&mut p));
    }
}
