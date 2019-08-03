use super::command_impl_prelude::*;

/// Enables the lock bit of the selected region
#[derive(Debug)]
pub struct LockBitEnable {}

impl TransmitCommandData for LockBitEnable {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x7A,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for LockBitEnable {
    type Response = ();
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
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
        let cmd = LockBitEnable {};
        let command_bytes = [0x7A];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = LockBitEnable {};
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }
}
