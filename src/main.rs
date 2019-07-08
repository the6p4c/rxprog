extern crate rxprog;
extern crate serialport;

use serialport::prelude::*;
use std::io;
use std::time;

// TODO: Reorganise the rxprog crate to make this unnecessary
use rxprog::command::commands;
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
    let mut programmer = programmer.connect()?.expect("Connect failed");

    println!("Connected");

    let response = programmer.execute(&commands::SupportedDeviceInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer.execute(&commands::ClockModeInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer.execute(&commands::MultiplicationRatioInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer.execute(&commands::OperatingFrequencyInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer.execute(&commands::UserBootAreaInformationInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer.execute(&commands::UserAreaInformationInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer.execute(&commands::ErasureBlockInformationInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer.execute(&commands::ProgrammingSizeInquiry {})?;
    println!("Response: {:x?}", response);

    Ok(())
}
