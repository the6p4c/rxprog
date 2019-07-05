extern crate serialport;

use std::io;
use std::time;
use serialport::prelude::*;

mod command;
mod programmer;

fn main() -> io::Result<()> {
    let s = SerialPortSettings {
        baud_rate: 9600,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: time::Duration::from_millis(100),
    };
    let p = serialport::open_with_settings("COM3", &s).expect("Open failed");

    let programmer = programmer::Programmer::new(p);
    let programmer = programmer.connect()?;

    match programmer {
        Ok(_) => println!("Connected"),
        Err(error) => println!("Error: {:?}", error),
    }

    Ok(())
}
