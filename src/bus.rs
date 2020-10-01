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
use crate::acia::*;
use crate::cpu;
use crate::err::*;
use crate::mem::*;
use crate::sound::*;
use crate::video::*;

use std::ops::RangeInclusive;
use std::os::raw::c_uint;
use std::sync::{Arc, Mutex, RwLock};

pub const ROM_START: usize = 0x740000;
pub const ROM_END: usize = 0x74ffff;
pub const ROM_SIZE: usize = 0x8000;

pub const RAM_START: usize = 0;
pub const RAM_END: usize = 0x1fffff;
pub const RAM_SIZE: usize = 0x200000;

pub const DEBUG_RAM_START: usize = 0x760000;
pub const DEBUG_RAM_END: usize = 0x76ffff;
pub const DEBUG_RAM_SIZE: usize = 0x1000;

pub const SOUND_START: usize = 0x788000;
pub const SOUND_END: usize = 0x788fff;

pub const ACIA_START: usize = 0x78c000;
pub const ACIA_END: usize = 0x78c007;

pub const VIDEO_CTRL_START: usize = 0x784000;
pub const VIDEO_CTRL_END: usize = 0x785fff;

pub const VIDEO_RAM_START: usize = 0x600000;
pub const VIDEO_RAM_END: usize = 0x61ffff;
pub const VIDEO_RAM_SIZE: usize = 0x20000;

// The existence of this global, mutable shared state is unfortunately
// made necessary by the nature of the C Musashi 68K core library.
// There must be a global bus available for the extern C functions
// that are used to read and write to the bus. In order to prevent
// deadlocks, however, *only* the extern C functions and a few select
// helper functions are allowed to lock the bus mutex!
lazy_static! {
    pub static ref BUS: Mutex<Bus> = Mutex::new(Bus::new());
}

pub fn load_rom(boot_rom: &str) -> Result<(), SimError> {
    BUS.lock().unwrap().load_rom(boot_rom)
}

pub type BusDevice = Arc<RwLock<dyn IoDevice + Send + Sync>>;
pub type MemoryDevice = Arc<RwLock<Memory>>;
pub type SoundDevice = Arc<RwLock<Sound>>;
pub type AciaDevice = Arc<RwLock<Acia>>;
pub type VideoControlDevice = Arc<RwLock<VideoControl>>;

pub struct Bus {
    pub map_rom: bool,
    pub rom: Option<MemoryDevice>,
    pub ram: Option<MemoryDevice>,
    pub debug_ram: Option<MemoryDevice>,
    pub sound: Option<SoundDevice>,
    pub acia: Option<AciaDevice>,
    pub video_ctrl: Option<VideoControlDevice>,
    pub video_ram: Option<MemoryDevice>,
}

