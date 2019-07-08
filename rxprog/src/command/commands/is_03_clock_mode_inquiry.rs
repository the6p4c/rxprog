use super::*;
use std::io;

use super::reader::*;

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

#[derive(Debug, PartialEq)]
pub struct ClockModeInquiryResponse {
    pub modes: Vec<u8>,
}

impl Receive for ClockModeInquiry {
    type Response = ClockModeInquiryResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x31));

        let data = reader.read_response()?.data;

        Ok(Ok(ClockModeInquiryResponse {
            modes: data.to_vec(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = ClockModeInquiry {};
        let command_bytes = [0x21];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = ClockModeInquiry {};
        let response_bytes = [0x31, 0x02, 0x00, 0x01, 0xCC];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(ClockModeInquiryResponse {
                modes: vec![0x00, 0x01],
            })
        );
        assert!(all_read(&mut p));
    }
}
