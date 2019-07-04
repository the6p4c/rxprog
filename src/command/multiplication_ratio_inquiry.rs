use super::*;
use std::io;

struct MultiplicationRatioInquiry {}

#[derive(Debug, PartialEq)]
struct MultiplicationRatioInquiryResponse {
    clock_types: Vec<Vec<MultiplicationRatio>>,
}

impl Transmit for MultiplicationRatioInquiry {
    fn bytes(&self) -> Vec<u8> {
        CommandData {
            opcode: 0x22,
            has_size_field: false,
            payload: vec![],
        }
        .bytes()
    }

    fn tx<T: io::Write>(&self, p: &mut T) {
        p.write(&self.bytes());
        p.flush();
    }
}

impl Receive for MultiplicationRatioInquiry {
    type Response = MultiplicationRatioInquiryResponse;
    type Error = Infallible;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() {
        let cmd = MultiplicationRatioInquiry {};

        let bytes = cmd.bytes();

        assert_eq!(bytes, vec![0x22]);
    }

    #[test]
    fn test_rx() {
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
}
