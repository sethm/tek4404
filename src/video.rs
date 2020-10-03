//! Video controller
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
use crate::err::*;

use std::ops::RangeInclusive;
use std::result::Result;

pub struct Video {}

impl Video {
    pub fn new() -> Self {
        Video {}
    }
}

impl IoDevice for Video {
    fn range(&self) -> RangeInclusive<usize> {
        VIDEO_START..=VIDEO_END
    }

    fn read_8(&mut self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        info!("Read 8 (address={:08x})", address);
        Ok(0)
    }

    fn read_16(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u16, BusError> {
        info!("Read 16 (address={:08x})", address);
        Ok(0)
    }

    fn read_32(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u32, BusError> {
        info!("Read 32 (address={:08x})", address);
        Ok(0)
    }

    fn write_8(self: &mut Self, _bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        info!("Write 8 (address={:08x} value={:02x})", address, value);
        Ok(())
    }

    fn write_16(
        self: &mut Self,
        _bus: &mut Bus,
        address: usize,
        value: u16,
    ) -> Result<(), BusError> {
        info!("Write 16 (address={:08x} value={:04x})", address, value);
        Ok(())
    }

    fn write_32(
        self: &mut Self,
        _bus: &mut Bus,
        address: usize,
        value: u32,
    ) -> Result<(), BusError> {
        info!("Write 32 (address={:08x} value={:08x})", address, value);
        Ok(())
    }
}
