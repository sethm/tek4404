///
/// Tektronix 4404 Bus Implementation
///

use crate::err::*;

use std::os::raw::c_uint;

pub trait IoDevice {
    fn read_8(self: &Self, address: usize) -> Result<u8, BusError>;
    fn read_16(self: &Self, address: usize) -> Result<u16, BusError>;
    fn read_32(self: &Self, address: usize) -> Result<u32, BusError>;
    fn write_8(self: &mut Self, address: usize, value: u8) -> Result<(), BusError>;
    fn write_16(self: &mut Self, address: usize, value: u16) -> Result<(), BusError>;
    fn write_32(self: &mut Self, address: usize, value: u32) -> Result<(), BusError>;
}

#[no_mangle]
pub fn m68k_read_memory_8(address: c_uint) -> c_uint {
    println!("[READ08] addr=0x{:08x}", address);
    return 0;
}

#[no_mangle]
pub fn m68k_read_memory_16(address: c_uint) -> c_uint {
    println!("[READ16] addr=0x{:08x}", address);
    return 0x4e71;
}

#[no_mangle]
pub fn m68k_read_memory_32(address: c_uint) -> c_uint {
    println!("[READ32] addr=0x{:08x}", address);
    if address == 0 {
        println!("[READ32]    ... STACK = 0x10000");
        return 0x10000;
    } else if address == 4 {
        println!("[READ32]    ...    PC = 0x30000");
        return 0x30000;
    } else {
        return 0x4e71;
    }
}

#[no_mangle]
pub fn m68k_write_memory_8(address: c_uint, value: c_uint) {
    println!("[WRITE08] addr=0x{:08x} val=0x{:02x}", address, value);
}

#[no_mangle]
pub fn m68k_write_memory_16(address: c_uint, value: c_uint) {
    println!("[WRITE16] addr=0x{:08x} val=0x{:04x}", address, value);
}

#[no_mangle]
pub fn m68k_write_memory_32(address: c_uint, value: c_uint) {
    println!("[WRITE32] addr=0x{:08x} val=0x{:08x}", address, value);
}
