use super::*;
use std::io;

struct ClockModeInquiry {}

impl TransmitCommandData for ClockModeInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x21,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for ClockModeInquiry {
    type Response = u8;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, _p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        panic!("Datasheet unclear - test on real device");
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
}
