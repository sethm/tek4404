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

const ADDRESS_REG: usize = 0x7bc000;

// NCR 5386 registers
const DATA_REG_1: usize = 0x7be000;
const CMND_REG: usize = 0x7be002;
const CNTRL_REG: usize = 0x7be004;
const DST_ID_REG: usize = 0x7be006;
const AUX_STAT_REG: usize = 0x7be008;
const ID_REG: usize = 0x7be00a;
const INT_REG: usize = 0x7be00c;
const SRC_ID_REG: usize = 0x7be00e;
const DATA_REG_2: usize = 0x7be010;
const DIAG_STAT_REG: usize = 0x7be012;
// 014 and 016 not used
const XFR_H_REG: usize = 0x7be018;
const XFR_M_REG: usize = 0x7be01a;
const XFR_L_REG: usize = 0x7be01c;

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

    fn read_8(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        info!("(READ 8) addr={:08x}", address);
        match address {
            DIAG_STAT_REG => Ok(0x80),
            _ => Ok(0),
        }
    }

    fn write_8(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u8,
    ) -> Result<(), BusError> {
        info!("(WRITE 8) addr={:08x} val={:02x}", _address, _value);
        Ok(())
    }
}
