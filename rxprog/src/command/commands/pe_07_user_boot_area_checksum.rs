use std::convert::Infallible;
use std::io;

use super::command::*;
use super::reader::*;

#[derive(Debug)]
pub struct UserBootAreaChecksum {}

impl TransmitCommandData for UserBootAreaChecksum {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x4A,
            has_size_field: false,
            payload: vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct UserBootAreaChecksumResponse {
    pub checksum: u32,
}

impl Receive for UserBootAreaChecksum {
    type Response = UserBootAreaChecksumResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x5A));

        let data = reader.read_response()?.data;

        let mut checksum_bytes = [0u8; 4];
        checksum_bytes.copy_from_slice(&data);

        let checksum = u32::from_be_bytes(checksum_bytes);

        Ok(Ok(UserBootAreaChecksumResponse { checksum: checksum }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_util::is_script_complete;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = UserBootAreaChecksum {};
        let command_bytes = [0x4A];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = UserBootAreaChecksum {};
        let response_bytes = [0x5A, 0x04, 0x12, 0x34, 0x56, 0x78, 0x8E];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(UserBootAreaChecksumResponse {
                checksum: 0x12345678,
            })
        );
        assert!(is_script_complete(&mut p));
    }
}
