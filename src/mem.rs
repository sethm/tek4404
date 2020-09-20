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

use crate::err::*;
use crate::bus::IoDevice;
use byteorder::{ByteOrder, BigEndian};

use std::ops::RangeInclusive;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Memory {
    read_only: bool,
    start_address: usize,
    end_address: usize,
    mem: Vec<u8>,
}

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

    fn valid(&self, offset: usize) -> bool {
        offset <= (self.end_address - self.start_address)
    }
}

impl IoDevice for Memory {
    fn range(&self) -> RangeInclusive<usize> {
        self.start_address..=self.end_address
    }

    /// Read an 8-bit value from memory.
    fn read_8(&self, offset: usize) -> std::result::Result<u8, BusError> {
        if self.valid(offset) {
            Ok(self.mem[offset])
        } else {
            Err(BusError::Access)
        }
    }

    /// Read a Big-Endian 16-bit value from memory.
    fn read_16(&self, offset: usize) -> std::result::Result<u16, BusError> {
        if offset & 1 != 0 {
            Err(BusError::Alignment)
        } else if self.valid(offset) {
            let buf = &self.mem[offset..=offset+1];
            Ok(BigEndian::read_u16(buf))
        } else {
            Err(BusError::Access)
        }
    }

    /// Read a Big-Endian 32-bit value from memory.
    fn read_32(&self, offset: usize) -> std::result::Result<u32, BusError> {
        if offset & 1 != 0 {
            Err(BusError::Alignment)
        } else if self.valid(offset) {
            let buf = &self.mem[offset..=offset+3];
            Ok(BigEndian::read_u32(buf))
        } else {
            Err(BusError::Access)
        }
    }

    fn write_8(&mut self, offset: usize, value: u8) -> Result<(), BusError> {
        if self.valid(offset) {
            if self.read_only {
                Err(BusError::ReadOnly)
            } else {
                self.mem[offset] = value;
                Ok(())
            }
        } else {
            Err(BusError::Access)
        }
    }

    fn write_16(&mut self, offset: usize, value: u16) -> Result<(), BusError> {
        if offset & 1 != 0 {
            Err(BusError::Alignment)
        } else if self.valid(offset) {
            if self.read_only {
                Err(BusError::ReadOnly)
            } else {
                let buf = &mut self.mem[offset..=offset+1];
                Ok(BigEndian::write_u16(buf, value))
            }
        } else {
            Err(BusError::Access)
        }
    }

    fn write_32(&mut self, offset: usize, value: u32) -> Result<(), BusError> {
        if offset & 1 != 0 {
            Err(BusError::Alignment)
        } else if self.valid(offset) {
            if self.read_only {
                Err(BusError::ReadOnly)
            } else {
                let buf = &mut self.mem[offset..=offset+3];
                Ok(BigEndian::write_u32(buf, value))
            }
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
        assert!(mem.is_err());
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
    fn test_write_8_read_only() {
        let mut mem = Memory::new(0x1000, 0xffff, true).unwrap();

        let result = mem.write_8(0x100, 0x01);
        assert_eq!(Err(BusError::ReadOnly), result);
    }

    #[test]
    fn test_write_16() {
        let mut mem = Memory::new(0x1000, 0xffff, false).unwrap();

        let _ = mem.write_16(0x100, 0x0102);
        assert_eq!(0x01, mem.mem[0x100]);
        assert_eq!(0x02, mem.mem[0x101]);

        let result = mem.write_16(0x101, 0x0102);
        assert_eq!(Err(BusError::Alignment), result);

        let result = mem.write_16(0x103, 0x0102);
        assert_eq!(Err(BusError::Alignment), result);

        let result = mem.write_16(0xf000, 0x0102);
        assert_eq!(Err(BusError::Access), result);
    }

    #[test]
    fn test_write_16_read_only() {
        let mut mem = Memory::new(0x1000, 0xffff, true).unwrap();

        let result = mem.write_16(0x100, 0x0102);
        assert_eq!(Err(BusError::ReadOnly), result);
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

    #[test]
    fn test_write_32_read_only() {
        let mut mem = Memory::new(0x1000, 0xffff, true).unwrap();

        let result = mem.write_32(0x100, 0x01020304);
        assert_eq!(Err(BusError::ReadOnly), result);
    }
}
