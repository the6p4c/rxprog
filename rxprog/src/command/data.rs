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
