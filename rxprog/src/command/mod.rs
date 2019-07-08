use std::io;

mod command;
pub mod commands;
pub mod data;
mod reader;

pub use command::Command;

fn all_read<T: io::Read>(p: &mut T) -> bool {
    let mut buf = [0u8; 1];
    p.read(&mut buf).unwrap() == 0
}
