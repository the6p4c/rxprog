use super::*;
use std::io;

#[derive(Debug)]
pub struct SupportedDeviceInquiry {}

#[derive(Debug, PartialEq)]
pub struct SupportedDevice {
    pub device_code: u32,
    pub series_name: String,
}

#[derive(Debug, PartialEq)]
pub struct SupportedDeviceInquiryResponse {
    pub devices: Vec<SupportedDevice>,
}

impl TransmitCommandData for SupportedDeviceInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x20,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for SupportedDeviceInquiry {
    type Response = SupportedDeviceInquiryResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let reader: ResponseReader<_, SizedResponse> = ResponseReader::new(
            p,
            ResponseFirstByte::Byte(0x30),
            ErrorResponseFirstByte::None,
        );

        let response = reader.read_response()?;

        Ok(match response {
            SizedResponse::Response(data) => {
                let device_count = data[0];

                let mut devices: Vec<SupportedDevice> = vec![];
                let mut remaining_data = &data[1..];
                for i in 0..device_count {
                    let character_count = remaining_data[0] as usize;
                    let series_name_character_count = character_count - 4;

                    let mut device_code_bytes = [0u8; 4];
                    device_code_bytes.copy_from_slice(&remaining_data[1..=4]);

                    let use_le = i % 2 == 0;
                    let device_code = if use_le {
                        u32::from_le_bytes(device_code_bytes)
                    } else {
                        u32::from_be_bytes(device_code_bytes)
                    };

                    devices.push(SupportedDevice {
                        device_code: device_code,
                        series_name: str::from_utf8(
                            &remaining_data[5..(5 + series_name_character_count)],
                        )
                        .expect("Could not decode series name")
                        .to_string(),
                    });

                    remaining_data = &remaining_data[(1 + character_count)..];
                }

                Ok(SupportedDeviceInquiryResponse { devices: devices })
            }
            SizedResponse::Error(_) => panic!("Error should not ocurr"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = SupportedDeviceInquiry {};
        let command_bytes = [0x20];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = SupportedDeviceInquiry {};
        let response_bytes = [
            0x30, 0x14, 0x02, // Header
            0x08, 0x78, 0x56, 0x34, 0x12, 0x41, 0x42, 0x43, 0x44, // Device 1
            0x09, 0x89, 0xAB, 0xCD, 0xEF, 0x56, 0x57, 0x58, 0x59, 0x5A, // Device 2
            0xE3, // Checksum
        ];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(SupportedDeviceInquiryResponse {
                devices: vec![
                    SupportedDevice {
                        device_code: 0x12345678,
                        series_name: "ABCD".to_string(),
                    },
                    SupportedDevice {
                        device_code: 0x89ABCDEF,
                        series_name: "VWXYZ".to_string(),
                    },
                ],
            })
        );
        assert!(all_read(&mut p));
    }
}
