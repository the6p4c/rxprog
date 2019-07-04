use super::*;
use std::io;

struct ClockModeInquiry {}

impl Transmit for ClockModeInquiry {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x21,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }

    fn tx<T: io::Write>(&self, p: &mut T) {
        p.write(&self.bytes());
        p.flush();
    }
}

impl Receive for ClockModeInquiry {
    type Response = u8;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        panic!("Datasheet unclear - test on real device");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() {
        let cmd = ClockModeInquiry {};

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x21]);
    }
}
