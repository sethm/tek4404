use crate::bus;
use crate::err::*;
use crate::bus::*;

use std::ops::RangeInclusive;
use std::result::Result;


pub struct VideoControl {}

impl VideoControl {
    pub fn new() -> Self {
        VideoControl {}
    }
}

impl IoDevice for VideoControl {
    fn range(&self) -> RangeInclusive<usize> {
        bus::VIDEO_CTRL_START..=bus::VIDEO_CTRL_END
    }

    fn read_8(&mut self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        info!("Read 8 (address={:08x})", address);
        Ok(0)
    }

    fn read_16(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u16, crate::err::BusError> {
        info!("Read 16 (address={:08x})", address);
        Ok(0)
    }

    fn read_32(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u32, crate::err::BusError> {
        info!("Read 32 (address={:08x})", address);
        Ok(0)
    }

    fn write_8(
        self: &mut Self,
        _bus: &mut Bus,
        address: usize,
        value: u8,
    ) -> Result<(), crate::err::BusError> {
        info!("Write 8 (address={:08x} value={:02x})", address, value);
        Ok(())
    }

    fn write_16(
        self: &mut Self,
        _bus: &mut Bus,
        address: usize,
        value: u16,
    ) -> Result<(), crate::err::BusError> { 
        info!("Write 8 (address={:08x} value={:04x})", address, value);
        Ok(())
    }

    fn write_32(
        self: &mut Self,
        _bus: &mut Bus,
        address: usize,
        value: u32,
    ) -> Result<(), crate::err::BusError> {
        info!("Write 8 (address={:08x} value={:08x})", address, value);
        Ok(())
    }

    fn load(self: &mut Self, _data: &Vec<u8>) {}
}
