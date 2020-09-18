///
/// Tektronix 4404 Memory Implementation
///

use crate::err::*;
use crate::bus::IoDevice;
use byteorder::{ByteOrder, BigEndian};

#[allow(dead_code)]
pub struct Memory {
    read_only: bool,
    start_address: usize,
    end_address: usize,
    mem: Vec<u8>,
}

#[allow(dead_code)]
impl Memory {
    pub fn new(start_address: usize, end_address: usize, read_only: bool) -> Result<Memory, SimError> {
        if start_address > end_address {
            return Err(SimError::Init(String::from("Invalid memory range")));
        }

        Ok(Memory {
            read_only,
            start_address,
            end_address,
            mem: vec![0; end_address - start_address + 1],
        })
    }

    fn valid(&self, address: usize) -> bool {
        address <= (self.end_address - self.start_address)
    }
}

impl IoDevice for Memory {

    fn start_address(&self) -> usize {
        self.start_address
    }

    fn end_address(&self) -> usize {
        self.end_address
    }

    /// Read an 8-bit value from memory.
    fn read_8(&self, address: usize) -> std::result::Result<u8, BusError> {
        if self.valid(address) {
            Ok(self.mem[address])
        } else {
            Err(BusError::Access)
        }
    }

    /// Read a Big-Endian 16-bit value from memory.
    fn read_16(&self, address: usize) -> std::result::Result<u16, BusError> {
        if address & 1 != 0 {
            Err(BusError::Alignment)
        } else if self.valid(address) {
            let buf = &self.mem[address..=address+1];
            Ok(BigEndian::read_u16(buf))
        } else {
            Err(BusError::Access)
        }
    }

    /// Read a Big-Endian 32-bit value from memory.
    fn read_32(&self, address: usize) -> std::result::Result<u32, BusError> {
        if address & 1 != 0 {
            Err(BusError::Alignment)
        } else if self.valid(address) {
            let buf = &self.mem[address..=address+3];
            Ok(BigEndian::read_u32(buf))
        } else {
            Err(BusError::Access)
        }
    }

    fn write_8(&mut self, address: usize, value: u8) -> Result<(), BusError> {
        if self.valid(address) {
            self.mem[address] = value;
            Ok(())
        } else {
            Err(BusError::Access)
        }
    }

    fn write_16(&mut self, address: usize, value: u16) -> Result<(), BusError> {
        if address & 1 != 0 {
            Err(BusError::Alignment)
        } else if self.valid(address) {
            let buf = &mut self.mem[address..=address+1];
            Ok(BigEndian::write_u16(buf, value))
        } else {
            Err(BusError::Access)
        }
    }

    fn write_32(&mut self, address: usize, value: u32) -> Result<(), BusError> {
        if address & 1 != 0 {
            Err(BusError::Alignment)
        } else if self.valid(address) {
            let buf = &mut self.mem[address..=address+3];
            Ok(BigEndian::write_u32(buf, value))
        } else {
            Err(BusError::Access)
        }
    }

    fn load(&mut self, data: Vec<u8>) {
        self.mem.copy_from_slice(data.as_slice());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_range() {
        let mem = Memory::new(0x1000, 0, false);
        assert!(mem.is_err(), "Expected invalid memory");
    }

    #[test]
    fn test_read8() {
        let mut mem = Memory::new(0x1000, 0xffff, false).unwrap();

        mem.mem[0x100] = 0x01;
        mem.mem[0x101] = 0x02;
        mem.mem[0x102] = 0x03;
        mem.mem[0x103] = 0x04;

        assert_eq!(0x01, mem.read_8(0x100).unwrap());
        assert_eq!(0x02, mem.read_8(0x101).unwrap());
        assert_eq!(0x03, mem.read_8(0x102).unwrap());
        assert_eq!(0x04, mem.read_8(0x103).unwrap());

        let result = mem.read_8(0xf000);
        assert!(result.is_err(), "Access Error expected.");
    }

    #[test]
    fn test_read16() {
        let mut mem = Memory::new(0x1000, 0xffff, false).unwrap();

        mem.mem[0x100] = 0x01;
        mem.mem[0x101] = 0x02;
        mem.mem[0x102] = 0x03;
        mem.mem[0x103] = 0x04;

        assert_eq!(0x0102, mem.read_16(0x100).unwrap());
        assert_eq!(0x0304, mem.read_16(0x102).unwrap());

        let result = mem.read_16(0x101);
        assert!(result.is_err(), "Alignment Error expected.");

        let result = mem.read_16(0x103);
        assert!(result.is_err(), "Alignment Error expected.");

        let result = mem.read_16(0xf000);
        assert!(result.is_err(), "Access Error expected.");
    }

    #[test]
    fn test_read32() {
        let mut mem = Memory::new(0x1000, 0xffff, false).unwrap();

        mem.mem[0x100] = 0x01;
        mem.mem[0x101] = 0x02;
        mem.mem[0x102] = 0x03;
        mem.mem[0x103] = 0x04;

        assert_eq!(0x01020304, mem.read_32(0x100).unwrap());
        assert_eq!(0x03040000, mem.read_32(0x102).unwrap());

        let result = mem.read_32(0x101);
        assert!(result.is_err(), "Alignment Error expected.");

        let result = mem.read_32(0x103);
        assert!(result.is_err(), "Alignment Error expected.");

        let result = mem.read_32(0xf000);
        assert!(result.is_err(), "Access Error expected.");
    }

    #[test]
    fn test_write_8() {
        let mut mem = Memory::new(0x1000, 0xffff, false).unwrap();

        let _ = mem.write_8(0x100, 0x01);
        assert_eq!(0x01, mem.mem[0x100]);

        let result = mem.write_8(0xf000, 0x01);
        assert!(result.is_err(), "Access Error expected.");
    }

    #[test]
    fn test_write_16() {
        let mut mem = Memory::new(0x1000, 0xffff, false).unwrap();

        let _ = mem.write_16(0x100, 0x0102);
        assert_eq!(0x01, mem.mem[0x100]);
        assert_eq!(0x02, mem.mem[0x101]);

        let result = mem.write_16(0x101, 0x0102);
        assert!(result.is_err(), "Alignment Error expected.");

        let result = mem.write_16(0x103, 0x0102);
        assert!(result.is_err(), "Alignment Error expected.");

        let result = mem.write_16(0xf000, 0x0102);
        assert!(result.is_err(), "Access Error expected.");
    }

    #[test]
    fn test_write_32() {
        let mut mem = Memory::new(0x1000, 0xffff, false).unwrap();

        let _ = mem.write_32(0x100, 0x01020304);
        assert_eq!(0x01, mem.mem[0x100]);
        assert_eq!(0x02, mem.mem[0x101]);
        assert_eq!(0x03, mem.mem[0x102]);
        assert_eq!(0x04, mem.mem[0x103]);

        let _ = mem.write_32(0x102, 0x01020304);
        assert_eq!(0x01, mem.mem[0x102]);
        assert_eq!(0x02, mem.mem[0x103]);
        assert_eq!(0x03, mem.mem[0x104]);
        assert_eq!(0x04, mem.mem[0x105]);

        let result = mem.write_32(0x101, 0x01020304);
        assert!(result.is_err(), "Alignment Error expected.");

        let result = mem.write_32(0x103, 0x01020304);
        assert!(result.is_err(), "Alignment Error expected.");

        let result = mem.write_32(0xf000, 0x01020304);
        assert!(result.is_err(), "Access Error expected.");
    }
}
