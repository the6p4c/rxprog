use std::convert::Infallible;
use std::io;
use std::num::Wrapping;
use std::str;

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

pub trait Command {
    type Response;
    type Error;

    fn bytes(&self) -> Vec<u8>;

    // TODO: Consider correct return type for proper error handling
    fn tx<T: io::Write>(&self, p: &mut T) {
        p.write(&self.bytes());
        p.flush();
    }

    // TODO: Consider correct return type for proper error handling
    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error>;
}

struct SupportedDeviceInquiry {}

#[derive(Debug, PartialEq)]
struct SupportedDevice {
    device_code: u32,
    series_name: String,
}

#[derive(Debug, PartialEq)]
struct SupportedDeviceInquiryResponse {
    devices: Vec<SupportedDevice>,
}

impl Command for SupportedDeviceInquiry {
    type Response = SupportedDeviceInquiryResponse;
    type Error = Infallible;

    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x20,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        assert_eq!(b1, 0x30);

        let mut _size = [0u8; 1];
        p.read(&mut _size);

        let mut device_count = [0u8; 1];
        p.read(&mut device_count);
        let device_count = device_count[0];

        let mut use_le = true;
        let mut devices: Vec<SupportedDevice> = vec![];
        for _ in 0..device_count {
            let mut character_count = [0u8; 1];
            p.read(&mut character_count);
            let character_count = character_count[0];

            let series_name_character_count = character_count - 4;

            let mut device_code_bytes = [0u8; 4];
            p.read(&mut device_code_bytes);

            let mut series_name_bytes = vec![0u8; series_name_character_count as usize];
            p.read(&mut series_name_bytes);

            let device_code = if use_le {
                u32::from_le_bytes(device_code_bytes)
            } else {
                u32::from_be_bytes(device_code_bytes)
            };

            use_le = !use_le;

            devices.push(SupportedDevice {
                device_code: device_code,
                series_name: str::from_utf8(&series_name_bytes)
                    .expect("Could not decode series name")
                    .to_string(),
            });
        }

        Ok(SupportedDeviceInquiryResponse { devices: devices })
    }
}

struct DeviceSelection {
    device_code: u32,
}

impl Command for DeviceSelection {
    type Response = ();
    type Error = u8;

    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x10,
            has_size_field: true,
            // TODO: Check endianness
            payload: self.device_code.to_le_bytes().to_vec(),
        }
        .bytes()
    }

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        match b1 {
            0x06 => Ok(()),
            0x90 => {
                let mut b2 = [0u8; 1];
                p.read(&mut b2);
                let b2 = b2[0];

                Err(b2)
            }
            _ => panic!("Invalid response received"),
        }
    }
}

struct ClockModeInquiry {}

impl Command for ClockModeInquiry {
    type Response = u8;
    type Error = Infallible;

    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x21,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        panic!("Datasheet unclear - test on real device");
    }
}

struct ClockModeSelection {
    mode: u8,
}

impl Command for ClockModeSelection {
    type Response = ();
    type Error = u8;

    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x11,
            has_size_field: true,
            payload: vec![self.mode],
        }
        .bytes()
    }

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        match b1 {
            0x06 => Ok(()),
            0x91 => {
                let mut b2 = [0u8; 1];
                p.read(&mut b2);
                let b2 = b2[0];

                Err(b2)
            }
            _ => panic!("Invalid response received"),
        }
    }
}

struct MultiplicationRatioInquiry {}

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

#[derive(Debug, PartialEq)]
struct MultiplicationRatioInquiryResponse {
    clock_types: Vec<Vec<MultiplicationRatio>>,
}

impl Command for MultiplicationRatioInquiry {
    type Response = MultiplicationRatioInquiryResponse;
    type Error = Infallible;

    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x22,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        assert_eq!(b1, 0x32);

        let mut _size = [0u8; 1];
        p.read(&mut _size);

        let mut clock_type_count = [0u8; 1];
        p.read(&mut clock_type_count);
        let clock_type_count = clock_type_count[0];

        let mut clock_types: Vec<Vec<MultiplicationRatio>> = vec![];
        for _ in 0..clock_type_count {
            let mut multiplication_ratio_count = [0u8; 1];
            p.read(&mut multiplication_ratio_count);
            let multiplication_ratio_count = multiplication_ratio_count[0];

            let mut multiplication_ratios = vec![0u8; multiplication_ratio_count as usize];
            p.read(&mut multiplication_ratios);

            clock_types.push(
                multiplication_ratios
                    .iter()
                    .map(|x| MultiplicationRatio::from(*x))
                    .collect(),
            );
        }

        Ok(MultiplicationRatioInquiryResponse {
            clock_types: clock_types,
        })
    }
}

struct OperatingFrequencyInquiry {}

