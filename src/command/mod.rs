use std::convert::Infallible;
use std::io;
use std::num::Wrapping;
use std::str;

mod clock_mode_inquiry;
mod clock_mode_selection;
mod device_selection;
mod multiplication_ratio_inquiry;
mod new_bit_rate_selection;
mod operating_frequency_inquiry;
mod programming_erasure_state_transition;
mod supported_device_inquiry;

trait Command {
    type Response;
    type Error;

    fn execute<T: io::Read + io::Write>(
        &self,
        p: &mut T,
    ) -> io::Result<Result<Self::Response, Self::Error>>;
}

trait Transmit {
    fn tx<T: io::Write>(&self, p: &mut T) -> io::Result<()>;
}

struct CommandData {
    opcode: u8,
    has_size_field: bool,
    payload: Vec<u8>,
}

impl CommandData {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        let payload = &self.payload;
        let payload_size = payload.len();

        bytes.push(self.opcode);

        if self.has_size_field {
            bytes.push(payload_size as u8);
        }

        bytes.extend(payload);

        if payload_size != 0 {
            let sum = bytes.iter().map(|x| Wrapping(*x)).sum::<Wrapping<u8>>().0;
            let checksum = !sum + 1;
            bytes.push(checksum);
        }

        bytes
    }
}

trait TransmitCommandData {
    fn command_data(&self) -> CommandData;
}

impl<T: TransmitCommandData> Transmit for T {
    fn tx<U: io::Write>(&self, p: &mut U) -> io::Result<()> {
        p.write(&self.command_data().bytes())?;
        p.flush()?;

        Ok(())
    }
}

trait Receive {
    type Response;
    type Error;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>>;
}

impl<T: Transmit + Receive> Command for T {
    type Response = T::Response;
    type Error = T::Error;

    fn execute<U: io::Read + io::Write>(
        &self,
        p: &mut U,
    ) -> io::Result<Result<Self::Response, Self::Error>> {
        self.tx(p)?;
        Ok(self.rx(p)?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MultiplicationRatio {
    DivideBy(u8),
    MultiplyBy(u8),
}

impl From<u8> for MultiplicationRatio {
    fn from(item: u8) -> Self {
        let item_signed = i8::from_le_bytes([item]);
        let ratio = item_signed.abs() as u8;

        match item_signed {
            x if x < 0 => MultiplicationRatio::DivideBy(ratio),
            x if x > 0 => MultiplicationRatio::MultiplyBy(ratio),
            _ => panic!("Multiplication ratio cannot be zero"),
        }
    }
}

impl From<MultiplicationRatio> for u8 {
    fn from(item: MultiplicationRatio) -> Self {
        match item {
            MultiplicationRatio::DivideBy(ratio) => -(ratio as i8) as u8,
            MultiplicationRatio::MultiplyBy(ratio) => ratio as u8,
        }
    }
}

fn all_read<T: io::Read>(p: &mut T) -> bool {
    let mut buf = [0u8; 1];
    p.read(&mut buf).unwrap() == 0
}
