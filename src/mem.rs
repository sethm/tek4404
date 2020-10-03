//! Memory (RAM and ROM) devices
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
use crate::err::*;
use byteorder::{BigEndian, ByteOrder};
use std::ops::RangeInclusive;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Memory {
    read_only: bool,
    start_address: usize,
    end_address: usize,
    size: usize,
    pub mem: Vec<u8>,
}

impl Memory {
    pub fn new(
        start_address: usize,
        end_address: usize,
        size: usize,
        read_only: bool,
    ) -> Result<Memory, SimError> {
        if start_address > end_address {
            return Err(SimError::Init(String::from("Invalid memory range")));
        }

        Ok(Memory {
            read_only,
            start_address,
            end_address,
            size,
            mem: vec![0; size],
        })
    }

    fn get_offset(&self, bus: &mut Bus, address: usize) -> Result<usize, BusError> {
        if self.read_only && bus.map_rom {
            Ok(address % self.size)
        } else if self.range().contains(&address) {
            Ok((address - self.start_address) % self.size)
        } else {
            Err(BusError::Access)
        }
    }
}

impl IoDevice for Memory {
    fn range(&self) -> RangeInclusive<usize> {
        self.start_address..=self.end_address
    }

    fn read_8(&mut self, bus: &mut Bus, address: usize) -> std::result::Result<u8, BusError> {
        let offset = self.get_offset(bus, address)?;
        Ok(self.mem[offset])
    }

    fn read_16(&mut self, bus: &mut Bus, address: usize) -> std::result::Result<u16, BusError> {
        let offset = self.get_offset(bus, address)?;
        if offset & 1 != 0 {
            Err(BusError::Alignment)
        } else {
            let buf = &self.mem[offset..=offset + 1];
            Ok(BigEndian::read_u16(buf))
        }
    }

    fn read_32(&mut self, bus: &mut Bus, address: usize) -> std::result::Result<u32, BusError> {
        let offset = self.get_offset(bus, address)?;
        if offset & 1 != 0 {
            Err(BusError::Alignment)
        } else {
            let buf = &self.mem[offset..=offset + 3];
            Ok(BigEndian::read_u32(buf))
        }
    }

    fn write_8(&mut self, bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        let offset = self.get_offset(bus, address)?;
        if self.read_only {
            Err(BusError::ReadOnly)
        } else {
            self.mem[offset] = value;
            Ok(())
        }
    }

    fn write_16(&mut self, bus: &mut Bus, address: usize, value: u16) -> Result<(), BusError> {
        let offset = self.get_offset(bus, address)?;
        if offset & 1 != 0 {
            Err(BusError::Alignment)
        } else {
            if self.read_only {
                Err(BusError::ReadOnly)
            } else {
                let buf = &mut self.mem[offset..=offset + 1];
                Ok(BigEndian::write_u16(buf, value))
            }
        }
    }

    fn write_32(&mut self, bus: &mut Bus, address: usize, value: u32) -> Result<(), BusError> {
        let offset = self.get_offset(bus, address)?;
        if offset & 1 != 0 {
            Err(BusError::Alignment)
        } else {
            if self.read_only {
                Err(BusError::ReadOnly)
            } else {
                let buf = &mut self.mem[offset..=offset + 3];
                Ok(BigEndian::write_u32(buf, value))
            }
        }
    }

