use super::*;
use std::io;

struct DeviceSelection {
    device_code: u32,
}

#[derive(Debug, PartialEq)]
enum DeviceSelectionError {
    Checksum,
    DeviceCode,
}

impl TransmitCommandData for DeviceSelection {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x10,
            has_size_field: true,
            // TODO: Check endianness
            payload: self.device_code.to_le_bytes().to_vec(),
        }
    }
}

impl Receive for DeviceSelection {
    type Response = ();
    type Error = DeviceSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut b1 = [0u8; 1];
        p.read_exact(&mut b1)?;
        let b1 = b1[0];

        match b1 {
            0x06 => Ok(Ok(())),
            0x90 => {
                let mut error = [0u8; 1];
                p.read_exact(&mut error)?;
                let error = error[0];

                Ok(Err(match error {
                    0x11 => DeviceSelectionError::Checksum,
                    0x21 => DeviceSelectionError::DeviceCode,
                    _ => panic!("Unknown error code"),
                }))
            }
            _ => panic!("Invalid response received"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = DeviceSelection {
            device_code: 0x12345678,
        };
        let command_bytes = [0x10, 0x04, 0x78, 0x56, 0x34, 0x12, 0xD8];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = DeviceSelection {
            device_code: 0x12345678,
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
            device_code: 0x12345678,
        };
        let response_bytes = [0x90, 0x21];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(DeviceSelectionError::DeviceCode));
        assert!(all_read(&mut p));
    }
}
