///
/// Tektronix 4404 Memory Implementation
///

use crate::bus::IoDevice;
use crate::bus::BusError;
use byteorder::{ByteOrder, BigEndian};

#[allow(dead_code)]
struct Memory {
    virt: bool,
    bootrom: Vec<u8>,
    debugrom: Vec<u8>,
    display: Vec<u8>,
    sysmem: Vec<u8>,
    expmem: Vec<u8>,
}

#[allow(dead_code)]
impl Memory {
    fn new() -> Memory {
        Memory {
            virt: false,
            bootrom: vec![0; 0x10000],
            debugrom: vec![0; 0x10000],
            display: vec![0; 0x100000],
            sysmem: vec![0; 0x100000],
            expmem: vec![0; 0x100000],
        }
    }
}

impl IoDevice for Memory {

    /// Read an 8-bit value from memory.
    fn read_8(&self, address: usize) -> std::result::Result<u8, BusError> {
        // Physical Access
        match address {
            0..=0xfffff => Ok(self.sysmem[address]),
            _ => Err(BusError::Access)
        }
        
    }

    /// Read a Big-Endian 16-bit value from memory.
    fn read_16(&self, address: usize) -> std::result::Result<u16, BusError> {
        if address & 1 != 0 {
            return Err(BusError::Alignment);
        }
        
        match address {
            0..=0xfffff => {
                let buf = &self.sysmem[address..=address+1];
                Ok(BigEndian::read_u16(buf))
            },
            _ => Err(BusError::Access)
        }
    }

    /// Read a Big-Endian 32-bit value from memory.
    fn read_32(&self, address: usize) -> std::result::Result<u32, BusError> {
        if address & 3 != 0 {
            return Err(BusError::Alignment);
        }

        match address {
            0..=0xfffff => {
                let buf = &self.sysmem[address..=address+3];
                Ok(BigEndian::read_u32(buf))
            },
            _ => Err(BusError::Access)
        }
    }

    fn write_8(&mut self, _: usize, _: u8) -> Result<(), BusError> {
        todo!()
    }

    fn write_16(&mut self, _: usize, _: u16) -> Result<(), BusError> {
        todo!()
    }

    fn write_32(&mut self, _: usize, _: u32) -> Result<(), BusError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read8() -> Result<(), String> {
        let mut mem = Memory::new();

        mem.sysmem[0x100] = 0x01;
        mem.sysmem[0x101] = 0x02;
        mem.sysmem[0x102] = 0x03;
        mem.sysmem[0x103] = 0x04;

        assert_eq!(0x01, mem.read_8(0x100).unwrap());
        assert_eq!(0x02, mem.read_8(0x101).unwrap());
        assert_eq!(0x03, mem.read_8(0x102).unwrap());
        assert_eq!(0x04, mem.read_8(0x103).unwrap());

        match mem.read_8(0x100001) {
            Ok(_) => Err(String::from("Should not have been able to read past end of memory")),
            Err(_) => Ok(())
        }
    }

    #[test]
    fn test_read16() -> Result<(), String> {
        let mut mem = Memory::new();

        mem.sysmem[0x100] = 0x01;
        mem.sysmem[0x101] = 0x02;
        mem.sysmem[0x102] = 0x03;
        mem.sysmem[0x103] = 0x04;

        assert_eq!(0x0102, mem.read_16(0x100).unwrap());
        assert_eq!(0x0304, mem.read_16(0x102).unwrap());

        match mem.read_16(0x101) {
            Ok(_) => return Err(String::from("Should have been alignment error")),
            Err(BusError::Access) => return Err(String::from("Should not have been access error")),
            _ => {}
        }

        match mem.read_16(0x103) {
            Ok(_) => return Err(String::from("Should have been alignment error")),
            Err(BusError::Access) => return Err(String::from("Should not have been access error")),
            _ => {}
        }
        
        match mem.read_16(0x100002) {
            Ok(_) => Err(String::from("Should not have been able to read past end of memory")),
            Err(_) => Ok(())
        }
    }

    #[test]
    fn test_read32() -> Result<(), String> {
        let mut mem = Memory::new();

        mem.sysmem[0x100] = 0x01;
        mem.sysmem[0x101] = 0x02;
        mem.sysmem[0x102] = 0x03;
        mem.sysmem[0x103] = 0x04;

        assert_eq!(0x01020304, mem.read_32(0x100).unwrap());
        assert_eq!(0, mem.read_32(0x104).unwrap());

        match mem.read_32(0x101) {
            Ok(_) => return Err(String::from("Should have been alignment error")),
            Err(BusError::Access) => return Err(String::from("Should not have been access error")),
            _ => {}
        }

        match mem.read_32(0x102) {
            Ok(_) => return Err(String::from("Should have been alignment error")),
            Err(BusError::Access) => return Err(String::from("Should not have been access error")),
            _ => {}
        }

        match mem.read_32(0x103) {
            Ok(_) => return Err(String::from("Should have been alignment error")),
            Err(BusError::Access) => return Err(String::from("Should not have been access error")),
            _ => {}
        }

        match mem.read_32(0x100004) {
            Ok(_) => return Err(String::from("Should not have been able to read past end of memory")),
            Err(BusError::Alignment) => return Err(String::from("Should not have been alignment error")),
            _ => {}
        }

        Ok(())
    }
}
