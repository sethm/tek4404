/// Copyright 2020 Seth Morabito <web@loomcom.com>
///
/// Permission is hereby granted, free of charge, to any person
/// obtaining a copy of this software and associated documentation
/// files (the "Software"), to deal in the Software without
/// restriction, including without limitation the rights to use, copy,
/// modify, merge, publish, distribute, sublicense, and/or sell copies
/// of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be
/// included in all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
/// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
/// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
/// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
/// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
/// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
/// DEALINGS IN THE SOFTWARE.

use crate::cpu;
use crate::err::*;
use crate::mem::*;

use std::os::raw::c_uint;
use std::sync::{Arc, Mutex};
use std::ops::RangeInclusive;

pub const ROM_START: usize = 0x740000;
pub const ROM_END: usize = 0x747fff;

pub const RAM_START: usize = 0;
pub const RAM_END: usize = 0x1fffff;

lazy_static! {
    pub static ref BUS: Mutex<Bus> = Mutex::new(Bus::new());
}

pub type BusDevice = Arc<Mutex<dyn IoDevice + Send + Sync>>;

pub struct Bus {
    reset: bool,
    rom: BusDevice,
    ram: BusDevice,
}

pub fn reset() {
    BUS.lock().unwrap().reset();
    BUS.lock().unwrap().load_rom();
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            reset: false,
            rom: Arc::new(Mutex::new(
                Memory::new(ROM_START, ROM_END, true).expect("Unable to init ROM"),
            )),
            ram: Arc::new(Mutex::new(
                Memory::new(RAM_START, RAM_END, false).expect("Unable to init RAM"),
            )),
        }
    }

    pub fn reset(&mut self) {
        self.reset = true;
    }

    fn get_offset(&self, address: usize, range: RangeInclusive<usize>) -> Result<usize, BusError> {
        let start_addr = *range.start();
        let end_addr = *range.end();
        if self.reset {
            Ok(address % 0x8000)
        } else {
            if address < start_addr || address > end_addr {
                Err(BusError::Access)
            } else {
                Ok(address - start_addr)
            }
        }
    }

    fn get_device(
        &self,
        address: usize,
    ) -> Result<BusDevice, BusError> {
        if self.reset {
            Ok(Arc::clone(&self.rom))
        } else {
            match address {
                ROM_START..=ROM_END => Ok(Arc::clone(&self.rom)),
                RAM_START..=RAM_END => Ok(Arc::clone(&self.ram)),
                _ => Err(BusError::Access),
            }
        }
    }

    fn load_rom(&mut self) {
        let rom_data = std::fs::read("./rom/boot.bin").unwrap();
        trace!("Loaded {} bytes from ./rom/boot.bin", rom_data.len());
        self.rom.lock().unwrap().load(rom_data);
    }

    fn read_8(&self, address: usize) -> Result<u8, BusError> {
        let mutex = self.get_device(address)?;
        let dev = mutex.lock().unwrap();
        dev.read_8(self.get_offset(address, dev.range())?)
    }

    fn read_16(&self, address: usize) -> Result<u16, BusError> {
        let mutex = self.get_device(address)?;
        let dev = mutex.lock().unwrap();
        dev.read_16(self.get_offset(address, dev.range())?)
    }

    fn read_32(&self, address: usize) -> Result<u32, BusError> {
        let mutex = self.get_device(address)?;
        let dev = mutex.lock().unwrap();
        dev.read_32(self.get_offset(address, dev.range())?)
    }

    fn write_8(&self, address: usize, value: u8) -> Result<(), BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.lock().unwrap();
        let range = dev.range();
        dev.write_8(self.get_offset(address, range)?, value)
    }

    fn write_16(&self, address: usize, value: u16) -> Result<(), BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.lock().unwrap();
        let range = dev.range();
        dev.write_16(self.get_offset(address, range)?, value)
    }

    fn write_32(&self, address: usize, value: u32) -> Result<(), BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.lock().unwrap();
        let range = dev.range();
        dev.write_32(self.get_offset(address, range)?, value)
    }
}

pub trait IoDevice {
    fn load(self: &mut Self, data: Vec<u8>);
    fn range(self: &Self) -> RangeInclusive<usize>;
    fn read_8(self: &Self, offset: usize) -> Result<u8, BusError>;
    fn read_16(self: &Self, offset: usize) -> Result<u16, BusError>;
    fn read_32(self: &Self, offset: usize) -> Result<u32, BusError>;
    fn write_8(self: &mut Self, offset: usize, value: u8) -> Result<(), BusError>;
    fn write_16(self: &mut Self, offset: usize, value: u16) -> Result<(), BusError>;
    fn write_32(self: &mut Self, offset: usize, value: u32) -> Result<(), BusError>;
}

#[no_mangle]
pub fn m68k_read_disassembler_8(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_8(address as usize) {
        Ok(byte) => byte as c_uint,
        Err(_) => {
            0
        }
    }
}

#[no_mangle]
pub fn m68k_read_disassembler_16(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_16(address as usize) {
        Ok(byte) => byte as c_uint,
        Err(_) => {
            0
        }
    }
}

#[no_mangle]
pub fn m68k_read_disassembler_32(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_32(address as usize) {
        Ok(byte) => byte as c_uint,
        Err(_) => {
            0
        }
    }
}

