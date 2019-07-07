use super::*;
use std::io;

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
        let reader: ResponseReader<_, SizedResponse<u8>> = ResponseReader::new(
            p,
            ResponseFirstByte::Byte(0x5A),
            ErrorResponseFirstByte::None,
        );

        let response = reader.read_response()?;

        Ok(match response {
            SizedResponse::Response(data, _) => {
                let mut checksum_bytes = [0u8; 4];
                checksum_bytes.copy_from_slice(&data);

                let checksum = u32::from_be_bytes(checksum_bytes);

                Ok(UserBootAreaChecksumResponse { checksum: checksum })
            }
            SizedResponse::Error(_) => panic!("Error should not ocurr"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = UserBootAreaChecksum {};
        let command_bytes = [0x4A];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = UserBootAreaChecksum {};
        let response_bytes = [0x5A, 0x04, 0x12, 0x34, 0x56, 0x78, 0x8E];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(UserBootAreaChecksumResponse {
                checksum: 0x12345678,
            })
        );
        assert!(all_read(&mut p));
    }
}
