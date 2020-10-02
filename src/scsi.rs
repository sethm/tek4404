use crate::bus::*;
use crate::err::BusError;

use std::ops::RangeInclusive;

pub struct Scsi {}

impl Scsi {
    pub fn new() -> Self {
        Scsi {}
    }
}

impl IoDevice for Scsi {
    fn range(&self) -> RangeInclusive<usize> {
        SCSI_START..=SCSI_END
    }

    fn read_8(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u8, BusError> {
        // info!("(READ 8) addr={:08x}", _address);
        Ok(0)
    }

    fn read_16(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u16, BusError> {
        // info!("(READ 16) addr={:08x}", _address);
        Ok(0)
    }

    fn read_32(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u32, BusError> {
        // info!("(READ 32) addr={:08x}", _address);
        Ok(0)
    }

    fn write_8(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u8,
    ) -> Result<(), BusError> {
        // info!("(WRITE 8) addr={:08x} val={:02x}", _address, _value);
        Ok(())
    }

    fn write_16(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u16,
    ) -> Result<(), BusError> {
        // info!("(WRITE 16) addr={:08x} val={:04x}", _address, _value);
        Ok(())
    }

    fn write_32(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u32,
    ) -> Result<(), BusError> {
        // info!("(WRITE 32) addr={:08x} val={:08x}", _address, _value);
        Ok(())
    }
}
