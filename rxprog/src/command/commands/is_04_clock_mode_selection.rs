use super::command_impl_prelude::*;

/// Select a clock mode
#[derive(Debug)]
pub struct ClockModeSelection {
    /// The clock mode to select
    pub mode: u8,
}

impl TransmitCommandData for ClockModeSelection {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x11,
            has_size_field: true,
            payload: vec![self.mode],
        }
    }
}

/// Error preventing successful clock mode selection
#[derive(Debug, PartialEq)]
pub enum ClockModeSelectionError {
    /// Command checksum validation failed
    Checksum,
    /// Invalid clock mode
    ClockMode,
}

impl Receive for ClockModeSelection {
    type Response = ();
    type Error = ClockModeSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0x91),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(_) => Ok(()),
            Err(error_code) => Err(match error_code {
                0x11 => ClockModeSelectionError::Checksum,
                0x21 => ClockModeSelectionError::ClockMode,
                _ => panic!("Unknown error code"),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = ClockModeSelection { mode: 0xAB };
        let command_bytes = [0x11, 0x01, 0xAB, 0x43];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = ClockModeSelection { mode: 0xAB };
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = ClockModeSelection { mode: 0xAB };
        let response_bytes = [0x91, 0x21];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(ClockModeSelectionError::ClockMode));
        assert!(is_script_complete(&mut p));
    }
}
