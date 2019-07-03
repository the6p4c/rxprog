use std::num::Wrapping;

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
    fn bytes(&self) -> Vec<u8>;
}

struct SupportedDeviceInquiry {}

impl Command for SupportedDeviceInquiry {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x20,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }
}

struct DeviceSelection {
    device_code: u32,
}

impl Command for DeviceSelection {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x10,
            has_size_field: true,
            // TODO: Check endianness
            payload: self.device_code.to_le_bytes().to_vec(),
        }
        .bytes()
    }
}

struct ClockModeInquiry {}

impl Command for ClockModeInquiry {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x21,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }
}

struct ClockModeSelection {
    mode: u8,
}

impl Command for ClockModeSelection {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x11,
            has_size_field: true,
            payload: vec![self.mode],
        }
        .bytes()
    }
}

struct MultiplicationRatioInquiry {}

impl Command for MultiplicationRatioInquiry {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x22,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }
}

struct OperatingFrequencyInquiry {}

impl Command for OperatingFrequencyInquiry {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x23,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }
}

struct NewBitRateSelection {
    bit_rate: u16,
    input_frequency: u16,
    clock_type_count: u8,
    multiplication_ratio_1: u8,
    multiplication_ratio_2: u8,
}

impl Command for NewBitRateSelection {
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
                payload.push(self.multiplication_ratio_1);
                payload.push(self.multiplication_ratio_2);
                payload
            },
        }
        .bytes()
    }
}

struct ProgrammingErasureStateTransition {}

impl Command for ProgrammingErasureStateTransition {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x40,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod command_inquiry_selection {
        use super::*;

        #[test]
        fn test_supported_device_inquiry() {
            let cmd = SupportedDeviceInquiry {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x20]);
        }

        #[test]
        fn test_device_selection() {
            let cmd = DeviceSelection {
                device_code: 0x12345678,
            };

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x10, 0x04, 0x78, 0x56, 0x34, 0x12, 0xD8]);
        }

        #[test]
        fn test_clock_mode_inquiry() {
            let cmd = ClockModeInquiry {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x21]);
        }

        #[test]
        fn test_clock_mode_selection() {
            let cmd = ClockModeSelection { mode: 0xAB };

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x11, 0x01, 0xAB, 0x43]);
        }

        #[test]
        fn test_multiplication_ratio_inquiry() {
            let cmd = MultiplicationRatioInquiry {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x22]);
        }

        #[test]
        fn test_operating_frequency_inquiry() {
            let cmd = OperatingFrequencyInquiry {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x23]);
        }

        #[test]
        fn test_new_bit_rate_selection() {
            let cmd = NewBitRateSelection {
                bit_rate: 0x00C0,
                input_frequency: 0x04E2,
                clock_type_count: 0x02,
                multiplication_ratio_1: 0x04,
                multiplication_ratio_2: 0xFE,
            };

            let bytes = cmd.bytes();

            assert_eq!(
                bytes,
                vec![0x3F, 0x07, 0xC0, 0x00, 0xE2, 0x04, 0x02, 0x04, 0xFE, 0x10]
            );
        }

        #[test]
        fn test_programming_erasure_state_transition() {
            let cmd = ProgrammingErasureStateTransition {};

            let bytes = cmd.bytes();

            assert_eq!(bytes, vec![0x40]);
        }
    }
}
