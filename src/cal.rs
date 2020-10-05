//! Calendar time chip
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

    fn read_8(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
    ) -> Result<u8, crate::err::BusError> {
        Ok(0)
    }

    fn read_16(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
    ) -> Result<u16, crate::err::BusError> {
        Ok(0)
    }

    fn read_32(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
    ) -> Result<u32, crate::err::BusError> {
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

    fn load(self: &mut Self, _data: &[u8]) {}
}
