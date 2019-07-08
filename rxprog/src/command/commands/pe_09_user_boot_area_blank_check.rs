use std::convert::Infallible;
use std::io;

use super::command::*;
use super::reader::*;

#[derive(Debug)]
pub struct UserBootAreaBlankCheck {}

impl TransmitCommandData for UserBootAreaBlankCheck {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x4C,
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
pub struct UserBootAreaBlankCheckResponse {
    pub state: ErasureState,
}

impl Receive for UserBootAreaBlankCheck {
    type Response = UserBootAreaBlankCheckResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0xCC),
        );

        let response = reader.read_response()?;

        let state = match response {
            Ok(_) => ErasureState::Blank,
            Err(error_code) => match error_code {
                0x52 => ErasureState::NotBlank,
                _ => panic!("Unknown error code"),
            },
        };

        Ok(Ok(UserBootAreaBlankCheckResponse { state: state }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_util::is_script_complete;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = UserBootAreaBlankCheck {};
        let command_bytes = [0x4C];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_blank() {
        let cmd = UserBootAreaBlankCheck {};
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(UserBootAreaBlankCheckResponse {
                state: ErasureState::Blank,
            })
        );
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_not_blank() {
        let cmd = UserBootAreaBlankCheck {};
        let response_bytes = [0xCC, 0x52];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(UserBootAreaBlankCheckResponse {
                state: ErasureState::NotBlank,
            })
        );
        assert!(is_script_complete(&mut p));
    }
}