    fn load(&mut self, data: &Vec<u8>) {
        self.mem.copy_from_slice(data.as_slice());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    fn with_mem<T>(test: T) -> ()
    where
        T: FnOnce(&mut Memory, &mut Bus) -> () + panic::UnwindSafe,
    {
        let mut mem = Memory::new(0x1000, 0xffff, 0xefff, false).unwrap();
        let mut bus = Bus::empty();

        test(&mut mem, &mut bus);
    }

    #[test]
    fn test_invalid_range() {
        let mem = Memory::new(0x1000, 0, 0x1000, false);
        assert!(mem.is_err());
    }

    #[test]
    fn test_read8() {
        with_mem(|mem, bus| {
            mem.mem[0x100] = 0x01;
            mem.mem[0x101] = 0x02;
            mem.mem[0x102] = 0x03;
            mem.mem[0x103] = 0x04;

            assert_eq!(0x01, mem.read_8(bus, 0x1100).unwrap());
            assert_eq!(0x02, mem.read_8(bus, 0x1101).unwrap());
            assert_eq!(0x03, mem.read_8(bus, 0x1102).unwrap());
            assert_eq!(0x04, mem.read_8(bus, 0x1103).unwrap());

            assert_eq!(Err(BusError::Access), mem.read_8(bus, 0x10000));
        });
    }

    #[test]
    fn test_read16() {
        with_mem(|mem, bus| {
            mem.mem[0x100] = 0x01;
            mem.mem[0x101] = 0x02;
            mem.mem[0x102] = 0x03;
            mem.mem[0x103] = 0x04;

            assert_eq!(0x0102, mem.read_16(bus, 0x1100).unwrap());
            assert_eq!(0x0304, mem.read_16(bus, 0x1102).unwrap());

            assert_eq!(Err(BusError::Alignment), mem.read_16(bus, 0x1101));
            assert_eq!(Err(BusError::Alignment), mem.read_16(bus, 0x1103));
            assert_eq!(Err(BusError::Access), mem.read_16(bus, 0x10000));
        });
    }

    #[test]
    fn test_read32() {
        with_mem(|mem, bus| {
            mem.mem[0x100] = 0x01;
            mem.mem[0x101] = 0x02;
            mem.mem[0x102] = 0x03;
            mem.mem[0x103] = 0x04;

            assert_eq!(0x01020304, mem.read_32(bus, 0x1100).unwrap());
            assert_eq!(0x03040000, mem.read_32(bus, 0x1102).unwrap());

            assert_eq!(Err(BusError::Alignment), mem.read_32(bus, 0x1101));
            assert_eq!(Err(BusError::Alignment), mem.read_32(bus, 0x1103));
            assert_eq!(Err(BusError::Access), mem.read_32(bus, 0x10000));
        });
    }

    #[test]
    fn test_write_8() {
        with_mem(|mem, bus| {
            let _ = mem.write_8(bus, 0x1100, 0x01);
            assert_eq!(0x01, mem.mem[0x100]);

            assert_eq!(Err(BusError::Access), mem.write_8(bus, 0x10000, 0x01));
        })
    }

    #[test]
    fn test_write_8_read_only() {
        with_mem(|mem, bus| {
            mem.read_only = true;
            assert_eq!(Err(BusError::ReadOnly), mem.write_8(bus, 0x1100, 0x01));
        })
    }

    #[test]
    fn test_write_16() {
        with_mem(|mem, bus| {
            let _ = mem.write_16(bus, 0x1100, 0x0102);
            assert_eq!(0x01, mem.mem[0x100]);
            assert_eq!(0x02, mem.mem[0x101]);

            assert_eq!(Err(BusError::Alignment), mem.write_16(bus, 0x1101, 0x0102));
            assert_eq!(Err(BusError::Alignment), mem.write_16(bus, 0x1103, 0x0102));
            assert_eq!(Err(BusError::Access), mem.write_16(bus, 0x10000, 0x0102));
        })
    }

    #[test]
    fn test_write_16_read_only() {
        with_mem(|mem, bus| {
            mem.read_only = true;
            assert_eq!(Err(BusError::ReadOnly), mem.write_16(bus, 0x1100, 0x0102));
        })
    }

    #[test]
    fn test_write_32() {
        with_mem(|mem, bus| {
            let _ = mem.write_32(bus, 0x1100, 0x01020304);
            assert_eq!(0x01, mem.mem[0x100]);
            assert_eq!(0x02, mem.mem[0x101]);
            assert_eq!(0x03, mem.mem[0x102]);
            assert_eq!(0x04, mem.mem[0x103]);

            let _ = mem.write_32(bus, 0x1102, 0x01020304);
            assert_eq!(0x01, mem.mem[0x102]);
            assert_eq!(0x02, mem.mem[0x103]);
            assert_eq!(0x03, mem.mem[0x104]);
            assert_eq!(0x04, mem.mem[0x105]);

            assert_eq!(
                Err(BusError::Alignment),
                mem.write_32(bus, 0x1101, 0x01020304)
            );
            assert_eq!(
                Err(BusError::Alignment),
                mem.write_32(bus, 0x1103, 0x01020304)
            );
            assert_eq!(
                Err(BusError::Access),
                mem.write_32(bus, 0x10000, 0x01020304)
            );
        })
    }

    #[test]
    fn test_write_32_read_only() {
        with_mem(|mem, bus| {
            mem.read_only = true;
            assert_eq!(
                Err(BusError::ReadOnly),
                mem.write_32(bus, 0x1100, 0x01020304)
            );
        })
    }

    #[test]
    fn test_mirroring() {
        let mut mem = Memory::new(0x0, 0x7fff, 0x1000, false).unwrap();
        let mut bus = Bus::empty();

        mem.mem[0x100] = 0x01;
        mem.mem[0x101] = 0x02;
        mem.mem[0x102] = 0x03;
        mem.mem[0x103] = 0x04;

        assert_eq!(0x1000, mem.mem.len());

        for base in (0..0x7000).step_by(0x1000) {
            assert_eq!(0x01, mem.read_8(&mut bus, base + 0x100).unwrap());
            assert_eq!(0x0102, mem.read_16(&mut bus, base + 0x100).unwrap());
            assert_eq!(0x01020304, mem.read_32(&mut bus, base + 0x100).unwrap());
        }

        assert_eq!(Err(BusError::Access), mem.read_8(&mut bus, 0x8000));
        assert_eq!(Err(BusError::Access), mem.read_16(&mut bus, 0x8000));
        assert_eq!(Err(BusError::Access), mem.read_32(&mut bus, 0x8000));
    }
}
