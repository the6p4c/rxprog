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

impl Transmit for DeviceSelection {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x10,
            has_size_field: true,
            // TODO: Check endianness
            payload: self.device_code.to_le_bytes().to_vec(),
        }
        .bytes()
    }

    fn tx<T: io::Write>(&self, p: &mut T) {
        p.write(&self.bytes());
        p.flush();
    }
}

impl Receive for DeviceSelection {
    type Response = ();
    type Error = DeviceSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        match b1 {
            0x06 => Ok(()),
            0x90 => {
                let mut error = [0u8; 1];
                p.read(&mut error);
                let error = error[0];

                Err(match error {
                    0x11 => DeviceSelectionError::Checksum,
                    0x21 => DeviceSelectionError::DeviceCode,
                    _ => panic!("Unknown error code"),
                })
            }
            _ => panic!("Invalid response received"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() {
        let cmd = DeviceSelection {
            device_code: 0x12345678,
        };

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x10, 0x04, 0x78, 0x56, 0x34, 0x12, 0xD8]);
    }

    #[test]
    fn test_rx_success() {
        let cmd = DeviceSelection {
            device_code: 0x12345678,
        };
        let response_bytes = vec![0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(()));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = DeviceSelection {
            device_code: 0x12345678,
        };
        let response_bytes = vec![0x90, 0x21];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Err(DeviceSelectionError::DeviceCode));
        assert!(all_read(&mut p));
    }
}
