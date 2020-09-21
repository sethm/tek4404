use crate::bus;
use crate::bus::*;
use crate::err::*;
use std::ops::RangeInclusive;

pub struct Acia {
    pub char: u8,
}

impl Acia {
    pub fn new() -> Acia {
        Acia { char: 0 }
    }
}

impl IoDevice for Acia {
    fn range(&self) -> RangeInclusive<usize> {
        bus::ACIA_START..=bus::ACIA_END
    }

    fn read_8(&mut self, _: &mut Bus, address: usize) -> std::result::Result<u8, BusError> {
        info!("ACIA READ BYTE: address={:08x}", address);
        Ok(self.char)
    }

    fn read_16(&mut self, _: &mut Bus, address: usize) -> std::result::Result<u16, BusError> {
        info!("ACIA READ WORD: address={:08x}", address);
        Ok(0)
    }

    fn read_32(&mut self, _: &mut Bus, address: usize) -> std::result::Result<u32, BusError> {
        info!("ACIA READ LONG: address={:08x}", address);
        Ok(0)
    }

    fn write_8(
        &mut self,
        _: &mut Bus,
        address: usize,
        data: u8,
    ) -> std::result::Result<(), BusError> {
        info!(
            "ACIA WRITE: address={:08x} data={:02x} ({})",
            address, data, data as char
        );
        self.char = data;
        Ok(())
    }

    fn write_16(
        &mut self,
        _: &mut Bus,
        address: usize,
        data: u16,
    ) -> std::result::Result<(), BusError> {
        info!("ACIA WRITE: address={:08x} data={:04x}", address, data);
        Ok(())
    }

    fn write_32(
        &mut self,
        _: &mut Bus,
        address: usize,
        data: u32,
    ) -> std::result::Result<(), BusError> {
        info!("ACIA WRITE: address={:08x} data={:08x}", address, data);
        Ok(())
    }
}
