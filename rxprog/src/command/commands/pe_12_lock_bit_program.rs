use super::command_impl_prelude::*;

/// Programs the lock bit of a memory region in the specified area
#[derive(Debug)]
pub struct LockBitProgram {
    /// The area in which the address resides
    pub area: MemoryArea,
    /// Bits 15 to 8 of the address
    pub a15_to_a8: u8,
    /// Bits 23 to 16 of the address
    pub a23_to_a16: u8,
    /// Bits 31 to 24 of the address
    pub a31_to_a24: u8,
}

impl TransmitCommandData for LockBitProgram {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x77,
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

impl Receive for LockBitProgram {
    type Response = ();

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, CommandError>> {
        let mut reader = ResponseReader::<_, SimpleResponse, WithError>::new(
            p,
            ResponseFirstByte::Byte(0x06),
            ErrorFirstByte(0xF7),
        );

        let response = reader.read_response()?;

        Ok(match response {
            Ok(_) => Ok(()),
            Err(error_code) => Err(match error_code {
                0x11 => CommandError::Checksum,
                0x2A => CommandError::Address,
                0x53 => CommandError::Programming,
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
        let cmd = LockBitProgram {
            area: MemoryArea::UserArea,
            a15_to_a8: 0x00,
            a23_to_a16: 0xAA,
            a31_to_a24: 0xFF,
        };
        let command_bytes = [0x77, 0x04, 0x01, 0x00, 0xAA, 0xFF, 0xDB];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_success() {
        let cmd = LockBitProgram {
            area: MemoryArea::UserArea,
            a15_to_a8: 0x00,
            a23_to_a16: 0xAA,
            a31_to_a24: 0xFF,
        };
        let response_bytes = [0x06];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Ok(()));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_fail() {
        let cmd = LockBitProgram {
            area: MemoryArea::UserArea,
            a15_to_a8: 0x00,
            a23_to_a16: 0xAA,
            a31_to_a24: 0xFF,
        };
        let response_bytes = [0xF7, 0x2A];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(response, Err(CommandError::Address));
        assert!(is_script_complete(&mut p));
    }
}
