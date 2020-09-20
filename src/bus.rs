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
use crate::sound::*;

use std::ops::RangeInclusive;
use std::os::raw::c_uint;
use std::sync::{Arc, Mutex, RwLock};

pub const ROM_START: usize = 0x740000;
pub const ROM_END_PHYSICAL: usize = 0x747fff;
pub const ROM_END_VIRTUAL: usize = 0x74ffff;

pub const RAM_START: usize = 0;
pub const RAM_END: usize = 0x1fffff;

pub const SOUND_START: usize = 0x788000;
pub const SOUND_END: usize = 0x788fff;

pub const ACIA_START: usize = 0x78c000;
pub const ACIA_END: usize = 0x78c007;

lazy_static! {
    pub static ref BUS: Arc<Mutex<Bus>> = Arc::new(Mutex::new(Bus::new()));
}

pub type BusDevice = Arc<RwLock<dyn IoDevice + Send + Sync>>;

pub struct Bus {
    pub reset: bool,
    rom: BusDevice,
    ram: BusDevice,
    sound: BusDevice,
}

pub fn load_rom(boot_rom: &str) -> Result<(), SimError> {
    Arc::clone(&BUS).lock().unwrap().load_rom(boot_rom)
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            reset: true,
            rom: Arc::new(RwLock::new(
                Memory::new(ROM_START, ROM_END_PHYSICAL, true).expect("Unable to init ROM"),
            )),
            ram: Arc::new(RwLock::new(
                Memory::new(RAM_START, RAM_END, false).expect("Unable to init RAM"),
            )),
            sound: Arc::new(RwLock::new(Sound {})),
        }
    }

    fn get_device(&mut self, address: usize) -> Result<BusDevice, BusError> {
        match address {
            ROM_START..=ROM_END_VIRTUAL => Ok(Arc::clone(&self.rom)),
            RAM_START..=RAM_END => {
                if self.reset {
                    Ok(Arc::clone(&self.rom))
                } else {
                    Ok(Arc::clone(&self.ram))
                }
            }
            SOUND_START..=SOUND_END => {
                if self.reset {
                    info!("Bus Reset flip-flop cleared. ROM now mapped");
                    self.reset = false;
                }
                Ok(Arc::clone(&self.sound))
            }
            ACIA_START..=ACIA_END => {
                todo!("ACIA at {:08x}-{:08x}", ACIA_START, ACIA_END);
            }
            _ => {
                error!("No device at address {:08x}", address);
                Err(BusError::Access)
            }
        }
    }

    fn load_rom(&mut self, rom_file: &str) -> Result<(), SimError> {
        let result = std::fs::read(rom_file);
        match result {
            Ok(data) => {
                self.rom.write().unwrap().load(&data);
                info!("Loaded {} bytes from {}", &data.len(), rom_file);
                Ok(())
            }
            Err(_) => Err(SimError::Init(String::from("Could not load ROM file."))),
        }
    }

    fn read_8(&mut self, address: usize) -> Result<u8, BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.write().unwrap();
        dev.read_8(&self, address)
    }

    fn read_16(&mut self, address: usize) -> Result<u16, BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.write().unwrap();
        dev.read_16(&self, address)
    }

    fn read_32(&mut self, address: usize) -> Result<u32, BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.write().unwrap();
        dev.read_32(&self, address)
    }

    fn write_8(&mut self, address: usize, value: u8) -> Result<(), BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.write().unwrap();
        dev.write_8(&self, address, value)
    }

    fn write_16(&mut self, address: usize, value: u16) -> Result<(), BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.write().unwrap();
        dev.write_16(&self, address, value)
    }

    fn write_32(&mut self, address: usize, value: u32) -> Result<(), BusError> {
        let mutex = self.get_device(address)?;
        let mut dev = mutex.write().unwrap();
        dev.write_32(&self, address, value)
    }
}

pub trait IoDevice {
    fn load(self: &mut Self, data: &Vec<u8>);
    fn range(self: &Self) -> RangeInclusive<usize>;
    fn read_8(self: &mut Self, bus: &Bus, address: usize) -> Result<u8, BusError>;
    fn read_16(self: &mut Self, bus: &Bus, address: usize) -> Result<u16, BusError>;
    fn read_32(self: &mut Self, bus: &Bus, address: usize) -> Result<u32, BusError>;
    fn write_8(self: &mut Self, bus: &Bus, address: usize, value: u8) -> Result<(), BusError>;
    fn write_16(self: &mut Self, bus: &Bus, address: usize, value: u16) -> Result<(), BusError>;
    fn write_32(self: &mut Self, bus: &Bus, address: usize, value: u32) -> Result<(), BusError>;
}

#[no_mangle]
pub fn m68k_read_disassembler_8(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_8(address as usize) {
        Ok(byte) => byte as c_uint,
        Err(_) => 0,
    }
}

#[no_mangle]
pub fn m68k_read_disassembler_16(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_16(address as usize) {
        Ok(byte) => byte as c_uint,
        Err(_) => 0,
    }
}

#[no_mangle]
pub fn m68k_read_disassembler_32(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_32(address as usize) {
        Ok(byte) => byte as c_uint,
        Err(_) => 0,
    }
}

#[no_mangle]
pub fn m68k_read_memory_8(address: c_uint) -> c_uint {
    match BUS.lock().unwrap().read_8(address as usize) {
        Ok(byte) => {
            trace!("Read BYTE {:08x} = {:04x}", address, byte);
            byte as c_uint
        }
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
        }
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
        }
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
        Ok(()) => {}
        Err(BusError::ReadOnly) => {
            trace!("READ-ONLY ERROR");
        }
        Err(_) => cpu::bus_error(),
    }
}

#[no_mangle]
pub fn m68k_write_memory_16(addr: c_uint, val: c_uint) {
    trace!("Write WORD {:08x} = {:04x}", addr, val);
    match BUS.lock().unwrap().write_16(addr as usize, val as u16) {
        Ok(()) => {}
        Err(BusError::ReadOnly) => {
            trace!("READ-ONLY ERROR");
        }
        Err(_) => cpu::bus_error(),
    }
}

#[no_mangle]
pub fn m68k_write_memory_32(addr: c_uint, val: c_uint) {
    trace!("Write LONG {:08x} = {:08x}", addr, val);
    match BUS.lock().unwrap().write_32(addr as usize, val as u32) {
        Ok(()) => {}
        Err(BusError::ReadOnly) => {
            trace!("READ-ONLY ERROR");
        }
        Err(_) => cpu::bus_error(),
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