#[derive(Debug, PartialEq)]
struct OperatingFrequencyRange {
    minimum_frequency: u16,
    maximum_frequency: u16,
}

#[derive(Debug, PartialEq)]
struct OperatingFrequencyInquiryResponse {
    clock_types: Vec<OperatingFrequencyRange>,
}

impl Command for OperatingFrequencyInquiry {
    type Response = OperatingFrequencyInquiryResponse;
    type Error = Infallible;

    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x23,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        assert_eq!(b1, 0x33);

        let mut _size = [0u8; 1];
        p.read(&mut _size);

        let mut clock_type_count = [0u8; 1];
        p.read(&mut clock_type_count);
        let clock_type_count = clock_type_count[0];

        let mut clock_types: Vec<OperatingFrequencyRange> = vec![];
        for _ in 0..clock_type_count {
            let mut minimum_frequency_bytes = [0u8; 2];
            p.read(&mut minimum_frequency_bytes);

            let mut maximum_frequency_bytes = [0u8; 2];
            p.read(&mut maximum_frequency_bytes);

            clock_types.push(OperatingFrequencyRange {
                // TODO: Check endianness
                minimum_frequency: u16::from_le_bytes(minimum_frequency_bytes),
                maximum_frequency: u16::from_le_bytes(maximum_frequency_bytes),
            });
        }

        Ok(OperatingFrequencyInquiryResponse {
            clock_types: clock_types,
        })
    }
}

struct NewBitRateSelection {
    bit_rate: u16,
    input_frequency: u16,
    clock_type_count: u8,
    multiplication_ratio_1: MultiplicationRatio,
    multiplication_ratio_2: MultiplicationRatio,
}

impl Command for NewBitRateSelection {
    type Response = ();
    type Error = Infallible;

    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x3F,
            has_size_field: true,
            payload: {
                let mut payload = vec![];
                // TODO: Check endianness
                payload.extend_from_slice(&self.bit_rate.to_le_bytes());
                payload.extend_from_slice(&self.input_frequency.to_le_bytes());
                payload.push(self.clock_type_count);
                payload.push(self.multiplication_ratio_1.into());
                payload.push(self.multiplication_ratio_2.into());
                payload
            },
        }
        .bytes()
    }

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        panic!("More complicated than tx/rx");
    }
}

struct ProgrammingErasureStateTransition {}

#[derive(Debug, PartialEq)]
enum IDCodeProtectionStatus {
    Disabled,
    Enabled,
}