impl Bus {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Bus {
            map_rom: false,
            rom: None,
            ram: None,
            debug_ram: None,
            sound: None,
            acia: None,
            video_ctrl: None,
            video_ram: None,
        }
    }

    pub fn new() -> Self {
        Bus {
            map_rom: true,
            rom: Some(Arc::new(RwLock::new(
                Memory::new(ROM_START, ROM_END, ROM_SIZE, true).unwrap(),
            ))),
            ram: Some(Arc::new(RwLock::new(
                Memory::new(RAM_START, RAM_END, RAM_SIZE, false).unwrap(),
            ))),
            debug_ram: Some(Arc::new(RwLock::new(
                Memory::new(DEBUG_RAM_START, DEBUG_RAM_END, DEBUG_RAM_SIZE, false).unwrap(),
            ))),
            sound: Some(Arc::new(RwLock::new(Sound::new()))),
            acia: None,
            video_ctrl: Some(Arc::new(RwLock::new(VideoControl::new()))),
            video_ram: Some(Arc::new(RwLock::new(
                Memory::new(VIDEO_RAM_START, VIDEO_RAM_END, VIDEO_RAM_SIZE, false).unwrap(),
            ))),
        }
    }

    pub fn set_acia(&mut self, acia: AciaDevice) {
        self.acia = Some(acia);
    }

    pub fn set_video_ram(&mut self, video_ram: MemoryDevice) {
        self.video_ram = Some(video_ram);
    }

    fn map_device(&mut self, addr: usize) -> Result<BusDevice, BusError> {
        match addr {
            ROM_START..=ROM_END => match &mut self.rom {
                Some(d) => Ok(d.clone()),
                None => Err(BusError::Access),
            },
            RAM_START..=RAM_END => {
                if self.map_rom {
                    match &mut self.rom {
                        Some(d) => Ok(d.clone()),
                        None => Err(BusError::Access),
                    }
                } else {
                    match &mut self.ram {
                        Some(d) => Ok(d.clone()),
                        None => Err(BusError::Access),
                    }
                }
            }
            DEBUG_RAM_START..=DEBUG_RAM_END => match &mut self.debug_ram {
                Some(d) => Ok(d.clone()),
                None => Err(BusError::Access),
            },
            SOUND_START..=SOUND_END => match &mut self.sound {
                Some(d) => Ok(d.clone()),
                None => Err(BusError::Access),
            },
            ACIA_START..=ACIA_END => match &mut self.acia {
                Some(d) => Ok(d.clone()),
                None => Err(BusError::Access),
            },
            VIDEO_CTRL_START..=VIDEO_CTRL_END => match &mut self.acia {
                Some(d) => Ok(d.clone()),
                None => Err(BusError::Access),
            },
            VIDEO_RAM_START..=VIDEO_RAM_END => match &mut self.video_ram {
                Some(d) => Ok(d.clone()),
                None => Err(BusError::Access),
            },
            _ => {
                error!("No device at address {:08x}", addr);
                Err(BusError::Access)
            }
        }
    }

    fn load_rom(&mut self, rom_file: &str) -> Result<(), SimError> {
        let result = std::fs::read(rom_file);
        match result {
            Ok(data) => match &mut self.rom {
                Some(dev) => {
                    info!("Loaded {} bytes from {}", &data.len(), rom_file);
                    Ok(dev.write().unwrap().load(&data))
                }
                None => Err(SimError::Init(String::from("Could not load ROM file."))),
            },
            Err(_) => Err(SimError::Init(String::from("Could not load ROM file."))),
        }
    }

    fn read_8(&mut self, address: usize) -> Result<u8, BusError> {
        self.map_device(address)?
            .write()
            .unwrap()
            .read_8(self, address)
    }

    fn read_16(&mut self, address: usize) -> Result<u16, BusError> {
        self.map_device(address)?
            .write()
            .unwrap()
            .read_16(self, address)
    }

    fn read_32(&mut self, address: usize) -> Result<u32, BusError> {
        self.map_device(address)?
            .write()
            .unwrap()
            .read_32(self, address)
    }

    fn write_8(&mut self, address: usize, value: u8) -> Result<(), BusError> {
        self.map_device(address)?
            .write()
            .unwrap()
            .write_8(self, address, value)
    }

    fn write_16(&mut self, address: usize, value: u16) -> Result<(), BusError> {
        self.map_device(address)?
            .write()
            .unwrap()
            .write_16(self, address, value)
    }

    fn write_32(&mut self, address: usize, value: u32) -> Result<(), BusError> {
        self.map_device(address)?
            .write()
            .unwrap()
            .write_32(self, address, value)
    }
}

pub trait IoDevice {
    fn range(self: &Self) -> RangeInclusive<usize>;

