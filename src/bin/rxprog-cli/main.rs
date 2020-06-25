extern crate clap;
extern crate rxprog;
extern crate serialport;

mod connection_string;
mod image;

use std::cmp;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::iter;
use std::path::Path;
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

fn list_ports() -> Result<(), CLIError> {
    let ports =
        serialport::available_ports().map_err(|_| "could not retrieve list of available ports")?;
    print_table(
        vec!["Port name"],
        ports
            .iter()
            .map(|port| vec![port.port_name.as_str()])
            .collect::<Vec<_>>(),
    );

    Ok(())
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

enum CLIError {
    Message(String),
    Programmer(rxprog::Error),
    IO(std::io::Error),
    SerialPort(serialport::Error),
}

impl From<&str> for CLIError {
    fn from(s: &str) -> CLIError {
        CLIError::Message(s.to_string())
    }
}

impl From<String> for CLIError {
    fn from(s: String) -> CLIError {
        CLIError::Message(s)
    }
}

impl From<rxprog::Error> for CLIError {
    fn from(e: rxprog::Error) -> CLIError {
        CLIError::Programmer(e)
    }
}

impl From<std::io::Error> for CLIError {
    fn from(e: std::io::Error) -> CLIError {
        CLIError::IO(e)
    }
}

impl From<serialport::Error> for CLIError {
    fn from(e: serialport::Error) -> CLIError {
        CLIError::SerialPort(e)
    }
}

enum ImageType {
    IHEX,
    SREC,
}

impl ImageType {
    fn from_arg(s: &str) -> ImageType {
        match s {
            "ihex" => ImageType::IHEX,
            "srec" => ImageType::SREC,
            _ => unreachable!(),
        }
    }

    fn from_extension(extension: Option<&OsStr>) -> Option<ImageType> {
        match extension {
            Some(extension) => match extension.to_str() {
                Some(extension) => match extension {
                    "hex" | "ihex" | "ihx" => Some(ImageType::IHEX),
                    "srec" | "mot" => Some(ImageType::SREC),
                    _ => None,
                },
                None => None,
            },
            None => None,
        }
    }
}

impl fmt::Display for ImageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ImageType::IHEX => "ihex",
                ImageType::SREC => "srec",
            }
        )
    }
}

