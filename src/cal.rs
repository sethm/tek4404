use crate::bus::*;

use std::ops::RangeInclusive;

pub struct Calendar {}

impl Calendar {
    pub fn new() -> Self {
        Calendar {}
    }
}

impl IoDevice for Calendar {
    fn range(&self) -> RangeInclusive<usize> {
        CAL_START..=CAL_END
    }

    fn read_8(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u8, crate::err::BusError> {
        Ok(0)
    }

    fn read_16(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u16, crate::err::BusError> {
        Ok(0)
    }

    fn read_32(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u32, crate::err::BusError> {
        Ok(0)
    }

    fn write_8(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u8,
    ) -> Result<(), crate::err::BusError> {
        Ok(())
    }

    fn write_16(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u16,
    ) -> Result<(), crate::err::BusError> {
        Ok(())
    }

    fn write_32(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u32,
    ) -> Result<(), crate::err::BusError> {
        Ok(())
    }

    fn load(self: &mut Self, _data: &Vec<u8>) {}
}
