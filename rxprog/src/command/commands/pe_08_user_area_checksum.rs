use super::command_impl_prelude::*;

/// Requests a 32-bit checksum of the user area
#[derive(Debug)]
pub struct UserAreaChecksum {}

impl TransmitCommandData for UserAreaChecksum {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x4B,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for UserAreaChecksum {
    type Response = u32;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, CommandError>> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x5B));

        let data = reader.read_response()?.data;

        let mut checksum_bytes = [0u8; 4];
        checksum_bytes.copy_from_slice(&data);

        let checksum = u32::from_be_bytes(checksum_bytes);

        Ok(Ok(checksum))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = UserAreaChecksum {};
        let command_bytes = [0x4B];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = UserAreaChecksum {};
        let response_bytes = [0x5B, 0x04, 0x12, 0x34, 0x56, 0x78, 0x8E];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(0x12345678));
        assert!(is_script_complete(&mut p));
    }
}
