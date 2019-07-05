use super::*;
use std::io;

#[derive(Debug)]
pub struct ClockModeSelection {
    pub mode: u8,
}

#[derive(Debug, PartialEq)]
pub enum ClockModeSelectionError {
    Checksum,
    ClockMode,
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

impl Receive for ClockModeSelection {
    type Response = ();
    type Error = ClockModeSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let reader: ResponseReader<_, SimpleResponse> = ResponseReader::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorResponseFirstByte::Byte(0x91),
        );

        let response = reader.read_response()?;

        Ok(match response {
            SimpleResponse::Response(_) => Ok(()),
            SimpleResponse::Error(error) => Err(match error {
                0x11 => ClockModeSelectionError::Checksum,
                0x21 => ClockModeSelectionError::ClockMode,
                _ => panic!("Unknown error code"),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = ClockModeSelection { mode: 0xAB };
        let command_bytes = [0x11, 0x01, 0xAB, 0x43];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = ClockModeSelection { mode: 0xAB };
        let response_bytes = [0x06];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = ClockModeSelection { mode: 0xAB };
        let response_bytes = [0x91, 0x21];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(ClockModeSelectionError::ClockMode));
        assert!(all_read(&mut p));
    }
}
