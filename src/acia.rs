use crate::bus;
use crate::bus::*;
use crate::err::*;
use std::ops::RangeInclusive;

const DATA_REG: usize = 0x78c000;
const STAT_REG: usize = 0x78c002;
const CMD_REG: usize = 0x78c004;
const CTRL_REG: usize = 0x78c006;

pub struct Acia {
    data: u8,
    control: u8,
    command: u8,
    status: u8,
}

impl Acia {
    pub fn new() -> Acia {
        Acia {
            data: 0,
            control: 0,
            command: 0,
            status: 0,
        }
    }

    fn handle_command(&mut self) {
        info!("ACIA: HANDLING COMMAND {:02x}", self.command);

        self.status = 0b00010000u8;
    }
}

impl IoDevice for Acia {
    fn range(&self) -> RangeInclusive<usize> {
        bus::ACIA_START..=bus::ACIA_END
    }

    fn read_8(&mut self, _: &mut Bus, address: usize) -> std::result::Result<u8, BusError> {
        let result = match address {
            DATA_REG => self.data,
            STAT_REG => self.status,
            CMD_REG => self.command,
            CTRL_REG => self.control,
            _ => 0,
        };
        Ok(result)
    }

    fn write_8(
        &mut self,
        _: &mut Bus,
        address: usize,
        data: u8,
    ) -> std::result::Result<(), BusError> {
        match address {
            DATA_REG => {
                info!(
                    "DEBUG ACIA TRANSMIT: ({})",
                    match data {
                        0x20..=0x7f => data as char,
                        _ => '.',
                    }
                );
                self.data = data;
            }
            STAT_REG => self.data = 0,
            CMD_REG => {
                self.command = data;
                self.handle_command();
            }
            CTRL_REG => self.control = data,
            _ => {}
        }
        Ok(())
    }
}
