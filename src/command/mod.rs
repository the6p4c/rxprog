use std::convert::Infallible;
use std::io;
use std::marker::PhantomData;
use std::num::Wrapping;
use std::str;

mod reader;

pub mod block_erasure;
pub mod boot_program_status_inquiry;
pub mod clock_mode_inquiry;
pub mod clock_mode_selection;
pub mod device_selection;
pub mod erasure_block_information_inquiry;
pub mod erasure_selection;
pub mod lock_bit_disable;
pub mod lock_bit_enable;
pub mod lock_bit_program;
pub mod memory_read;
pub mod multiplication_ratio_inquiry;
pub mod new_bit_rate_selection;
pub mod new_bit_rate_selection_confirmation;
pub mod operating_frequency_inquiry;
pub mod programming_erasure_state_transition;
pub mod programming_size_inquiry;
pub mod read_lock_bit_status;
pub mod supported_device_inquiry;
pub mod user_area_blank_check;
pub mod user_area_checksum;
pub mod user_area_information_inquiry;
pub mod user_boot_area_blank_check;
pub mod user_boot_area_checksum;
pub mod user_boot_area_information_inquiry;
pub mod user_boot_area_programming_selection;
pub mod user_data_area_programming_selection;
pub mod x256_byte_programming;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MultiplicationRatio {
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

#[derive(Debug)]
pub enum MemoryArea {
    UserBootArea,
    UserArea,
}

#[derive(Debug, PartialEq)]
pub enum LockBitStatus {
    Locked,
    Unlocked,
}

fn all_read<T: io::Read>(p: &mut T) -> bool {
    let mut buf = [0u8; 1];
    p.read(&mut buf).unwrap() == 0
}