fn main2() -> Result<(), CLIError> {
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
                .help("A semicolon (;) separated list of key=value pairs specifying the required configuration options to connect to a target"),
        )
        .arg(Arg::with_name("image_path").index(2))
        .arg(Arg::with_name("image_type").long("image-type").short("T").value_name("IMAGE_TYPE").help("The type of the image file").possible_values(&["ihex", "srec"]).takes_value(true))
        .long_about("Programming utility for Renesas microcontrollers supporting the Boot Mode protocol\n\
\n\
The connection to the target is specified by way of a connection string. This connection string specifies the serial port (p), device (d), clock mode (cm), input frequency (if), multiplication ratios (mr), and bit rate (br) required by the Boot Mode protocol.\n\
\n\
It is recommended to surround the connection string in double quotes (\"\") to ensure it is passed to rxprog-cli correctly.\n\
\n\
Though all fields are required to successfully program a target, omitting any field will query the target for the available values for the missing field. This can be used to successively build a connection string, beginning with an empty string to show all available serial ports and finishing by specifying if, mr and br.\n\
\n\
Each query shows the fields of the connection string which need to be populated to progress.\n\
\n\
For example, to connect to a target on COM3 to query the available devices:\n\
\trxprog-cli \"p=COM3\"\n\
To connect to a target on /dev/ttyS4 and program an image:\n\
\trxprog-cli \"p=/dev/ttyS4;d=7805;cm=0;if=3200;mr=x1,x1;br=115200\" image.ihex\n\
\n\
rxprog-cli will attempt to guess the format of the image based on its extension. If the image has a non-standard extension, the image type can be specified explicitly with -T.\n")
        .about("Programming utility for Renesas microcontrollers supporting the Boot Mode protocol")
        .get_matches();

    // An empty connection string is valid and simply parsed as having no
    // key/value pairs. Since not specifying a connection string and not
    // specifying a port within the connection string have the same behaviour,
    // we're OK to specify a default
    let connection_string = matches.value_of("connection_string").unwrap_or("");
    let connection_string = ConnectionString::try_from(connection_string)
        .map_err(|e| format!("could not parse connection string ({})", e))?;

    let port = connection_string.get("p");
    if port.is_none() {
        println!("No port specified in connection string. Listing available serial ports:");
        list_ports()?;

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
        .map_err(|_| "invalid clock mode")?;

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
    let bit_rate = bit_rate
        .unwrap()
        .parse::<u32>()
        .map_err(|_| "invalid bit rate")?;
    if bit_rate % 100 != 0 {
        return Err("bit rate must be a multiple of 100".into());
    }
    let input_frequency = input_frequency
        .unwrap()
        .parse::<u16>()
        .map_err(|_| "invalid input frequency")?;
    let multiplication_ratios = multiplication_ratios
        .unwrap()
        .split(',')
        .map(|mrs| {
            // A multiplication ratio must at least be a 'x' or '/' followed by
            // one digit, so anything shorter than two characters must be
            // invalid. Also stops the `split_at()` and `next().unwrap()` calls
            // from panicking if the string is too short.
            if mrs.len() < 2 {
                return Err(());
            }

            let (c, ratio) = mrs.split_at(1);
            let c = c.chars().next().unwrap();
            let ratio = ratio.parse::<u8>().map_err(|_| ())?;

            match c {
                'x' => Ok(MultiplicationRatio::MultiplyBy(ratio)),
                '/' => Ok(MultiplicationRatio::DivideBy(ratio)),
                _ => Err(()),
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "invalid multiplication ratio")?;

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
    let image_string = fs::read_to_string(image_path)?;

    let image_type = matches
        .value_of("image_type")
        .map(ImageType::from_arg)
        .or_else(|| {
            let image_type = ImageType::from_extension(Path::new(image_path).extension());

            // If we guessed the type of the image from the extension, tell the
            // user. We could totally be wrong!
            if let Some(image_type) = &image_type {
                println!("Detected {} image from extension", image_type);
            }

            image_type
        })
        .ok_or("could not determine image type (hint: specify explicitly with -T)")?;

    let mut image = Image::new(&prog.user_area()?);
    match image_type {
        ImageType::IHEX => {
            let reader = ihex::Reader::new(image_string.as_str());
            image
                .add_data_from_ihex(reader)
                .map_err(|e| format!("failed to parse ihex ({})", e))?;
        }
        ImageType::SREC => {
            let records = srec::reader::read_records(image_string.as_str());
            image
                .add_data_from_srec(records)
                .map_err(|e| format!("failed to parse srec ({})", e))?;
        }
    }

    let prog = prog.programming_erasure_state_transition()?;

    println!("Transitioned to programming/erasure state successfully");
    println!();

    println!("Programming...");
    let mut prog = prog.program_user_or_data_area()?;
    for block in image.programmable_blocks(256) {
        let mut data = [0u8; 256];
        data.copy_from_slice(&block.data);
        prog.program_block(block.start_address, data)?;
    }
    let mut prog = prog.end()?;
    println!("Programming complete.");

    println!("Verifying...");
    let mut verification_failed = false;
    for block in image.programmable_blocks(256) {
        let programmed_data = prog.read_memory(
            MemoryArea::UserArea,
            block.start_address,
            block.data.len() as u32,
        )?;

        if programmed_data != block.data {
            verification_failed = true;

            println!(
                "Verify: block of {:#X} bytes at {:#X} did not match",
                block.data.len(),
                block.start_address
            );
        }
    }

    if !verification_failed {
        println!("Verification complete.");
    } else {
        println!("Verification failed.");
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

fn main() {
    match main2() {
        Ok(_) => {}
        Err(CLIError::Message(s)) => println!("Error: {}", s),
        Err(CLIError::Programmer(e)) => println!("Programmer error: {}", e),
        Err(CLIError::IO(e)) => println!("IO error: {}", e),
        Err(CLIError::SerialPort(e)) => println!("Serial error: {}", e),
    }
}
