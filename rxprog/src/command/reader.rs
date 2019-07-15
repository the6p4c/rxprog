use std::io;
use std::marker::PhantomData;

pub trait ResponseBody: Sized {
    fn read_body<T: io::Read>(p: &mut T, first_byte: u8) -> io::Result<Self>;
}

#[derive(Debug, PartialEq)]
pub struct SimpleResponse {
    pub first_byte: u8,
}

impl ResponseBody for SimpleResponse {
    fn read_body<T: io::Read>(_p: &mut T, first_byte: u8) -> io::Result<SimpleResponse> {
        Ok(SimpleResponse {
            first_byte: first_byte,
        })
    }
}

pub trait ResponseSize {
    fn read_size<T: io::Read>(p: &mut T) -> io::Result<usize>;
}

impl ResponseSize for u8 {
    fn read_size<T: io::Read>(p: &mut T) -> io::Result<usize> {
        let mut size = [0u8; 1];
        p.read_exact(&mut size)?;
        let size = size[0] as usize;

        Ok(size)
    }
}

impl ResponseSize for u16 {
    fn read_size<T: io::Read>(p: &mut T) -> io::Result<usize> {
        let mut size = [0u8; 2];
        p.read_exact(&mut size)?;
        let size = u16::from_be_bytes(size) as usize;

        Ok(size)
    }
}

impl ResponseSize for u32 {
    fn read_size<T: io::Read>(p: &mut T) -> io::Result<usize> {
        let mut size = [0u8; 4];
        p.read_exact(&mut size)?;
        let size = u32::from_be_bytes(size) as usize;

        Ok(size)
    }
}

#[derive(Debug, PartialEq)]
pub struct SizedResponse<T: ResponseSize> {
    pub data: Vec<u8>,

    phantom: PhantomData<T>,
}

impl<T: ResponseSize> ResponseBody for SizedResponse<T> {
    fn read_body<U: io::Read>(p: &mut U, _first_byte: u8) -> io::Result<SizedResponse<T>> {
        let size = T::read_size(p)?;

        let mut data = vec![0u8; size];
        p.read_exact(&mut data)?;

        let mut _checksum = [0u8; 1];
        p.read_exact(&mut _checksum)?;
        let _checksum = _checksum[0];

        Ok(SizedResponse {
            data: data,

            phantom: PhantomData,
        })
    }
}

pub struct NoError {}

pub struct WithError {}

pub enum ResponseFirstByte {
    Byte(u8),
    OneByteOf(Vec<u8>),
}

impl ResponseFirstByte {
    fn as_valid_bytes(self) -> Vec<u8> {
        match self {
            ResponseFirstByte::Byte(byte) => vec![byte],
            ResponseFirstByte::OneByteOf(bytes) => bytes,
        }
    }
}

pub struct ErrorFirstByte(pub u8);

pub struct ResponseReader<T: io::Read, TResponse: ResponseBody, TError> {
    p: T,
    response_first_bytes: Vec<u8>,
    error_first_byte: Option<u8>,

    phantom_1: PhantomData<TResponse>,
    phantom_2: PhantomData<TError>,
}

impl<T: io::Read, TResponse: ResponseBody> ResponseReader<T, TResponse, WithError> {
    pub fn new(
        p: T,
        response_first_byte: ResponseFirstByte,
        error_first_byte: ErrorFirstByte,
    ) -> ResponseReader<T, TResponse, WithError> {
        ResponseReader {
            p: p,
            response_first_bytes: response_first_byte.as_valid_bytes(),
            error_first_byte: Some(error_first_byte.0),

            phantom_1: PhantomData,
            phantom_2: PhantomData,
        }
    }
}

impl<T: io::Read, TResponse: ResponseBody> ResponseReader<T, TResponse, NoError> {
    pub fn new(
        p: T,
        response_first_byte: ResponseFirstByte,
    ) -> ResponseReader<T, TResponse, NoError> {
        ResponseReader {
            p: p,
            response_first_bytes: response_first_byte.as_valid_bytes(),
            error_first_byte: None,

            phantom_1: PhantomData,
            phantom_2: PhantomData,
        }
    }
}

