use std::io;

use super::command::*;
use super::data::MemoryArea;
use super::reader::*;

#[derive(Debug)]
pub struct MemoryRead {
    pub area: MemoryArea,
    pub start_address: u32,
    pub size: u32,
}

impl TransmitCommandData for MemoryRead {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x52,
            has_size_field: true,
            payload: {
                let mut payload = vec![];
                payload.push(match self.area {
                    MemoryArea::UserBootArea => 0x00,
                    MemoryArea::UserArea => 0x01,
                });
                payload.extend(&self.start_address.to_be_bytes());
                payload.extend(&self.size.to_be_bytes());
                payload
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MemoryReadResponse {
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum MemoryReadError {
    Checksum,
    Address,
    DataSize,
}

impl Receive for MemoryRead {
    type Response = MemoryReadResponse;
    type Error = MemoryReadError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SizedResponse<u32>, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x52),
            ErrorFirstByte(0xD2),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(SizedResponse { data, .. }) => Ok(MemoryReadResponse { data: data }),
            Err(error_code) => Err(match error_code {
                0x11 => MemoryReadError::Checksum,
                0x2A => MemoryReadError::Address,
                0x2B => MemoryReadError::DataSize,
                _ => panic!("Unknown error code"),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = MemoryRead {
            area: MemoryArea::UserArea,
            start_address: 0x12345678,
            size: 0x0A,
        };
        let command_bytes = [
            0x52, 0x09, 0x01, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x0A, 0x86,
        ];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = MemoryRead {
            area: MemoryArea::UserArea,
            start_address: 0x12345678,
            size: 0x0A,
        };
        let response_bytes = [
            0x52, 0x00, 0x00, 0x00, 0x0A, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            0x0A, 0x6D,
        ];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(MemoryReadResponse {
                data: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A],
            })
        );
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = MemoryRead {
            area: MemoryArea::UserArea,
            start_address: 0x12345678,
            size: 0x10,
        };
        let response_bytes = [0xD2, 0x2A];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(MemoryReadError::Address),);
        assert!(all_read(&mut p));
    }
}
