use super::command_impl_prelude::*;
/// Request a list of supported clock modes
#[derive(Debug)]
pub struct ClockModeInquiry {}

impl TransmitCommandData for ClockModeInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x21,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for ClockModeInquiry {
    type Response = Vec<u8>;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x31));

        Ok(reader.read_response()?.data)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
        let cmd = ClockModeInquiry {};
        let command_bytes = [0x21];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = ClockModeInquiry {};
        let response_bytes = [0x31, 0x02, 0x00, 0x01, 0xCC];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(vec![0x00, 0x01]));
        assert!(is_script_complete(&mut p));
    }
}
