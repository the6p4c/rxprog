use super::command_impl_prelude::*;

/// Confirm a new bit rate (sent after `NewBitRateSelection`)
#[derive(Debug)]
pub struct NewBitRateSelectionConfirmation {}

impl TransmitCommandData for NewBitRateSelectionConfirmation {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x06,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for NewBitRateSelectionConfirmation {
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
        let cmd = NewBitRateSelectionConfirmation {};
        let command_bytes = [0x06];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = NewBitRateSelectionConfirmation {};
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }
}
