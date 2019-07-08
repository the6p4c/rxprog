use std::io;

use super::command::*;
use super::reader::*;

#[derive(Debug)]
pub struct ProgrammingErasureStateTransition {}

#[derive(Debug, PartialEq)]
pub enum IDCodeProtectionStatus {
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

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::OneByteOf(vec![0x26, 0x16]),
            ErrorFirstByte(0xC0),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(SimpleResponse { first_byte }) => match first_byte {
                0x26 => Ok(IDCodeProtectionStatus::Disabled),
                0x16 => Ok(IDCodeProtectionStatus::Enabled),
                // TODO: Consider modifying ResponseReader so this can't happen
                _ => panic!("Response with unknown first byte"),
            },
            Err(0x51) => Err(()),
            Err(_) => panic!("Error with unknown second byte"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = ProgrammingErasureStateTransition {};
        let command_bytes = [0x40];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx_success_id_disabled() {
        let cmd = ProgrammingErasureStateTransition {};
        let response_bytes = [0x26];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(IDCodeProtectionStatus::Disabled));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_success_id_enabled() {
        let cmd = ProgrammingErasureStateTransition {};
        let response_bytes = vec![0x16];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(IDCodeProtectionStatus::Enabled));
        assert!(all_read(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = ProgrammingErasureStateTransition {};
        let response_bytes = vec![0xC0, 0x51];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(()));
        assert!(all_read(&mut p));
    }
}