impl<T: io::Read, TResponse: ResponseBody, TError> ResponseReader<T, TResponse, TError> {
    fn read_first_byte(&mut self) -> io::Result<u8> {
        let mut first_byte = [0u8; 1];
        self.p.read(&mut first_byte)?;
        let first_byte = first_byte[0];

        Ok(first_byte)
    }

    fn is_valid_response_first_byte(&self, first_byte: u8) -> bool {
        self.response_first_bytes
            .iter()
            .find(|&&x| x == first_byte)
            .is_some()
    }
}

impl<T: io::Read, TResponse: ResponseBody> ResponseReader<T, TResponse, WithError> {
    fn is_valid_error_first_byte(&self, first_byte: u8) -> bool {
        first_byte == self.error_first_byte.unwrap()
    }

    pub fn read_response(&mut self) -> io::Result<Result<TResponse, u8>> {
        let first_byte = self.read_first_byte()?;

        if self.is_valid_error_first_byte(first_byte) {
            let mut error_code = [0u8; 1];
            self.p.read(&mut error_code)?;
            let error_code = error_code[0];

            return Ok(Err(error_code));
        }

        if self.is_valid_response_first_byte(first_byte) {
            return Ok(Ok(TResponse::read_body(&mut self.p, first_byte)?));
        }

        panic!("Unknown first byte");
    }
}

impl<T: io::Read, TResponse: ResponseBody> ResponseReader<T, TResponse, NoError> {
    pub fn read_response(&mut self) -> io::Result<TResponse> {
        let first_byte = self.read_first_byte()?;

        if self.is_valid_response_first_byte(first_byte) {
            return Ok(TResponse::read_body(&mut self.p, first_byte)?);
        }

        panic!("Unknown first byte");
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::is_script_complete;
    use super::*;

    macro_rules! make_test {
        (name => $n:ident, response => $r:expr, rr => $rr:expr, result => panic) => {
            #[test]
            #[should_panic]
            fn $n() {
                let mut p = mock_io::Builder::new().read(&$r).build();
                let mut rr = $rr(&mut p);

                let _response = rr.read_response();
            }
        };

        (name => $n:ident, response => $r:expr, rr => $rr:expr, result => $re:expr) => {
            #[test]
            fn $n() -> io::Result<()> {
                let mut p = mock_io::Builder::new().read(&$r).build();
                let mut rr = $rr(&mut p);

                let response = rr.read_response()?;

                assert_eq!(response, $re);
                assert!(is_script_complete(p));

                Ok(())
            }
        };
    }

    mod simple_response_with_error {
        use super::*;

        make_test!(
            name => ok,
            response => [0x20],
            rr => |p| ResponseReader::<_, SimpleResponse, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => Ok(SimpleResponse {
                first_byte: 0x20,
            })
        );

        make_test!(
            name => err,
            response => [0x30, 0xEF],
            rr => |p| ResponseReader::<_, SimpleResponse, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => Err(0xEF)
        );

        make_test!(
            name => unknown,
            response => [0x40],
            rr => |p| ResponseReader::<_, SimpleResponse, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => panic
        );
    }

    mod simple_response_no_error {
        use super::*;

        make_test!(
            name => ok,
            response => [0x20],
            rr => |p| ResponseReader::<_, SimpleResponse, NoError>::new(
                p,
                ResponseFirstByte::Byte(0x20)
            ),
            result => SimpleResponse {
                first_byte: 0x20,
            }
        );

        make_test!(
            name => unknown,
            response => [0x40],
            rr => |p| ResponseReader::<_, SimpleResponse, NoError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
            ),
            result => panic
        );
    }

    mod sized_response_u8_with_error {
        use super::*;

        make_test!(
            name => ok,
            response => [0x20, 0x02, 0x12, 0x34, 0x98],
            rr => |p| ResponseReader::<_, SizedResponse<u8>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => Ok(SizedResponse {
                data: vec![0x12, 0x34],

                phantom: PhantomData,
            })
        );

