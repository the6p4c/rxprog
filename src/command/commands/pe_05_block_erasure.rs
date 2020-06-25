use super::command_impl_prelude::*;

/// Erases a block
#[derive(Debug)]
pub struct BlockErasure {
    /// Index of the block to erase, as returned by `ErasureBlockInformationInquiry`
    pub block: u8,
}

impl TransmitCommandData for BlockErasure {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x58,
            has_size_field: true,
            payload: vec![self.block],
        }
    }
}

impl Receive for BlockErasure {
    type Response = ();

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0xD8),
        );

        reader
            .read_response()?
            .map(|_| ())
            .map_err(|error_code| match error_code {
                0x11 => CommandError::Checksum.into(),
                0x29 => CommandError::BlockNumber.into(),
                0x51 => CommandError::Erasure.into(),
                _ => panic!("Unknown error code"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
        let cmd = BlockErasure { block: 0x38 };
        let command_bytes = [0x58, 0x01, 0x38, 0x6F];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = BlockErasure { block: 0x38 };
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = BlockErasure { block: 0x38 };
        let response_bytes = [0xD8, 0x29];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Err(CommandError::BlockNumber.into()));
        assert!(is_script_complete(&mut p));
    }
}
