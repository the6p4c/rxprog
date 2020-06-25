use super::command_impl_prelude::*;

/// Determine if the device has a data area
#[derive(Debug)]
pub struct DataAreaInquiry {}

impl TransmitCommandData for DataAreaInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x2A,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for DataAreaInquiry {
    type Response = DataAreaAvailability;

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x3A));

        let data = reader.read_response()?.data;

        Ok(match data[0] {
            0x18 => DataAreaAvailability::Unavailable,
            0x21 => DataAreaAvailability::Available,
            _ => panic!("Invalid availability byte"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    #[test]
    fn test_tx() -> Result<()> {
        let cmd = DataAreaInquiry {};
        let command_bytes = [0x2A];
        let mut p = mock_io::Builder::new().write(&command_bytes).build();

        cmd.tx(&mut p)?;

        assert!(is_script_complete(&mut p));

        Ok(())
    }

    #[test]
    fn test_rx_available() {
        let cmd = DataAreaInquiry {};
        let response_bytes = [0x3A, 0x01, 0x21, 0xA4];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(DataAreaAvailability::Available));
        assert!(is_script_complete(&mut p));
    }

    #[test]
    fn test_rx_unavailable() {
        let cmd = DataAreaInquiry {};
        let response_bytes = [0x3A, 0x01, 0x18, 0xAD];
        let mut p = mock_io::Builder::new().read(&response_bytes).build();

        let response = cmd.rx(&mut p);

        assert_eq!(response, Ok(DataAreaAvailability::Unavailable));
        assert!(is_script_complete(&mut p));
    }
}
