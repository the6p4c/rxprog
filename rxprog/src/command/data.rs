/// A device supported by the boot program
#[derive(Debug, PartialEq)]
pub struct SupportedDevice {
    /// A 4 character identifier
    pub device_code: String,
    /// Human-readable name of the device
    pub series_name: String,
}

/// A clock prescaler ratio
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MultiplicationRatio {
    /// Divide the input clock by the given ratio
    DivideBy(u8),
    /// Multiply the input clock by the given ratio
    MultiplyBy(u8),
}

impl From<u8> for MultiplicationRatio {
    /// Parse a byte encoded ratio
    ///
    /// # Examples
    /// ```
    /// use rxprog::command::data::MultiplicationRatio;
    ///
    /// assert_eq!(MultiplicationRatio::from(0xFF), MultiplicationRatio::DivideBy(1));
    /// assert_eq!(MultiplicationRatio::from(0xFE), MultiplicationRatio::DivideBy(2));
    /// assert_eq!(MultiplicationRatio::from(0x01), MultiplicationRatio::MultiplyBy(1));
    /// assert_eq!(MultiplicationRatio::from(0x02), MultiplicationRatio::MultiplyBy(2));
    /// ```
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
    /// Convert the ratio to a byte, where division is represented as the negative ratio in 2's
    /// complement
    ///
    /// # Examples
    /// ```
    /// use rxprog::command::data::MultiplicationRatio;
    ///
    /// assert_eq!(u8::from(MultiplicationRatio::DivideBy(1)), 0xFF);
    /// assert_eq!(u8::from(MultiplicationRatio::DivideBy(2)), 0xFE);
    /// assert_eq!(u8::from(MultiplicationRatio::MultiplyBy(1)), 0x01);
    /// assert_eq!(u8::from(MultiplicationRatio::MultiplyBy(2)), 0x02);
    /// ```
    fn from(item: MultiplicationRatio) -> Self {
        match item {
            MultiplicationRatio::DivideBy(ratio) => -(ratio as i8) as u8,
            MultiplicationRatio::MultiplyBy(ratio) => ratio as u8,
        }
    }
}

/// A distinct region of memory
#[derive(Debug)]
pub enum MemoryArea {
    /// User boot area, i.e. user specified bootloader
    UserBootArea,
    /// User area, i.e. typical application code and data
    UserArea,
}

/// State of the block
#[derive(Debug, PartialEq)]
pub enum ErasureState {
    /// No blocks programmed
    Blank,
    /// One or more blocks programmed
    NotBlank,
}

/// The state of the lock bit protecting a memory region
#[derive(Debug, PartialEq)]
pub enum LockBitStatus {
    /// Lock bit set - write/erase disallowed
    Locked,
    /// Lock bit not set - write/erase allowed
    Unlocked,
}
