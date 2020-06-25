extern crate clap;
extern crate rxprog;
extern crate serialport;

mod connection_string;
mod image;

use std::cmp;
use std::convert::TryFrom;
use std::error;
use std::fs;
use std::iter;
use std::time;

use clap::{App, Arg};
use rxprog::command::data::{MemoryArea, MultiplicationRatio};
use rxprog::programmer::{
    Programmer, ProgrammerConnected, ProgrammerConnectedClockModeSelected,
    ProgrammerConnectedDeviceSelected,
};
use rxprog::target::SerialTarget;
use serialport::prelude::*;

use connection_string::ConnectionString;
use image::Image;

fn print_table(headings: Vec<&str>, data: Vec<Vec<&str>>) {
    const COLUMN_SEPARATOR: &str = "    ";

    for row in &data {
        assert_eq!(
            row.len(),
            headings.len(),
            "Row entry count did not match heading entry count"
        );
    }

    let all_rows = iter::once(&headings).chain(data.iter());
    let col_count = headings.len();
    let col_lengths = all_rows.fold(vec![0; col_count], |acc, ss| {
        ss.iter()
            .map(|s| s.len())
            .zip(acc)
            .map(|(a, b)| cmp::max(a, b))
            .collect::<Vec<_>>()
    });

    let total_column_width = col_lengths.iter().sum::<usize>();
    let total_separator_width = (col_lengths.len() - 1) * COLUMN_SEPARATOR.len();
    let total_width = total_column_width + total_separator_width;

    let all_rows = iter::once(&headings).chain(data.iter());
    for (i, row) in all_rows.enumerate() {
        for (value, col_length) in row.iter().zip(&col_lengths) {
            print!("{0: <1$}{2}", value, col_length, COLUMN_SEPARATOR);
        }
        println!();

        if i == 0 {
            println!("{}", "=".repeat(total_width));
        }
    }
}

fn list_ports() {
    let ports = serialport::available_ports().expect("Could not retrieve list of available ports");
    print_table(
        vec!["Port name"],
        ports
            .iter()
            .map(|port| vec![port.port_name.as_str()])
            .collect::<Vec<_>>(),
    );
}

fn list_devices(prog: &mut ProgrammerConnected) -> rxprog::Result<()> {
    let devices = prog.supported_devices()?;
    print_table(
        vec!["Device code", "Series name"],
        devices
            .iter()
            .map(|device| vec![device.device_code.as_str(), device.series_name.as_str()])
            .collect::<Vec<_>>(),
    );

    Ok(())
}

