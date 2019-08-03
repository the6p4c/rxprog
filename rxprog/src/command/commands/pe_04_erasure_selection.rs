use super::command_impl_prelude::*;

/// Transitions into the erasure wait
#[derive(Debug)]
pub struct ErasureSelection {}

impl TransmitCommandData for ErasureSelection {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x48,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for ErasureSelection {
    type Response = ();

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader =
            ResponseReader::<_, SimpleResponse, NoError>::new(p, ResponseFirstByte::Byte(0x06));

        reader.read_response()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
        let cmd = ErasureSelection {};
        let command_bytes = [0x48];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = ErasureSelection {};
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }
}