#[no_mangle]
pub fn m68k_read_memory_8(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_8(address as usize) {
        Ok(byte) => {
            trace!("Read BYTE {:08x} = {:04x}", address, byte);
            byte as c_uint
        },
        Err(_) => {
            cpu::bus_error();
            0
        }
    }
}

#[no_mangle]
pub fn m68k_read_memory_16(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_16(address as usize) {
        Ok(word) => {
            trace!("Read WORD {:08x} = {:04x}", address, word);
            word as c_uint
        },
        Err(_) => {
            cpu::bus_error();
            0
        }
    }
}

#[no_mangle]
pub fn m68k_read_memory_32(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_32(address as usize) {
        Ok(long) => {
            trace!("Read LONG {:08x} = {:08x}", address, long);
            long as c_uint
        },
        Err(_) => {
            cpu::bus_error();
            0
        }
    }
}

#[no_mangle]
pub fn m68k_write_memory_8(addr: c_uint, val: c_uint) {
    trace!("Write BYTE {:08x} = {:02x}", addr, val);
    match BUS.lock().unwrap().write_8(addr as usize, val as u8) {
        Ok(()) => {},
        Err(BusError::ReadOnly) => {
            trace!("READ-ONLY ERROR");
        },
        Err(_) => {
            cpu::bus_error()
        }
    }
}

#[no_mangle]
pub fn m68k_write_memory_16(addr: c_uint, val: c_uint) {
    trace!("Write WORD {:08x} = {:04x}", addr, val);
    match BUS.lock().unwrap().write_16(addr as usize, val as u16) {
        Ok(()) => {},
        Err(BusError::ReadOnly) => {
            trace!("READ-ONLY ERROR");
        },
        Err(_) => {
            cpu::bus_error()
        }
    }
}

#[no_mangle]
pub fn m68k_write_memory_32(addr: c_uint, val: c_uint) {
    trace!("Write LONG {:08x} = {:08x}", addr, val);
    match BUS.lock().unwrap().write_32(addr as usize, val as u32) {
        Ok(()) => {},
        Err(BusError::ReadOnly) => {
            trace!("READ-ONLY ERROR");
        },
        Err(_) => {
            cpu::bus_error()
        }
    }
}

#[cfg(test)]
mod tests {

    mod rom_ram {
        use super::super::*;

        #[test]
        fn test_read_write_8() {
            let mut bus = Bus::new();
            bus.reset = false;

            let _ = bus.write_8(0x100, 0x01).unwrap();
            assert_eq!(0x01, bus.read_8(0x100).unwrap());
        }

        #[test]
        fn test_read_write_8_bad_address() {
            let mut bus = Bus::new();
            bus.reset = false;

            let result = bus.write_8(0x2000000, 0x01);
            assert_eq!(Err(BusError::Access), result);
        }

        #[test]
        fn test_read_write_8_read_only() {
            let mut bus = Bus::new();
            bus.reset = false;

            let result = bus.write_8(0x740000, 0x01);
            assert_eq!(Err(BusError::ReadOnly), result);
        }

        #[test]
        fn test_read_write_16() {
            let mut bus = Bus::new();
            bus.reset = false;

            let _ = bus.write_16(0x100, 0x0102).unwrap();
            assert_eq!(0x0102, bus.read_16(0x100).unwrap());
        }

        #[test]
        fn test_read_write_16_alignment() {
            let mut bus = Bus::new();
            bus.reset = false;

            assert_eq!(Err(BusError::Alignment), bus.write_16(0x101, 0x0102));
            assert_eq!(Err(BusError::Alignment), bus.read_16(0x101));
        }

        #[test]
        fn test_read_write_16_bad_address() {
            let mut bus = Bus::new();
            bus.reset = false;

            let result = bus.write_16(0x2000000, 0x0102);
            assert_eq!(Err(BusError::Access), result);
        }

        #[test]
        fn test_read_write_16_read_only() {
            let mut bus = Bus::new();
            bus.reset = false;

            let result = bus.write_16(0x740000, 0x0102);
            assert_eq!(Err(BusError::ReadOnly), result);
        }

        #[test]
        fn test_read_write_32() {
            let mut bus = Bus::new();
            bus.reset = false;

            let _ = bus.write_32(0x100, 0x01020304).unwrap();
            assert_eq!(0x01020304, bus.read_32(0x100).unwrap());
        }

        #[test]
        fn test_read_write_32_alignment() {
            let mut bus = Bus::new();
            bus.reset = false;

            assert_eq!(Err(BusError::Alignment), bus.write_32(0x101, 0x01020304));
            assert_eq!(Err(BusError::Alignment), bus.read_32(0x101));
        }

        #[test]
        fn test_read_write_32_bad_address() {
            let mut bus = Bus::new();
            bus.reset = false;

            let result = bus.write_32(0x2000000, 0x01020304);
            assert_eq!(Err(BusError::Access), result);
        }

        #[test]
        fn test_read_write_32_read_only() {
            let mut bus = Bus::new();
            bus.reset = false;

            let result = bus.write_16(0x740000, 0x0102);
            assert_eq!(Err(BusError::ReadOnly), result);
        }
    }
}
