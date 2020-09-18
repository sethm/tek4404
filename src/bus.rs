use crate::cpu;
///
/// Tektronix 4404 Bus Implementation
///
use crate::err::*;
use crate::mem::*;

use std::os::raw::c_uint;
use std::sync::{Arc, Mutex};

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

    fn dev_offset(&self, address: usize, dev: BusDevice) -> Result<usize, BusError> {
        let dev = &dev.lock().unwrap();
        let start_addr = dev.start_address();
        let end_addr = dev.end_address();
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
        println!("[    BUS] Loaded {} bytes from ./rom/boot.bin", rom_data.len());
        self.rom.lock().unwrap().load(rom_data);
    }

    fn read_8(&self, address: usize) -> Result<u8, BusError> {
        let dev = self.get_device(address)?;
        let offset = self.dev_offset(address, Arc::clone(&dev))?;
        Arc::clone(&dev).lock().unwrap().read_8(offset)
    }

    fn read_16(&self, address: usize) -> Result<u16, BusError> {
        let dev = self.get_device(address)?;
        let offset = self.dev_offset(address, Arc::clone(&dev))?;
        Arc::clone(&dev).lock().unwrap().read_16(offset)
    }

    fn read_32(&self, address: usize) -> Result<u32, BusError> {
        let dev = self.get_device(address)?;
        let offset = self.dev_offset(address, Arc::clone(&dev))?;
        Arc::clone(&dev).lock().unwrap().read_32(offset)
    }
}

pub trait IoDevice {
    fn start_address(self: &Self) -> usize;
    fn end_address(self: &Self) -> usize;
    fn read_8(self: &Self, address: usize) -> Result<u8, BusError>;
    fn read_16(self: &Self, address: usize) -> Result<u16, BusError>;
    fn read_32(self: &Self, address: usize) -> Result<u32, BusError>;
    fn write_8(self: &mut Self, address: usize, value: u8) -> Result<(), BusError>;
    fn write_16(self: &mut Self, address: usize, value: u16) -> Result<(), BusError>;
    fn write_32(self: &mut Self, address: usize, value: u32) -> Result<(), BusError>;
    fn load(self: &mut Self, data: Vec<u8>);
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
    println!("[    BUS] Write {:08x} = {:02x}", addr, val);
}

#[no_mangle]
pub fn m68k_write_memory_16(addr: c_uint, val: c_uint) {
    println!("[    BUS] Write {:08x} = {:02x}", addr, val);
}

#[no_mangle]
pub fn m68k_write_memory_32(addr: c_uint, val: c_uint) {
    println!("[    BUS] Write {:08x} = {:02x}", addr, val);
}