    // No-op defaults are provided as a convenience for any device
    // that does not need to implement all data sizes.
    fn read_8(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u8, BusError> {
        Ok(0)
    }

    fn read_16(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u16, BusError> {
        Ok(0)
    }

    fn read_32(self: &mut Self, _bus: &mut Bus, _address: usize) -> Result<u32, BusError> {
        Ok(0)
    }

    fn write_8(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u8,
    ) -> Result<(), BusError> {
        Ok(())
    }

    fn write_16(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u16,
    ) -> Result<(), BusError> {
        Ok(())
    }

    fn write_32(
        self: &mut Self,
        _bus: &mut Bus,
        _address: usize,
        _value: u32,
    ) -> Result<(), BusError> {
        Ok(())
    }

    // Only memory-like devices may need to load data, wo the default
    // implementation is a no-op.
    fn load(self: &mut Self, _data: &Vec<u8>) {}
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
            io!("[ READ] [BYTE] {:08x} = {:04x}", address, byte);
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
            io!("[ READ] [WORD] {:08x} = {:04x}", address, word);
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
            io!("[ READ] [LONG] {:08x} = {:08x}", address, long);
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
    io!("[WRITE] [BYTE] {:08x} = {:02x}", addr, val);
    match BUS.lock().unwrap().write_8(addr as usize, val as u8) {
        Ok(()) => {}
        Err(BusError::ReadOnly) => {
            io!("READ-ONLY ERROR");
        }
        Err(_) => cpu::bus_error(),
    }
}

#[no_mangle]
pub fn m68k_write_memory_16(addr: c_uint, val: c_uint) {
    io!("[WRITE] [WORD] {:08x} = {:04x}", addr, val);
    match BUS.lock().unwrap().write_16(addr as usize, val as u16) {
        Ok(()) => {}
        Err(BusError::ReadOnly) => {
            io!("READ-ONLY ERROR");
        }
        Err(_) => cpu::bus_error(),
    }
}

#[no_mangle]
pub fn m68k_write_memory_32(addr: c_uint, val: c_uint) {
    io!("[WRITE] [LONG] {:08x} = {:08x}", addr, val);
    match BUS.lock().unwrap().write_32(addr as usize, val as u32) {
        Ok(()) => {}
        Err(BusError::ReadOnly) => {
            io!("READ-ONLY ERROR");
        }
        Err(_) => cpu::bus_error(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    fn with_bus<T>(test: T) -> ()
    where
        T: FnOnce(&mut Bus) -> () + panic::UnwindSafe,
    {
        let mut bus = Bus::new();
        bus.map_rom = false;

        test(&mut bus);
    }

    mod rom_ram {
        use super::*;

        #[test]
        fn test_read_write_8() {
            with_bus(|bus| {
                let _ = bus.write_8(0x100, 0x01).unwrap();
                assert_eq!(0x01, bus.read_8(0x100).unwrap());
            })
        }

        #[test]
        fn test_read_write_8_bad_address() {
            with_bus(|bus| {
                assert_eq!(Err(BusError::Access), bus.write_8(0x2000000, 0x01));
            });
        }

        #[test]
        fn test_read_write_8_read_only() {
            with_bus(|bus| {
                let result = bus.write_8(0x740000, 0x01);
                assert_eq!(Err(BusError::ReadOnly), result);
            })
        }

        #[test]
        fn test_read_write_16() {
            with_bus(|bus| {
                let _ = bus.write_16(0x100, 0x0102).unwrap();
                assert_eq!(0x0102, bus.read_16(0x100).unwrap());
            })
        }

        #[test]
        fn test_read_write_16_alignment() {
            with_bus(|bus| {
                assert_eq!(Err(BusError::Alignment), bus.write_16(0x101, 0x0102));
                assert_eq!(Err(BusError::Alignment), bus.read_16(0x101));
            })
        }

        #[test]
        fn test_read_write_16_bad_address() {
            with_bus(|bus| {
                let result = bus.write_16(0x2000000, 0x0102);
                assert_eq!(Err(BusError::Access), result);
            })
        }

        #[test]
        fn test_read_write_16_read_only() {
            with_bus(|bus| {
                let result = bus.write_16(0x740000, 0x0102);
                assert_eq!(Err(BusError::ReadOnly), result);
            })
        }

        #[test]
        fn test_read_write_32() {
            with_bus(|bus| {
                let _ = bus.write_32(0x100, 0x01020304).unwrap();
                assert_eq!(0x01020304, bus.read_32(0x100).unwrap());
            })
        }

        #[test]
        fn test_read_write_32_alignment() {
            with_bus(|bus| {
                assert_eq!(Err(BusError::Alignment), bus.write_32(0x101, 0x01020304));
                assert_eq!(Err(BusError::Alignment), bus.read_32(0x101));
            })
        }

        #[test]
        fn test_read_write_32_bad_address() {
            with_bus(|bus| {
                let result = bus.write_32(0x2000000, 0x01020304);
                assert_eq!(Err(BusError::Access), result);
            })
        }

        #[test]
        fn test_read_write_32_read_only() {
            with_bus(|bus| {
                let result = bus.write_16(0x740000, 0x0102);
                assert_eq!(Err(BusError::ReadOnly), result);
            })
        }
    }
}
