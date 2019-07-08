use std::io;

pub fn is_script_complete<T: io::Read + io::Write>(mut p: T) -> bool {
    let mut buf = [0u8; 1];
    p.read(&mut buf).unwrap() == 0 && p.write(&[0x00]).is_err()
}
