use crate::bus::*;
use crate::err::BusError;

use std::ops::RangeInclusive;

pub struct Mmu {}

impl Mmu {
    pub fn new() -> Self {
        Mmu {}
    }
}

impl IoDevice for Mmu {
    fn range(&self) -> RangeInclusive<usize> {
        MMU_START..=MMU_END
    }

    fn read_8(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        info!("(READ 8) addr={:08x}", address);
        Ok(0)
    }

    fn read_16(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u16, BusError> {
        info!("(READ 16) addr={:08x}", address);
        Ok(0)
    }

    fn read_32(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u32, BusError> {
        info!("(READ 32) addr={:08x}", address);
        Ok(0)
    }

    fn write_8(self: &mut Self, _bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        info!("(WRITE 8) addr={:08x} val={:02x}", address, value);
        Ok(())
    }

    fn write_16(
        self: &mut Self,
        _bus: &mut Bus,
        address: usize,
        value: u16,
    ) -> Result<(), BusError> {
        info!("(WRITE 16) addr={:08x} val={:04x}", address, value);
        Ok(())
    }

    fn write_32(
        self: &mut Self,
        _bus: &mut Bus,
        address: usize,
        value: u32,
    ) -> Result<(), BusError> {
        info!("(WRITE 32) addr={:08x} val={:08x}", address, value);
        Ok(())
    }
}
