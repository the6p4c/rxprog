use super::*;
use std::io;

#[derive(Debug)]
pub struct NewBitRateSelectionConfirmation {}

impl TransmitCommandData for NewBitRateSelectionConfirmation {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x06,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for NewBitRateSelectionConfirmation {
    type Response = ();
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let reader: ResponseReader<_, SimpleResponse> = ResponseReader::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorResponseFirstByte::None,
        );

        reader.read_response()?;

        Ok(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = NewBitRateSelectionConfirmation {};
        let command_bytes = [0x06];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = NewBitRateSelectionConfirmation {};
        let response_bytes = [0x06];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(all_read(&mut p));
    }
}