fn list_clock_modes(prog: &mut ProgrammerConnectedDeviceSelected) -> rxprog::Result<()> {
    let clock_modes = prog.clock_modes()?;
    let rows = clock_modes
        .iter()
        .map(|clock_mode| vec![clock_mode.to_string()])
        .collect::<Vec<_>>();
    print_table(
        vec!["Clock mode"],
        rows.iter()
            .map(|row| row.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
            .collect(),
    );

    Ok(())
}

fn list_multiplication_ratios(
    prog: &mut ProgrammerConnectedClockModeSelected,
) -> rxprog::Result<()> {
    let multiplication_ratios = prog.multiplication_ratios()?;
    let rows = multiplication_ratios
        .iter()
        .enumerate()
        .map(|(clock, ratios)| {
            let ratios_str = ratios
                .iter()
                .map(|ratio| match ratio {
                    MultiplicationRatio::DivideBy(ratio) => format!("/{}", ratio),
                    MultiplicationRatio::MultiplyBy(ratio) => format!("x{}", ratio),
                })
                .collect::<Vec<_>>()
                .join(", ");

            vec![clock.to_string(), ratios_str]
        })
        .collect::<Vec<_>>();

    print_table(
        vec!["Clock", "Multiplication ratios"],
        rows.iter()
            .map(|row| row.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
            .collect(),
    );

    Ok(())
}

fn list_operating_frequencies(
    prog: &mut ProgrammerConnectedClockModeSelected,
) -> rxprog::Result<()> {
    let operating_frequencies = prog.operating_frequencies()?;
    let rows = operating_frequencies
        .iter()
        .enumerate()
        .map(|(clock, operating_frequency)| {
            let min_freq = operating_frequency.start();
            let max_freq = operating_frequency.end();

            vec![
                clock.to_string(),
                min_freq.to_string(),
                max_freq.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    print_table(
        vec!["Clock", "Minimum frequency", "Maximum frequency"],
        rows.iter()
            .map(|row| row.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
            .collect(),
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = App::new("rxprog-cli")
        .arg(
            Arg::with_name("show_checksums")
                .long("show-checksums")
                .short("c")
                .help("Print the checksums of the user boot and user areas after programming/verifying")
        )
        .arg(
            Arg::with_name("connection_string")
                .index(1)
                .help("A semicolon (;) separated list of key=value pairs specifiying the required configuration options to connect to a target"),
        )
        .arg(Arg::with_name("image_path").index(2))
        .get_matches();

    // An empty connection string is valid and simply parsed as having no
    // key/value pairs. Since not specifying a connection string and not
    // specifying a port within the connection string have the same behaviour,
    // we're OK to specify a default
    let connection_string = matches.value_of("connection_string").unwrap_or("");
    let connection_string = ConnectionString::try_from(connection_string);
    if let Err(e) = connection_string {
        println!("Error parsing connection string: {}", e);

        return Ok(());
    }
    let connection_string = connection_string.unwrap();

    let port = connection_string.get("p");
    if port.is_none() {
        println!("No port specified in connection string. Listing availiable serial ports:");
        list_ports();

        println!();
        println!("Hint: select a port with p=<port name>");
        return Ok(());
    }
    let port = port.unwrap();

    println!("Connecting to target on {}", port);

    let p = serialport::open_with_settings(
        port,
        &SerialPortSettings {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: time::Duration::from_millis(10_000),
        },
    )?;
    let target = SerialTarget::new(p);
    let mut prog = Programmer::new(Box::new(target)).connect()?;

    println!("Initial connection succeeded");

    let device = connection_string.get("d");
    if device.is_none() {
        println!();
        println!(
            "No device specified in connection string. Querying target for supported devices:"
        );
        list_devices(&mut prog)?;

        println!();
        println!("Hint: select a device with d=<device code>");
        return Ok(());
    }
    let device = device.unwrap();

    let mut prog = prog.select_device(&device.to_string())?;

    let clock_mode = connection_string.get("cm");
    if clock_mode.is_none() {
        println!();
        println!("No clock mode specified in connection string. Querying target for supported clock modes:");
        list_clock_modes(&mut prog)?;

        println!();
        println!("Hint: select a clock mode with cm=<clock mode>");
        return Ok(());
    }
    let clock_mode = clock_mode
        .unwrap()
        .parse::<u8>()
        .expect("Invalid clock mode");

    let mut prog = prog.select_clock_mode(clock_mode)?;

    let bit_rate = connection_string.get("br");
    let input_frequency = connection_string.get("if");
    let multiplication_ratios = connection_string.get("mr");
    if bit_rate.is_none() || input_frequency.is_none() || multiplication_ratios.is_none() {
        println!();
        println!("No input frequency, multiplication ratio and/or bit rate specified in connection string. Querying target for supported multiplication ratios and operating frequency ranges:");
        list_multiplication_ratios(&mut prog)?;
        list_operating_frequencies(&mut prog)?;

        println!();
        println!("Hint: select an input frequency, multiplication ratio and bit rate with if=<input frequency>;mr=<ratio 1>,<ratio 2>,...;br=<bit rate>");
        return Ok(());
    }
    let bit_rate = bit_rate.unwrap().parse::<u32>().expect("Invalid bit rate");
    assert!(bit_rate % 100 == 0, "Bit rate must be a multiple of 100");
    let input_frequency = input_frequency
        .unwrap()
        .parse::<u16>()
        .expect("Invalid input frequency");
    let multiplication_ratios = multiplication_ratios
        .unwrap()
        .split(',')
        .map(|mrs| {
            let (c, ratio) = mrs.split_at(1);
            let c = c.chars().next().unwrap();
            let ratio = ratio.parse::<u8>().expect("Invalid multiplication ratio");

            match c {
                'x' => MultiplicationRatio::MultiplyBy(ratio),
                '/' => MultiplicationRatio::DivideBy(ratio),
                _ => panic!("Invalid multiplication ratio"),
            }
        })
        .collect::<Vec<_>>();

    let bit_rate = (bit_rate / 100) as u16;
    let mut prog = prog.set_new_bit_rate(bit_rate, input_frequency, multiplication_ratios)?;

    let image_path = matches.value_of("image_path");
    if image_path.is_none() {
        println!();
        println!("Hint: specify an image to program the device");
        println!("Nothing to do");
        return Ok(());
    }
    let image_path = image_path.unwrap();

    let mut image = Image::new(&prog.user_area()?);
    let mut address_high = 0u16;
    for record in ihex::reader::Reader::new(fs::read_to_string(image_path)?.as_str()) {
        match record.expect("record is Ok") {
            ihex::record::Record::Data {
                offset,
                value: data,
            } => {
                let address = ((address_high as u32) << 16) | (offset as u32);
                image.add_data(address, &data);
            }
            ihex::record::Record::ExtendedLinearAddress(ela) => address_high = ela,
            _ => (),
        }
    }

    let prog = prog.programming_erasure_state_transition()?;

    println!("Transitioned to programming/erasure state successfully");
    println!();

    let mut prog = prog.program_user_or_data_area()?;
    for block in image.programmable_blocks(256) {
        println!(
            "Programming {:#X} bytes at {:#X}",
            block.data.len(),
            block.start_address
        );

        let mut data = [0u8; 256];
        data.copy_from_slice(&block.data);
        prog.program_block(block.start_address, data)?;
    }
    let mut prog = prog.end()?;

    for block in image.programmable_blocks(256) {
        println!(
            "Verifying {:#X} bytes at {:#X}",
            block.data.len(),
            block.start_address
        );

        let programmed_data = prog.read_memory(
            MemoryArea::UserArea,
            block.start_address,
            block.data.len() as u32,
        )?;

        if programmed_data == block.data {
            println!("Verified");
        } else {
            println!("Falied to verify");
        }
    }

    if matches.is_present("show_checksums") {
        let uba_checksum = prog.user_boot_area_checksum()?;
        let ua_checksum = prog.user_area_checksum()?;

        println!();
        println!("User boot area checksum: {:#010X}", uba_checksum);
        println!("User area checksum: {:#010X}", ua_checksum);
    }

    Ok(())
}
