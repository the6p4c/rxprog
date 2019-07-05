use super::*;
use std::io;

#[derive(Debug)]
pub struct SupportedDeviceInquiry {}

#[derive(Debug, PartialEq)]
pub struct SupportedDevice {
    pub device_code: String,
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
                for _ in 0..device_count {
                    let (character_count, device_bytes) = remaining_data.split_first().unwrap();
                    let character_count = *character_count as usize;
                    let device_bytes = &device_bytes[..character_count];

                    let (device_code_bytes, series_name_bytes) = device_bytes.split_at(4);

                    devices.push(SupportedDevice {
                        device_code: str::from_utf8(device_code_bytes)
                            .expect("Could not decode device code")
                            .to_string(),
                        series_name: str::from_utf8(series_name_bytes)
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
            0x08, 0x44, 0x45, 0x56, 0x31, 0x41, 0x42, 0x43, 0x44, // Device 1
            0x09, 0x44, 0x45, 0x56, 0x32, 0x56, 0x57, 0x58, 0x59, 0x5A, // Device 2
            0xC6, // Checksum
        ];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

        assert_eq!(
            response,
            Ok(SupportedDeviceInquiryResponse {
                devices: vec![
                    SupportedDevice {
                        device_code: "DEV1".to_string(),
                        series_name: "ABCD".to_string(),
                    },
                    SupportedDevice {
                        device_code: "DEV2".to_string(),
                        series_name: "VWXYZ".to_string(),
                    },
                ],
            })
        );
        assert!(all_read(&mut p));
    }
}
