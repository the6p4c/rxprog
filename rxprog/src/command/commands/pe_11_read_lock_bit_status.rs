use super::command_impl_prelude::*;

/// Requests the state of the lock bit for a specified memory region
#[derive(Debug)]
pub struct ReadLockBitStatus {
    /// The area in which the address resides
    pub area: MemoryArea,
    /// Bits 15 to 8 of the address
    pub a15_to_a8: u8,
    /// Bits 23 to 16 of the address
    pub a23_to_a16: u8,
    /// Bits 31 to 24 of the address
    pub a31_to_a24: u8,
}

impl TransmitCommandData for ReadLockBitStatus {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x71,
            has_size_field: true,
            payload: {
                let mut payload = vec![];
                payload.push(match self.area {
                    MemoryArea::UserBootArea => 0x00,
                    MemoryArea::UserArea => 0x01,
                });
                payload.push(self.a15_to_a8);
                payload.push(self.a23_to_a16);
                payload.push(self.a31_to_a24);
                payload
            },
        }
    }
}

/// Error preventing lock bit status reading
#[derive(Debug, PartialEq)]
pub enum ReadLockBitStatusError {
    /// Command checksum validation failed
    Checksum,
    /// Address not in specified area
    Address,
}

impl Receive for ReadLockBitStatus {
    type Response = LockBitStatus;
    type Error = ReadLockBitStatusError;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::OneByteOf(vec![0x00, 0x40]),
            ErrorFirstByte(0xF1),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(SimpleResponse { first_byte }) => match first_byte {
                0x00 => Ok(LockBitStatus::Locked),
                0x40 => Ok(LockBitStatus::Unlocked),
                _ => panic!("Response with unknown first byte"),
            },
            Err(error_code) => Err(match error_code {
                0x11 => ReadLockBitStatusError::Checksum,
                0x2A => ReadLockBitStatusError::Address,
                _ => panic!("Unknown error code"),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = ReadLockBitStatus {
            area: MemoryArea::UserArea,
            a15_to_a8: 0x00,
            a23_to_a16: 0xAA,
            a31_to_a24: 0xFF,
        };
        let command_bytes = [0x71, 0x04, 0x01, 0x00, 0xAA, 0xFF, 0xE1];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_success_locked() {
        let cmd = ReadLockBitStatus {
            area: MemoryArea::UserArea,
            a15_to_a8: 0x00,
            a23_to_a16: 0xAA,
            a31_to_a24: 0xFF,
        };
        let response_bytes = [0x00];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(LockBitStatus::Locked));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_success_unlocked() {
        let cmd = ReadLockBitStatus {
            area: MemoryArea::UserArea,
            a15_to_a8: 0x00,
            a23_to_a16: 0xAA,
            a31_to_a24: 0xFF,
        };
        let response_bytes = [0x40];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(LockBitStatus::Unlocked));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = ReadLockBitStatus {
            area: MemoryArea::UserArea,
            a15_to_a8: 0x00,
            a23_to_a16: 0xAA,
            a31_to_a24: 0xFF,
        };
        let response_bytes = [0xF1, 0x2A];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(ReadLockBitStatusError::Address));
        assert!(is_script_complete(&mut p));
    }
}
