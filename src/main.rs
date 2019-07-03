use std::num::Wrapping;

trait Command {
    fn opcode(&self) -> u8;
    fn has_size_field(&self) -> bool;
    fn payload(&self) -> Vec<u8>;

    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        let payload = self.payload();
        let payload_size = payload.len();

        bytes.push(self.opcode());

        if self.has_size_field() {
            bytes.push(payload_size as u8);
        }

        bytes.extend(self.payload());

        if payload_size != 0 {
            let sum = bytes.iter().map(|x| Wrapping(*x)).sum::<Wrapping<u8>>().0;
            let checksum = !sum + 1;
            bytes.push(checksum);
        }

        bytes
    }
}

enum CommandInquirySelection {
    SupportedDeviceInquiry,
    DeviceSelection {
        device_code: u32,
    },
    ClockModeInquiry,
    ClockModeSelection {
        mode: u8,
    },
    MultiplicationRatioInquiry,
    OperatingFrequencyInquiry,
    NewBitRateSelection {
        bit_rate: u16,
        input_frequency: u16,
        clock_type_count: u8,
        multiplication_ratio_1: u8,
        multiplication_ratio_2: u8,
    },
    ProgrammingErasureStateTransition,
}

impl Command for CommandInquirySelection {
    fn opcode(&self) -> u8 {
        use CommandInquirySelection::*;

        match self {
            SupportedDeviceInquiry => 0x20,
            DeviceSelection { .. } => 0x10,
            ClockModeInquiry => 0x21,
            ClockModeSelection { .. } => 0x11,
            MultiplicationRatioInquiry => 0x22,
            OperatingFrequencyInquiry => 0x23,
            NewBitRateSelection { .. } => 0x3F,
            ProgrammingErasureStateTransition => 0x40,
        }
    }

    fn has_size_field(&self) -> bool {
        use CommandInquirySelection::*;

        match self {
            SupportedDeviceInquiry => false,
            DeviceSelection { .. } => true,
            ClockModeInquiry => false,
            ClockModeSelection { .. } => true,
            MultiplicationRatioInquiry => false,
            OperatingFrequencyInquiry => false,
            NewBitRateSelection { .. } => true,
            ProgrammingErasureStateTransition => false,
        }
    }

    fn payload(&self) -> Vec<u8> {
        use CommandInquirySelection::*;

        match self {
            DeviceSelection { device_code } => {
                // TODO: Check endianness
                device_code.to_le_bytes().to_vec()
            }
            ClockModeSelection { mode } => vec![*mode],
            NewBitRateSelection {
                bit_rate,
                input_frequency,
                clock_type_count,
                multiplication_ratio_1,
                multiplication_ratio_2,
            } => {
                let mut payload = vec![];
                // TODO: Check endianness
                payload.extend_from_slice(&bit_rate.to_le_bytes());
                payload.extend_from_slice(&input_frequency.to_le_bytes());
                payload.push(*clock_type_count);
                payload.push(*multiplication_ratio_1);
                payload.push(*multiplication_ratio_2);
                payload
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_device_inquiry() {
        let cmd = CommandInquirySelection::SupportedDeviceInquiry;

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x20]);
    }

    #[test]
    fn test_device_selection() {
        let cmd = CommandInquirySelection::DeviceSelection {
            device_code: 0x12345678,
        };

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x10, 0x04, 0x78, 0x56, 0x34, 0x12, 0xD8]);
    }

    #[test]
    fn test_clock_mode_inquiry() {
        let cmd = CommandInquirySelection::ClockModeInquiry;

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x21]);
    }

    #[test]
    fn test_clock_mode_selection() {
        let cmd = CommandInquirySelection::ClockModeSelection { mode: 0xAB };

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x11, 0x01, 0xAB, 0x43]);
    }

    #[test]
    fn test_multiplication_ratio_inquiry() {
        let cmd = CommandInquirySelection::MultiplicationRatioInquiry;

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x22]);
    }

    #[test]
    fn test_operating_frequency_inquiry() {
        let cmd = CommandInquirySelection::OperatingFrequencyInquiry;

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x23]);
    }

    #[test]
    fn test_new_bit_rate_selection() {
        let cmd = CommandInquirySelection::NewBitRateSelection {
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
        let cmd = CommandInquirySelection::ProgrammingErasureStateTransition;

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x40]);
    }
}

fn main() {
    println!("Hello, world!");
}
