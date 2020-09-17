///
/// Tektronix 4404 Memory Implementation
///

use crate::bus::IoDevice;
use crate::bus::BusError;
use byteorder::{ByteOrder, BigEndian};

const SYSMEM_END: usize = 0xfffff;

const RAM_SIZE: usize = 0x100000;
const ROM_SIZE: usize = 0x10000;

#[allow(dead_code)]
struct Memory {
    virt: bool,
    reset: bool,
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
            reset: false,
            bootrom: vec![0; ROM_SIZE],
            debugrom: vec![0; ROM_SIZE],
            display: vec![0; RAM_SIZE],
            sysmem: vec![0; RAM_SIZE],
            expmem: vec![0; RAM_SIZE],
        }
    }

    fn is_sysmem(address: usize) -> bool {
        address <= SYSMEM_END
    }

    fn load_rom(&mut self, _: &str) -> Result<(), BusError> {
        todo!()
    }
}

impl IoDevice for Memory {

    /// Read an 8-bit value from memory.
    fn read_8(&self, address: usize) -> std::result::Result<u8, BusError> {
        // Physical Access
        match address {
            a if Memory::is_sysmem(a) => Ok(self.sysmem[address]),
            _ => Err(BusError::Access)
        }
        
    }

    /// Read a Big-Endian 16-bit value from memory.
    fn read_16(&self, address: usize) -> std::result::Result<u16, BusError> {
        if address & 1 != 0 {
            return Err(BusError::Alignment);
        }
        
        match address {
            a if Memory::is_sysmem(a) => {
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
            a if Memory::is_sysmem(a) => {
                let buf = &self.sysmem[address..=address+3];
                Ok(BigEndian::read_u32(buf))
            },
            _ => Err(BusError::Access)
        }
    }

    fn write_8(&mut self, address: usize, value: u8) -> Result<(), BusError> {
        match address {
            a if Memory::is_sysmem(a) => {
                self.sysmem[address] = value;
                Ok(())
            },
            _ => Err(BusError::Access)
        }
    }

    fn write_16(&mut self, address: usize, value: u16) -> Result<(), BusError> {
        if address & 1 != 0 {
            return Err(BusError::Alignment);
        }
        
        match address {
            a if Memory::is_sysmem(a) => {
                let buf = &mut self.sysmem[address..=address+1];
                Ok(BigEndian::write_u16(buf, value))
            },
            _ => Err(BusError::Access)
        }
    }

    fn write_32(&mut self, address: usize, value: u32) -> Result<(), BusError> {
        if address & 3 != 0 {
            return Err(BusError::Alignment);
        }
        
        match address {
            a if Memory::is_sysmem(a) => {
                let buf = &mut self.sysmem[address..=address+3];
                Ok(BigEndian::write_u32(buf, value))
            },
            _ => Err(BusError::Access)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_r_alignment_err {
        ($mem:ident, $fn:ident, $addr:expr) => ({
            match $mem.$fn($addr) {
                Ok(_) => panic!("Alignment Error expected"),
                Err(BusError::Access) => panic!("Access Error not expected"),
                Err(BusError::Alignment) => {}
            }
        })
    }

    macro_rules! assert_r_access_err {
        ($mem:ident, $fn:ident, $addr:expr) => ({
            match $mem.$fn($addr) {
                Ok(_) => panic!("Access Error expected"),
                Err(BusError::Access) => {},
                Err(BusError::Alignment) => panic!("Alignment Error not expected"),
            }
        })
    }

    macro_rules! assert_w_alignment_err {
        ($mem:ident, $fn:ident, $addr:expr, $val:expr) => ({
            match $mem.$fn($addr, $val) {
                Ok(_) => panic!("Alignment Error expected"),
                Err(BusError::Access) => panic!("Access Error not expected"),
                Err(BusError::Alignment) => {}
            }
        })
    }
    
    macro_rules! assert_w_access_err {
        ($mem:ident, $fn:ident, $addr:expr, $val:expr) => ({
            match $mem.$fn($addr, $val) {
                Ok(_) => panic!("Access Error expected"),
                Err(BusError::Access) => {},
                Err(BusError::Alignment) => panic!("Alignment Error not expected"),
            }
        })
    }

    
    #[test]
    fn test_read8() {
        let mut mem = Memory::new();

        mem.sysmem[0x100] = 0x01;
        mem.sysmem[0x101] = 0x02;
        mem.sysmem[0x102] = 0x03;
        mem.sysmem[0x103] = 0x04;

        assert_eq!(0x01, mem.read_8(0x100).unwrap());
        assert_eq!(0x02, mem.read_8(0x101).unwrap());
        assert_eq!(0x03, mem.read_8(0x102).unwrap());
        assert_eq!(0x04, mem.read_8(0x103).unwrap());

        assert_r_access_err!(mem, read_8, 0x100000);
    }

    #[test]
    fn test_read16() {
        let mut mem = Memory::new();

        mem.sysmem[0x100] = 0x01;
        mem.sysmem[0x101] = 0x02;
        mem.sysmem[0x102] = 0x03;
        mem.sysmem[0x103] = 0x04;

        assert_eq!(0x0102, mem.read_16(0x100).unwrap());
        assert_eq!(0x0304, mem.read_16(0x102).unwrap());

        assert_r_alignment_err!(mem, read_16, 0x101);
        assert_r_alignment_err!(mem, read_16, 0x103);

        assert_r_access_err!(mem, read_16, 0x100000);
    }

    #[test]
    fn test_read32() {
        let mut mem = Memory::new();

        mem.sysmem[0x100] = 0x01;
        mem.sysmem[0x101] = 0x02;
        mem.sysmem[0x102] = 0x03;
        mem.sysmem[0x103] = 0x04;

        assert_eq!(0x01020304, mem.read_32(0x100).unwrap());

        assert_r_alignment_err!(mem, read_32, 0x101);
        assert_r_alignment_err!(mem, read_32, 0x102);
        assert_r_alignment_err!(mem, read_32, 0x103);

        assert_r_access_err!(mem, read_32, 0x100004);
    }

    #[test]
    fn test_write_8() {
        let mut mem = Memory::new();

        let _ = mem.write_8(0x100, 0x5a);
        assert_eq!(0x5a, mem.sysmem[0x100]);

        assert_w_access_err!(mem, write_8, 0x1000000, 0x5a);
    }

    #[test]
    fn test_write_16() {
        let mut mem = Memory::new();

        let _ = mem.write_16(0x100, 0x0102);
        assert_eq!(0x01, mem.sysmem[0x100]);
        assert_eq!(0x02, mem.sysmem[0x101]);

        assert_w_alignment_err!(mem, write_16, 0x101, 0x0102);
        assert_w_alignment_err!(mem, write_16, 0x103, 0x0102);
        
        assert_w_access_err!(mem, write_16, 0x100000, 0x0102);
    }

    #[test]
    fn test_write_32() {
        let mut mem = Memory::new();

        let _ = mem.write_32(0x100, 0x01020304);
        assert_eq!(0x01, mem.sysmem[0x100]);
        assert_eq!(0x02, mem.sysmem[0x101]);
        assert_eq!(0x03, mem.sysmem[0x102]);
        assert_eq!(0x04, mem.sysmem[0x103]);

        assert_w_alignment_err!(mem, write_32, 0x101, 0x01020304);
        assert_w_alignment_err!(mem, write_32, 0x102, 0x01020304);
        assert_w_alignment_err!(mem, write_32, 0x103, 0x01020304);

        assert_w_access_err!(mem, write_32, 0x100000, 0x01020304);
        
    }
}
        
