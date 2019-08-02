use std::ops::RangeInclusive;

const UNPROGRAMMED_BYTE: u8 = 0xFF;

#[derive(Debug, PartialEq)]
struct Region {
    address_range: RangeInclusive<u32>,
    data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct Image {
    regions: Vec<Region>,
}

#[derive(Debug, PartialEq)]
pub struct Block<'a> {
    pub start_address: u32,
    pub data: &'a [u8],
}

impl Image {
    pub fn new(regions: &[RangeInclusive<u32>]) -> Image {
        let regions = regions
            .iter()
            .map(|address_range| {
                let length = address_range.end() - address_range.start() + 1;
                let data = vec![UNPROGRAMMED_BYTE; length as usize];

                Region {
                    address_range: address_range.clone(),
                    data,
                }
            })
            .collect::<Vec<_>>();

        Image { regions }
    }

    pub fn add_data(&mut self, address: u32, data: &[u8]) {
        let region = self
            .regions
            .iter_mut()
            .find(|region| region.address_range.contains(&address))
            .expect(format!("region containing address {} must exist", address).as_str());

        let offset = (address - region.address_range.start()) as usize;
        region.data[offset..offset + data.len()].copy_from_slice(data);
    }

    pub fn programmable_blocks(&self, block_length: usize) -> impl Iterator<Item = Block> + '_ {
        self.regions
            .iter()
            .flat_map(move |region| {
                region
                    .data
                    .chunks_exact(block_length)
                    .enumerate()
                    .map(move |(i, chunk)| {
                        let start_address =
                            *region.address_range.start() + (i * block_length) as u32;

                        Block {
                            start_address,
                            data: chunk,
                        }
                    })
            })
            .filter(|block| !block.data.iter().all(|&x| x == UNPROGRAMMED_BYTE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_empty_image() {
        let i = Image::new(&[0x0..=0xF, 0x20..=0x2F]);

        assert_eq!(
            i,
            Image {
                regions: vec![
                    Region {
                        address_range: 0x0..=0xF,
                        data: vec![UNPROGRAMMED_BYTE; 0x10]
                    },
                    Region {
                        address_range: 0x20..=0x2F,
                        data: vec![UNPROGRAMMED_BYTE; 0x10]
                    }
                ]
            }
        );
    }

    #[test]
    fn add_data_inserts_data_correctly() {
        let mut i = Image::new(&[0x0..=0xF, 0x20..=0x2F]);

        i.add_data(0x0, &[0x00, 0x11, 0x22, 0x33]);
        i.add_data(0x22, &[0x22, 0x33, 0x44, 0x55]);

        assert_eq!(
            i,
            Image {
                regions: vec![
                    Region {
                        address_range: 0x0..=0xF,
                        data: vec![
                            0x00,
                            0x11,
                            0x22,
                            0x33,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE
                        ],
                    },
                    Region {
                        address_range: 0x20..=0x2F,
                        data: vec![
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            0x22,
                            0x33,
                            0x44,
                            0x55,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE,
                            UNPROGRAMMED_BYTE
                        ],
                    }
                ]
            }
        );
    }

    #[test]
    fn programmable_blocks_empty_image_returns_empty_list() {
        let i = Image::new(&[0x0..=0xF, 0x20..=0x2F]);

        assert_eq!(i.programmable_blocks(0x2).count(), 0);
    }

    #[test]
    fn programmable_blocks_returns_correct_blocks() {
        let mut i = Image::new(&[0x0..=0xF, 0x20..=0x2F]);

        i.add_data(0x0, &[0x00, 0x11, 0x22, 0x33]);
        i.add_data(0x22, &[0x22, 0x33, 0x44, 0x55]);

        let mut pb = i.programmable_blocks(0x4);
        assert_eq!(
            pb.next(),
            Some(Block {
                start_address: 0x0,
                data: &[0x00, 0x11, 0x22, 0x33],
            })
        );
        assert_eq!(
            pb.next(),
            Some(Block {
                start_address: 0x20,
                data: &[UNPROGRAMMED_BYTE, UNPROGRAMMED_BYTE, 0x22, 0x33],
            })
        );
        assert_eq!(
            pb.next(),
            Some(Block {
                start_address: 0x24,
                data: &[0x44, 0x55, UNPROGRAMMED_BYTE, UNPROGRAMMED_BYTE],
            })
        );
        assert_eq!(pb.next(), None);
    }
}
