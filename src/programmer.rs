use std::io;
use std::thread;
use std::time;

use crate::command;

#[derive(Debug)]
pub enum ConnectError {
    NoResponse,
    BadResponse,
    Failed,
}

pub struct Programmer {
    p: Box<dyn serialport::SerialPort>,
}

impl Programmer {
    pub fn new(p: Box<dyn serialport::SerialPort>) -> Programmer {
        Programmer { p: p }
    }

    pub fn connect(mut self) -> io::Result<Result<ProgrammerConnected, ConnectError>> {
        self.p.clear(serialport::ClearBuffer::All)?;

        let mut attempts = 0;
        while self.p.bytes_to_read()? < 1 && attempts < 30 {
            self.p.write(&[0x00])?;
            thread::sleep(time::Duration::from_millis(10));

            attempts += 1;
        }

        if attempts >= 30 {
            return Ok(Err(ConnectError::NoResponse));
        }

        let mut response1 = [0u8; 1];
        self.p.read_exact(&mut response1)?;
        let response1 = response1[0];

        if response1 != 0x00 {
            return Ok(Err(ConnectError::BadResponse));
        }

        self.p.write(&[0x55])?;

        let mut response2 = [0u8; 1];
        self.p.read_exact(&mut response2)?;
        let response2 = response2[0];

        Ok(match response2 {
            0xE6 => Ok(ProgrammerConnected(self)),
            0xFF => Err(ConnectError::Failed),
            _ => Err(ConnectError::BadResponse),
        })
    }
}

pub struct ProgrammerConnected(Programmer);

impl ProgrammerConnected {
    pub fn execute<T: command::Command>(
        &mut self,
        cmd: &T,
    ) -> io::Result<Result<T::Response, T::Error>> {
        cmd.execute(&mut self.0.p)
    }
}
