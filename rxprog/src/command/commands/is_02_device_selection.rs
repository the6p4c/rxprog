use std::io;

use super::command::*;
use super::reader::*;

#[derive(Debug)]
pub struct DeviceSelection {
    pub device_code: String,
}

#[derive(Debug, PartialEq)]
pub enum DeviceSelectionError {
    Checksum,
    DeviceCode,
}

impl TransmitCommandData for DeviceSelection {
    fn command_data(&self) -> CommandData {
        assert_eq!(self.device_code.len(), 4);

        CommandData {
            opcode: 0x10,
            has_size_field: true,
            payload: self.device_code.bytes().collect(),
        }
    }
}

impl Receive for DeviceSelection {
    type Response = ();
    type Error = DeviceSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0x90),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(_) => Ok(()),
            Err(error_code) => Err(match error_code {
                0x11 => DeviceSelectionError::Checksum,
                0x21 => DeviceSelectionError::DeviceCode,
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
        let cmd = DeviceSelection {
            device_code: "DEV1".to_string(),
        };
        let command_bytes = [0x10, 0x04, 0x44, 0x45, 0x56, 0x31, 0xDC];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = DeviceSelection {
            device_code: "DEV1".to_string(),
        };
        let response_bytes = [0x06];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = DeviceSelection {
            device_code: "DEV1".to_string(),
        };
        let response_bytes = [0x90, 0x21];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(DeviceSelectionError::DeviceCode));
        assert!(all_read(&mut p));
    }
}
