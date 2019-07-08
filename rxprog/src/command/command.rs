use std::io;
use std::num::Wrapping;

pub trait Command {
    type Response;
    type Error;

    fn execute<T: io::Read + io::Write>(
        &self,
        p: &mut T,
    ) -> io::Result<Result<Self::Response, Self::Error>>;
}

pub trait Transmit {
    fn tx<T: io::Write>(&self, p: &mut T) -> io::Result<()>;
}

pub struct CommandData {
    pub opcode: u8,
    pub has_size_field: bool,
    pub payload: Vec<u8>,
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

pub trait TransmitCommandData {
    fn command_data(&self) -> CommandData;
}

impl<T: TransmitCommandData> Transmit for T {
    fn tx<U: io::Write>(&self, p: &mut U) -> io::Result<()> {
        p.write(&self.command_data().bytes())?;
        p.flush()?;

        Ok(())
    }
}

pub trait Receive {
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

pub fn all_read<T: io::Read>(p: &mut T) -> bool {
    let mut buf = [0u8; 1];
    p.read(&mut buf).unwrap() == 0
}
