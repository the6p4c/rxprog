use super::*;
use std::io;

use super::reader::*;

#[derive(Debug)]
pub struct UserAreaBlankCheck {}

impl TransmitCommandData for UserAreaBlankCheck {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x4D,
            has_size_field: false,
            payload: vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ErasureState {
    Blank,
    NotBlank,
}

#[derive(Debug, PartialEq)]
pub struct UserAreaBlankCheckResponse {
    pub state: ErasureState,
}

impl Receive for UserAreaBlankCheck {
    type Response = UserAreaBlankCheckResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0xCD),
        );

        let response = reader.read_response()?;

        let state = match response {
            Ok(_) => ErasureState::Blank,
            Err(error_code) => match error_code {
                0x52 => ErasureState::NotBlank,
                _ => panic!("Unknown error code"),
            },
        };

        Ok(Ok(UserAreaBlankCheckResponse { state: state }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = UserAreaBlankCheck {};
        let command_bytes = [0x4D];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx_blank() {
        let cmd = UserAreaBlankCheck {};
        let response_bytes = [0x06];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(UserAreaBlankCheckResponse {
                state: ErasureState::Blank,
            })
        );
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_not_blank() {
        let cmd = UserAreaBlankCheck {};
        let response_bytes = [0xCD, 0x52];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(UserAreaBlankCheckResponse {
                state: ErasureState::NotBlank,
            })
        );
        assert!(all_read(&mut p));
    }
}
