use super::*;
use std::io;

struct ProgrammingErasureStateTransition {}

#[derive(Debug, PartialEq)]
enum IDCodeProtectionStatus {
    Disabled,
    Enabled,
}

impl TransmitCommandData for ProgrammingErasureStateTransition {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x40,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for ProgrammingErasureStateTransition {
    type Response = IDCodeProtectionStatus;
    type Error = ();

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        match b1 {
            0x26 => Ok(IDCodeProtectionStatus::Disabled),
            0x16 => Ok(IDCodeProtectionStatus::Enabled),
            0xC0 => {
                let mut b2 = [0u8; 1];
                p.read(&mut b2);
                let b2 = b2[0];

                assert_eq!(b2, 0x51);

                Err(())
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
        let cmd = ProgrammingErasureStateTransition {};
        let command_bytes = vec![0x40];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p);
    }

    #[test]
    fn test_rx_success_id_disabled() {
        let cmd = ProgrammingErasureStateTransition {};
        let response_bytes = vec![0x26];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(IDCodeProtectionStatus::Disabled));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_success_id_enabled() {
        let cmd = ProgrammingErasureStateTransition {};
        let response_bytes = vec![0x16];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(IDCodeProtectionStatus::Enabled));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = ProgrammingErasureStateTransition {};
        let response_bytes = vec![0xC0, 0x51];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Err(()));
        assert!(all_read(&mut p));
    }
}
