use super::*;
use std::io;

struct ClockModeSelection {
    mode: u8,
}

#[derive(Debug, PartialEq)]
enum ClockModeSelectionError {
    Checksum,
    ClockMode,
}

impl Transmit for ClockModeSelection {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x11,
            has_size_field: true,
            payload: vec![self.mode],
        }
        .bytes()
    }

    fn tx<T: io::Write>(&self, p: &mut T) {
        p.write(&self.bytes());
        p.flush();
    }
}

impl Receive for ClockModeSelection {
    type Response = ();
    type Error = ClockModeSelectionError;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        match b1 {
            0x06 => Ok(()),
            0x91 => {
                let mut error = [0u8; 1];
                p.read(&mut error);
                let error = error[0];

                Err(match error {
                    0x11 => ClockModeSelectionError::Checksum,
                    0x21 => ClockModeSelectionError::ClockMode,
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
        let cmd = ClockModeSelection { mode: 0xAB };

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x11, 0x01, 0xAB, 0x43]);
    }

    #[test]
    fn test_rx_success() {
        let cmd = ClockModeSelection { mode: 0xAB };
        let response_bytes = vec![0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(()));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = ClockModeSelection { mode: 0xAB };
        let response_bytes = vec![0x91, 0x21];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Err(ClockModeSelectionError::ClockMode));
        assert!(all_read(&mut p));
    }
}