        make_test!(
            name => err,
            response => [0x30, 0xEF],
            rr => |p| ResponseReader::<_, SizedResponse<u8>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => Err(0xEF)
        );

        make_test!(
            name => unknown,
            response => [0x40, 0x02, 0x12, 0x34, 0x78],
            rr => |p| ResponseReader::<_, SizedResponse<u8>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => panic
        );
    }

    mod sized_response_u8_no_error {
        use super::*;

        make_test!(
            name => ok,
            response => [0x20, 0x02, 0x12, 0x34, 0x98],
            rr => |p| ResponseReader::<_, SizedResponse<u8>, NoError>::new(
                p,
                ResponseFirstByte::Byte(0x20)
            ),
            result => SizedResponse {
                data: vec![0x12, 0x34],

                phantom: PhantomData,
            }
        );

        make_test!(
            name => unknown,
            response => [0x40],
            rr => |p| ResponseReader::<_, SizedResponse<u8>, NoError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
            ),
            result => panic
        );
    }

    mod sized_response_u16_with_error {
        use super::*;

        make_test!(
            name => ok,
            response => [0x20, 0x00, 0x02, 0x12, 0x34, 0x98],
            rr => |p| ResponseReader::<_, SizedResponse<u16>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => Ok(SizedResponse {
                data: vec![0x12, 0x34],

                phantom: PhantomData,
            })
        );

        make_test!(
            name => err,
            response => [0x30, 0xEF],
            rr => |p| ResponseReader::<_, SizedResponse<u16>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => Err(0xEF)
        );

        make_test!(
            name => unknown,
            response => [0x40, 0x00, 0x02, 0x12, 0x34, 0x78],
            rr => |p| ResponseReader::<_, SizedResponse<u16>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => panic
        );
    }

    mod sized_response_u16_no_error {
        use super::*;

        make_test!(
            name => ok,
            response => [0x20, 0x00, 0x02, 0x12, 0x34, 0x98],
            rr => |p| ResponseReader::<_, SizedResponse<u16>, NoError>::new(
                p,
                ResponseFirstByte::Byte(0x20)
            ),
            result => SizedResponse {
                data: vec![0x12, 0x34],

                phantom: PhantomData,
            }
        );

        make_test!(
            name => unknown,
            response => [0x40],
            rr => |p| ResponseReader::<_, SizedResponse<u16>, NoError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
            ),
            result => panic
        );
    }

    mod sized_response_u32_with_error {
        use super::*;

        make_test!(
            name => ok,
            response => [0x20, 0x00, 0x00, 0x00, 0x02, 0x12, 0x34, 0x98],
            rr => |p| ResponseReader::<_, SizedResponse<u32>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => Ok(SizedResponse {
                data: vec![0x12, 0x34],

                phantom: PhantomData,
            })
        );

        make_test!(
            name => err,
            response => [0x30, 0xEF],
            rr => |p| ResponseReader::<_, SizedResponse<u32>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => Err(0xEF)
        );

        make_test!(
            name => unknown,
            response => [0x40, 0x00, 0x00, 0x00, 0x02, 0x12, 0x34, 0x78],
            rr => |p| ResponseReader::<_, SizedResponse<u32>, WithError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
                ErrorFirstByte(0x30)
            ),
            result => panic
        );
    }

    mod sized_response_u32_no_error {
        use super::*;

        make_test!(
            name => ok,
            response => [0x20, 0x00, 0x00, 0x00, 0x02, 0x12, 0x34, 0x98],
            rr => |p| ResponseReader::<_, SizedResponse<u32>, NoError>::new(
                p,
                ResponseFirstByte::Byte(0x20)
            ),
            result => SizedResponse {
                data: vec![0x12, 0x34],

                phantom: PhantomData,
            }
        );

        make_test!(
            name => unknown,
            response => [0x40],
            rr => |p| ResponseReader::<_, SizedResponse<u32>, NoError>::new(
                p,
                ResponseFirstByte::Byte(0x20),
            ),
            result => panic
        );
    }
}