impl Command for ProgrammingErasureStateTransition {
    type Response = IDCodeProtectionStatus;
    type Error = ();

    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x40,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }

    fn rx<T: io::Read>(&self, p: &mut T) -> Result<Self::Response, Self::Error> {
        let mut b1 = [0u8; 1];
        p.read(&mut b1);
        let b1 = b1[0];

        match b1 {
            0x26 => Ok(IDCodeProtectionStatus::Disabled),
            0x16 => Ok(IDCodeProtectionStatus::Enabled),
            0xC0 => {
                let mut b2 = [0u8; 1];
                p.read(&mut b2);
                let b2 = b2[0];

                assert_eq!(b2, 0x51);

                Err(())
            }
            _ => panic!("Invalid response received"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod command_inquiry_selection {
        use super::*;

        fn all_read<T: io::Read>(p: &mut T) -> bool {
            let mut buf = [0u8; 1];
            p.read(&mut buf).unwrap() == 0
        }

        #[test]
        fn test_supported_device_inquiry_tx() {
            let cmd = SupportedDeviceInquiry {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x20]);
        }

        #[test]
        fn test_supported_device_inquiry_rx() {
            let cmd = SupportedDeviceInquiry {};
            let response_bytes = vec![
                0x30, 0x13, 0x02, // Header
                0x08, 0x78, 0x56, 0x34, 0x12, 0x41, 0x42, 0x43, 0x44, // Device 1
                0x09, 0x89, 0xAB, 0xCD, 0xEF, 0x56, 0x57, 0x58, 0x59, 0x5A, // Device 2
            ];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

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

        #[test]
        fn test_device_selection_tx() {
            let cmd = DeviceSelection {
                device_code: 0x12345678,
            };

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x10, 0x04, 0x78, 0x56, 0x34, 0x12, 0xD8]);
        }

        #[test]
        fn test_device_selection_rx_success() {
            let cmd = DeviceSelection {
                device_code: 0x12345678,
            };
            let response_bytes = vec![0x06];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(response, Ok(()));
            assert!(all_read(&mut p));
        }

        #[test]
        fn test_device_selection_rx_fail() {
            let cmd = DeviceSelection {
                device_code: 0x12345678,
            };
            let response_bytes = vec![0x90, 0x21];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(response, Err(0x21));
            assert!(all_read(&mut p));
        }

        #[test]
        fn test_clock_mode_inquiry_tx() {
            let cmd = ClockModeInquiry {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x21]);
        }

        #[test]
        fn test_clock_mode_selection_tx() {
            let cmd = ClockModeSelection { mode: 0xAB };

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x11, 0x01, 0xAB, 0x43]);
        }

        #[test]
        fn test_clock_mode_selection_rx_success() {
            let cmd = ClockModeSelection { mode: 0xAB };
            let response_bytes = vec![0x06];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(response, Ok(()));
            assert!(all_read(&mut p));
        }

        #[test]
        fn test_clock_mode_selection_rx_fail() {
            let cmd = ClockModeSelection { mode: 0xAB };
            let response_bytes = vec![0x91, 0x21];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(response, Err(0x21));
            assert!(all_read(&mut p));
        }

        #[test]
        fn test_multiplication_ratio_inquiry_tx() {
            let cmd = MultiplicationRatioInquiry {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x22]);
        }

        #[test]
        fn test_multiplication_ratio_inquiry_rx() {
            let cmd = MultiplicationRatioInquiry {};
            let response_bytes = vec![
                0x32, 0x0D, 0x02, // Header
                0x04, 0xFC, 0xFE, 0x02, 0x04, // Clock type 1
                0x06, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, // Clock type 2
            ];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(
                response,
                Ok(MultiplicationRatioInquiryResponse {
                    clock_types: vec![
                        vec![
                            MultiplicationRatio::DivideBy(4),
                            MultiplicationRatio::DivideBy(2),
                            MultiplicationRatio::MultiplyBy(2),
                            MultiplicationRatio::MultiplyBy(4)
                        ],
                        vec![
                            MultiplicationRatio::MultiplyBy(1),
                            MultiplicationRatio::MultiplyBy(2),
                            MultiplicationRatio::MultiplyBy(4),
                            MultiplicationRatio::MultiplyBy(8),
                            MultiplicationRatio::MultiplyBy(16),
                            MultiplicationRatio::MultiplyBy(32)
                        ],
                    ],
                })
            );
            assert!(all_read(&mut p));
        }

        #[test]
        fn test_operating_frequency_inquiry_tx() {
            let cmd = OperatingFrequencyInquiry {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x23]);
        }

        #[test]
        fn test_operating_frequency_inquiry_rx() {
            let cmd = OperatingFrequencyInquiry {};
            let response_bytes = vec![
                0x33, 0x09, 0x02, 0xE8, 0x03, 0xD0, 0x07, 0x64, 0x00, 0x10, 0x27,
            ];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(
                response,
                Ok(OperatingFrequencyInquiryResponse {
                    clock_types: vec![
                        OperatingFrequencyRange {
                            minimum_frequency: 1000,
                            maximum_frequency: 2000
                        },
                        OperatingFrequencyRange {
                            minimum_frequency: 100,
                            maximum_frequency: 10000
                        },
                    ],
                })
            );
            assert!(all_read(&mut p));
        }

        #[test]
        fn test_new_bit_rate_selection_tx() {
            let cmd = NewBitRateSelection {
                bit_rate: 0x00C0,
                input_frequency: 0x04E2,
                clock_type_count: 0x02,
                multiplication_ratio_1: MultiplicationRatio::MultiplyBy(4),
                multiplication_ratio_2: MultiplicationRatio::DivideBy(2),
            };

            let bytes = cmd.bytes();

            assert_eq!(
                bytes,
                vec![0x3F, 0x07, 0xC0, 0x00, 0xE2, 0x04, 0x02, 0x04, 0xFE, 0x10]
            );
        }

        #[test]
        fn test_programming_erasure_state_transition_tx() {
            let cmd = ProgrammingErasureStateTransition {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x40]);
        }

        #[test]
        fn test_programming_erasure_state_transition_rx_success_id_disabled() {
            let cmd = ProgrammingErasureStateTransition {};
            let response_bytes = vec![0x26];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(response, Ok(IDCodeProtectionStatus::Disabled));
            assert!(all_read(&mut p));
        }

        #[test]
        fn test_programming_erasure_state_transition_rx_success_id_enabled() {
            let cmd = ProgrammingErasureStateTransition {};
            let response_bytes = vec![0x16];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(response, Ok(IDCodeProtectionStatus::Enabled));
            assert!(all_read(&mut p));
        }

        #[test]
        fn test_programming_erasure_state_transition_rx_fail() {
            let cmd = ProgrammingErasureStateTransition {};
            let response_bytes = vec![0xC0, 0x51];
            let mut p = mock_io::Builder::new().read(&response_bytes).build();

            let response = cmd.rx(&mut p);

            assert_eq!(response, Err(()));
            assert!(all_read(&mut p));
        }
    }
}
