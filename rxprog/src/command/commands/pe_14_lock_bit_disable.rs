use super::command_impl_prelude::*;

/// Disables the lock bit of the selected region
#[derive(Debug)]
pub struct LockBitDisable {}

impl TransmitCommandData for LockBitDisable {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x75,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for LockBitDisable {
    type Response = ();

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, CommandError>> {
        let mut reader =
            ResponseReader::<_, SimpleResponse, NoError>::new(p, ResponseFirstByte::Byte(0x06));

        let _response = reader.read_response()?;

        Ok(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = LockBitDisable {};
        let command_bytes = [0x75];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = LockBitDisable {};
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }
}
