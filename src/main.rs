extern crate clap;
extern crate rxprog;
extern crate serialport;

mod image;

use std::cmp;
use std::fs;
use std::io;
use std::iter;
use std::time;

use clap::{App, Arg};
use rxprog::command::data::{MemoryArea, MultiplicationRatio};
use rxprog::programmer::{
    Programmer, ProgrammerConnected, ProgrammerConnectedClockModeSelected,
    ProgrammerConnectedDeviceSelected, ProgrammerConnectedNewBitRateSelected,
};
use rxprog::target::SerialTarget;
use serialport::prelude::*;

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

fn list_devices(prog: &mut ProgrammerConnected) -> io::Result<()> {
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

fn list_clock_modes(prog: &mut ProgrammerConnectedDeviceSelected) -> io::Result<()> {
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

fn list_multiplication_ratios(prog: &mut ProgrammerConnectedClockModeSelected) -> io::Result<()> {
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

fn list_operating_frequencies(prog: &mut ProgrammerConnectedClockModeSelected) -> io::Result<()> {
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

fn list_areas_and_blocks(prog: &mut ProgrammerConnectedNewBitRateSelected) -> io::Result<()> {
    println!("User boot area blocks");
    let rows = prog
        .user_boot_area()?
        .iter()
        .map(|r| vec![format!("0x{:x}", r.start()), format!("0x{:x}", r.end())])
        .collect::<Vec<_>>();

    print_table(
        vec!["Start address", "End address"],
        rows.iter()
            .map(|row| row.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
            .collect(),
    );

    println!();
    println!("User area blocks");
    let rows = prog
        .user_area()?
        .iter()
        .map(|r| vec![format!("0x{:x}", r.start()), format!("0x{:x}", r.end())])
        .collect::<Vec<_>>();

    print_table(
        vec!["Start address", "End address"],
        rows.iter()
            .map(|row| row.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
            .collect(),
    );

    println!();
    println!("Erasure blocks");
    let rows = prog
        .erasure_block()?
        .iter()
        .map(|r| vec![format!("0x{:x}", r.start()), format!("0x{:x}", r.end())])
        .collect::<Vec<_>>();

    print_table(
        vec!["Start address", "End address"],
        rows.iter()
            .map(|row| row.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
            .collect(),
    );

    Ok(())
}

fn main() -> io::Result<()> {
    let matches = App::new("rxprog-cli")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Serial port connected to the target")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("device")
                .short("d")
                .long("device")
                .value_name("DEVICE_CODE")
                .help("Device on the target to select")
                .takes_value(true)
                .requires("port"),
        )
        .arg(
            Arg::with_name("clock_mode")
                .short("c")
                .long("clock_mode")
                .value_name("CLOCK_MODE")
                .help("Clock mode to select")
                .takes_value(true)
                .requires("device"),
        )
        .arg(
            Arg::with_name("bit_rate")
                .short("b")
                .long("bit_rate")
                .value_name("BIT_RATE")
                .help("Bit rate for programming")
                .takes_value(true)
                .requires_all(&["input_frequency", "multiplication_ratios"]),
        )
        .arg(
            Arg::with_name("input_frequency")
                .short("f")
                .long("input_frequency")
                .value_name("INPUT_FREQUENCY")
                .help("Frequency of device clock input")
                .takes_value(true)
                .requires_all(&["bit_rate", "multiplication_ratios"]),
        )
        .arg(
            Arg::with_name("multiplication_ratios")
                .short("m")
                .long("multiplication_ratios")
                .value_name("MULTIPLICATION_RATIOS")
                .help("Multiplication ratio for each clock")
                .takes_value(true)
                .requires_all(&["bit_rate", "input_frequency"]),
        )
        .arg(Arg::with_name("image_path").index(1))
        .get_matches();

    let port = matches.value_of("port");

    if port.is_none() {
        list_ports();

        println!();
        println!("Hint: select a port with -p PORT");
        return Ok(());
    }

    let port = port.unwrap();
    let p = serialport::open_with_settings(
        port,
        &SerialPortSettings {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: time::Duration::from_millis(1000),
        },
    )?;
    let target = SerialTarget::new(p);
    let mut prog = Programmer::new(Box::new(target))
        .connect()?
        .expect("Couldn't connect to target");
    let device = matches.value_of("device");

    if device.is_none() {
        list_devices(&mut prog)?;

        println!();
        println!("Hint: select a device with -d DEVICE_CODE");
        return Ok(());
    }

    let device = device.unwrap();
    let mut prog = prog
        .select_device(&device.to_string())?
        .expect("Couldn't select device");
    let clock_mode = matches.value_of("clock_mode");

    if clock_mode.is_none() {
        list_clock_modes(&mut prog)?;

        println!();
        println!("Hint: select a clock mode with -c CLOCK_MODE");
        return Ok(());
    }

    let clock_mode = clock_mode
        .unwrap()
        .parse::<u8>()
        .expect("Invalid clock mode");
    let mut prog = prog
        .select_clock_mode(clock_mode)?
        .expect("Couldn't select clock mode");

    let bit_rate = matches.value_of("bit_rate");
    let input_frequency = matches.value_of("input_frequency");
    let multiplication_ratios = matches.value_of("multiplication_ratios");

    if bit_rate.is_none() || input_frequency.is_none() || multiplication_ratios.is_none() {
        list_multiplication_ratios(&mut prog)?;
        list_operating_frequencies(&mut prog)?;

        println!();
        println!("Hint: select an input frequency, multiplication ratio and bit rate with -f INPUT_FREQUENCY -m MULTIPLICATION_RATIOS -b BIT_RATE");
        return Ok(());
    }

    let bit_rate = bit_rate.unwrap().parse::<u16>().expect("Invalid bit rate");
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

    let mut prog = prog
        .set_new_bit_rate(bit_rate, input_frequency, multiplication_ratios)?
        .expect("Couldn't set new bit rate");

    println!("Connected with new bit rate set!");

    let image_path = matches.value_of("image_path");

    if image_path.is_none() {
        list_areas_and_blocks(&mut prog)?;

        println!();
        println!("Hint: specify an image to program the device");
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

    let mut prog = prog.programming_erasure_state_transition()?;
    let mut prog = prog.program_user_or_data_area()?;
    for block in image.programmable_blocks(256) {
        println!(
            "Programming {:#X} bytes at {:#X}",
            block.data.len(),
            block.start_address
        );

        let mut data = [0u8; 256];
        data.copy_from_slice(&block.data);
        prog.program_block(block.start_address, data)?
            .expect("Could not program block");
    }
    let mut prog = prog.end()?;

    for block in image.programmable_blocks(256) {
        println!(
            "Verifying {:#X} bytes at {:#X}",
            block.data.len(),
            block.start_address
        );

        let programmed_data = prog
            .read_memory(
                MemoryArea::UserArea,
                block.start_address,
                block.data.len() as u32,
            )?
            .expect("Could not read block");

        if programmed_data == block.data {
            println!("Verified");
        } else {
            println!("Falied to verify");
        }
    }

    Ok(())
}
