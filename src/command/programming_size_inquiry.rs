use super::*;
use std::io;

use super::reader::*;

#[derive(Debug)]
pub struct ProgrammingSizeInquiry {}

impl TransmitCommandData for ProgrammingSizeInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x27,
            has_size_field: false,
            payload: vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ProgrammingSizeInquiryResponse {
    pub programming_size: u16,
}

impl Receive for ProgrammingSizeInquiry {
    type Response = ProgrammingSizeInquiryResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x37));

        let data = reader.read_response()?.data;

        let mut programming_size_bytes = [0u8; 2];
        programming_size_bytes.copy_from_slice(&data);

        let programming_size = u16::from_be_bytes(programming_size_bytes);

        Ok(Ok(ProgrammingSizeInquiryResponse {
            programming_size: programming_size,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = ProgrammingSizeInquiry {};
        let command_bytes = [0x27];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = ProgrammingSizeInquiry {};
        let response_bytes = [0x37, 0x02, 0x12, 0x34, 0x81];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(ProgrammingSizeInquiryResponse {
                programming_size: 0x1234,
            })
        );
        assert!(all_read(&mut p));
    }
}
