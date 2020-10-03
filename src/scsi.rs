//! SCSI controller
//
// Copyright 2020 Seth Morabito <web@loomcom.com>
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy,
// modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//
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
