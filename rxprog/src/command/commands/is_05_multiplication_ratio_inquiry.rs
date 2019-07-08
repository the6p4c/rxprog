use std::convert::Infallible;
use std::io;

use super::command::*;
use super::data::MultiplicationRatio;
use super::reader::*;

#[derive(Debug)]
pub struct MultiplicationRatioInquiry {}

#[derive(Debug, PartialEq)]
pub struct MultiplicationRatioInquiryResponse {
    pub clock_types: Vec<Vec<MultiplicationRatio>>,
}

impl TransmitCommandData for MultiplicationRatioInquiry {
    fn command_data(&self) -> CommandData {
        CommandData {
            opcode: 0x22,
            has_size_field: false,
            payload: vec![],
        }
    }
}

impl Receive for MultiplicationRatioInquiry {
    type Response = MultiplicationRatioInquiryResponse;
    type Error = Infallible;

    fn rx<T: io::Read>(&self, p: &mut T) -> io::Result<Result<Self::Response, Self::Error>> {
        let mut reader =
            ResponseReader::<_, SizedResponse<u8>, NoError>::new(p, ResponseFirstByte::Byte(0x32));

        let data = reader.read_response()?.data;

        let clock_type_count = data[0];

        let mut clock_types: Vec<Vec<MultiplicationRatio>> = vec![];
        let mut remaining_data = &data[1..];
        for _ in 0..clock_type_count {
            let (multiplication_ratio_count, multiplication_ratios) =
                remaining_data.split_first().unwrap();
            let multiplication_ratio_count = *multiplication_ratio_count as usize;
            let multiplication_ratios = &multiplication_ratios[..multiplication_ratio_count];

            clock_types.push(
                multiplication_ratios
                    .iter()
                    .map(|x| MultiplicationRatio::from(*x))
                    .collect(),
            );

            remaining_data = &remaining_data[(1 + multiplication_ratio_count)..];
        }

        Ok(Ok(MultiplicationRatioInquiryResponse {
            clock_types: clock_types,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() -> io::Result<()> {
        let cmd = MultiplicationRatioInquiry {};
        let command_bytes = [0x22];
        let mut p = mockstream::MockStream::new();

        cmd.tx(&mut p)?;

        assert_eq!(p.pop_bytes_written(), command_bytes);

        Ok(())
    }

    #[test]
    fn test_rx() {
        let cmd = MultiplicationRatioInquiry {};
        let response_bytes = [
            0x32, 0x0D, 0x02, // Header
            0x04, 0xFC, 0xFE, 0x02, 0x04, // Clock type 1
            0x06, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, // Clock type 2
            0x76, // Checksum
        ];
        let mut p = mockstream::MockStream::new();
        p.push_bytes_to_read(&response_bytes);

        let response = cmd.rx(&mut p).unwrap();

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
