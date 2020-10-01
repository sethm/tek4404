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
use crate::bus::*;
use crate::err::*;

pub struct Sound {}

impl Sound {
    pub fn new() -> Sound {
        Sound {}
    }
    
    /// On initialization, the system comes up with the boot ROM
    /// mapped to all RAM and ROM memory locations. Any write to the
    /// sound chip will un-map the ROM from RAM space.
    fn unmap_rom(&mut self, bus: &mut Bus) {
        if bus.map_rom {
            info!("Un-mapping boot ROM.");
            bus.map_rom = false;
        }
    }
}

impl IoDevice for Sound {
    fn range(&self) -> std::ops::RangeInclusive<usize> {
        SOUND_START..=SOUND_END
    }

    // This is a write-only device. Reading produces no meaningful result.
    fn read_8(&mut self, _: &mut Bus, _: usize) -> std::result::Result<u8, BusError> {
        Ok(0)
    }

    // This is a write-only device. Reading produces no meaningful result.
    fn read_16(&mut self, _: &mut Bus, _: usize) -> std::result::Result<u16, BusError> {
        Ok(0)
    }

    // This is a write-only device. Reading produces no meaningful result.
    fn read_32(&mut self, _: &mut Bus, _: usize) -> std::result::Result<u32, BusError> {
        Ok(0)
    }

    fn write_8(&mut self, bus: &mut Bus, _: usize, data: u8) -> std::result::Result<(), BusError> {
        self.unmap_rom(bus);
        info!("SOUND WRITE: data={:02x}", data);
        Ok(())
    }

    fn write_16(
        &mut self,
        bus: &mut Bus,
        _: usize,
        data: u16,
    ) -> std::result::Result<(), BusError> {
        self.unmap_rom(bus);
        info!("SOUND WRITE: data={:04x}", data);
        Ok(())
    }

    fn write_32(
        &mut self,
        bus: &mut Bus,
        _: usize,
        data: u32,
    ) -> std::result::Result<(), BusError> {
        self.unmap_rom(bus);
        info!("SOUND WRITE: data={:04x}", data);
        Ok(())
    }
}
