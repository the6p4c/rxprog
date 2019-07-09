extern crate rxprog;
extern crate serialport;

use std::io;
use std::time;

use rxprog::programmer::Programmer;
use serialport::prelude::*;

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

    let prog = Programmer::new(p);
    let mut prog = prog.connect()?.expect("Could not connect");

    let devices = prog.supported_devices()?;
    println!("Received devices {:?}", devices);

    let device_code = &devices[0].device_code;
    let mut prog = prog
        .select_device(&device_code)?
        .expect("Could not select device");

    let clock_modes = prog.clock_modes()?;
    println!("Received clock modes {:?}", clock_modes);

    let clock_mode = clock_modes[0];
    let mut prog = prog
        .select_clock_mode(clock_mode)?
        .expect("Could not select clock mode");

    let multiplication_ratios = prog.multiplication_ratios()?;
    println!("Received multiplication ratios {:?}", multiplication_ratios);

    let operating_frequencies = prog.operating_frequencies()?;
    println!("Received operating frequencies {:?}", operating_frequencies);

    let mut _prog = prog
        .set_new_bit_rate(
            384,
            3200,
            vec![multiplication_ratios[0][0], multiplication_ratios[1][0]],
        )?
        .expect("Could not set new bit rate");

    println!("Connected after bit rate selection");

    Ok(())
}
