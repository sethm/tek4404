//! Memory management unit
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

use log::debug;

pub struct Mmu {}

impl Mmu {
    pub fn new() -> Self {
        Mmu {}
    }
}

impl IoDevice for Mmu {
    fn read_8(&mut self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        debug!("(READ 8) addr={:08x}", address);
        Ok(0)
    }

    fn read_16(&mut self, _bus: &mut Bus, address: usize) -> Result<u16, BusError> {
        debug!("(READ 16) addr={:08x}", address);
        Ok(0)
    }

    fn read_32(&mut self, _bus: &mut Bus, address: usize) -> Result<u32, BusError> {
        debug!("(READ 32) addr={:08x}", address);
        Ok(0)
    }

    fn write_8(&mut self, _bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        debug!("(WRITE 8) addr={:08x} val={:02x}", address, value);
        Ok(())
    }

    fn write_16(&mut self, _bus: &mut Bus, address: usize, value: u16) -> Result<(), BusError> {
        debug!("(WRITE 16) addr={:08x} val={:04x}", address, value);
        Ok(())
    }

    fn write_32(&mut self, _bus: &mut Bus, address: usize, value: u32) -> Result<(), BusError> {
        debug!("(WRITE 32) addr={:08x} val={:08x}", address, value);
        Ok(())
    }
}
