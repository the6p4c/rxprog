use super::command_impl_prelude::*;

/// Checks if the data area is unprogrammed
#[derive(Debug)]
pub struct DataAreaBlankCheck {}

impl TransmitCommandData for DataAreaBlankCheck {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x62,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for DataAreaBlankCheck {
    type Response = ErasureState;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0xE2),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(_) => ErasureState::Blank,
            Err(0x52) => ErasureState::NotBlank,
            _ => panic!("Unknown response"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
        let cmd = DataAreaBlankCheck {};
        let command_bytes = [0x62];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_blank() {
        let cmd = DataAreaBlankCheck {};
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(ErasureState::Blank));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_not_blank() {
        let cmd = DataAreaBlankCheck {};
        let response_bytes = [0xE2, 0x52];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(ErasureState::NotBlank));
        assert!(is_script_complete(&mut p));
    }
}
