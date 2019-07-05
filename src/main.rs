extern crate serialport;

use serialport::prelude::*;
use std::io;
use std::time;

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
    let mut programmer = programmer.connect()?.expect("Connect failed");

    println!("Connected");

    let response =
        programmer.execute(&command::supported_device_inquiry::SupportedDeviceInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer.execute(&command::clock_mode_inquiry::ClockModeInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer
        .execute(&command::multiplication_ratio_inquiry::MultiplicationRatioInquiry {})?;
    println!("Response: {:x?}", response);

    let response =
        programmer.execute(&command::operating_frequency_inquiry::OperatingFrequencyInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer
        .execute(&command::user_boot_area_information_inquiry::UserBootAreaInformationInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer
        .execute(&command::user_area_information_inquiry::UserAreaInformationInquiry {})?;
    println!("Response: {:x?}", response);

    let response = programmer
        .execute(&command::erasure_block_information_inquiry::ErasureBlockInformationInquiry {})?;
    println!("Response: {:x?}", response);

    let response =
        programmer.execute(&command::programming_size_inquiry::ProgrammingSizeInquiry {})?;
    println!("Response: {:x?}", response);

    Ok(())
}
