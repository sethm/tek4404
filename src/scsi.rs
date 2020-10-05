//! NCR 5386 SCSI controller
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
use crate::err::BusError;

use crate::IntQue;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

const DIAG_COMPLETE: u8 = 0x80;
const SCSI_INT: u8 = 3;

#[allow(dead_code)]
enum State {
    Disconnected,
    ConnectedAsTarget,
    ConnectedAsInitiator,
}

#[derive(FromPrimitive)]
enum RegAddr {
    Address = 0x7bc000,
    Data1 = 0x7be000,
    Command = 0x7be002,
    Control = 0x7be004,
    DestId = 0x7be006,
    AuxStatus = 0x7be008,
    Id = 0x7be00a,
    Interrupt = 0x7be00c,
    SourceId = 0x7be00e,
    Data2 = 0x7be0010,
    DiagStatus = 0x7be012,
    Xfer2 = 0x7be018,
    Xfer1 = 0x7be01a,
    Xfer0 = 0x7be01c,
}

#[derive(FromPrimitive)]
enum Command {
    // Immediate Commands
    ChipReset = 0,
    Disconnect = 1,
    Paused = 2,
    SetAtn = 3,
    MessageAccepted = 4,
    ChipDisable = 5,

    // Interrupt Driven Commands
    SelectWithAtn = 8,
    SelectWithoutAtn = 9,
    Reselect = 10,
    Diagnostic = 11,
    RxCmd = 12,
    RxData = 13,
    RxMessageOut = 14,
    RxUnspInfoOut = 15,
    TxStatus = 16,
    TxData = 17,
    TxMessageOut = 18,
    TxUnspInfoIn = 19,
    TransferInfo = 20,
    TransferPad = 21,
}

#[allow(dead_code)]
pub struct Scsi {
    state: State,
    address: usize,
    data1: u8,
    command: u8,
    control: u8,
    dest_id: u8,
    aux_stat: u8,
    id: u8,
    interrupt: u8,
    source_id: u8,
    data2: u8,
    diag_status: u8,
    xfer: u32,
    int_queue: Arc<Mutex<IntQue>>,
}

impl Scsi {
    pub fn new(int_queue: Arc<Mutex<IntQue>>) -> Self {
        Scsi {
            state: State::Disconnected,
            address: 0,
            data1: 0,
            command: 0,
            control: 0,
            dest_id: 0,
            aux_stat: 0,
            id: 0,
            interrupt: 0,
            source_id: 0,
            data2: 0,
            diag_status: 0,
            xfer: 0,
            int_queue,
        }
    }

    fn handle_command(&mut self) {
        let c = self.command;

        match FromPrimitive::from_u8(c) {
            Some(Command::ChipReset) => {
                // Chip reset
                info!("RESET");
                self.data1 = 0;
                self.control = 0;
                self.dest_id = 0;
                self.aux_stat = 0x2; // Xfer Counter Zero
                self.interrupt = 0;
                self.source_id = 0;
                self.data2 = 0;
                self.diag_status = DIAG_COMPLETE;
                self.xfer = 0;
            }
            Some(Command::SelectWithAtn) => {
                info!("SELECT_WITH_ATTENTION (id={})", self.dest_id);
                self.state = State::ConnectedAsInitiator;
                info!("SCHEDULING INTERRUPT");
                self.interrupt = 0x1;
                self.int_queue.lock().unwrap().schedule(SCSI_INT, 10000);
            }
            Some(Command::SelectWithoutAtn) => {
                info!("SELECT_WITHOUT_ATTENTION (id={})", self.dest_id);
                self.state = State::ConnectedAsInitiator;
                info!("SCHEDULING INTERRUPT");
                self.interrupt = 0x1;
                self.int_queue.lock().unwrap().schedule(SCSI_INT, 10000);
            }
            _ => {
                info!("Unhandled scsi command: 0x{:02x} (b{:06b})", c, c);
            }
        }
    }
}

impl IoDevice for Scsi {
    fn range(&self) -> RangeInclusive<usize> {
        SCSI_START..=SCSI_END
    }

    fn read_8(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        info!("(READ 8) addr={:08x}", address);
        match FromPrimitive::from_usize(address) {
            Some(RegAddr::Data1) => Ok(self.data1),
            Some(RegAddr::Command) => Ok(self.command),
            Some(RegAddr::Control) => Ok(self.control),
            Some(RegAddr::DestId) => Ok(self.dest_id),
            Some(RegAddr::AuxStatus) => {
                info!("SCSI Aux Info Read: {:02x}", self.aux_stat);
                Ok(self.aux_stat)
            }
            Some(RegAddr::Id) => Ok(self.id),
            Some(RegAddr::Interrupt) => {
                info!("SCSI Interrupt Read: {:02x}", self.interrupt);
                Ok(self.interrupt)
            }
            Some(RegAddr::SourceId) => Ok(self.source_id),
            Some(RegAddr::Data2) => Ok(self.data2),
            Some(RegAddr::DiagStatus) => Ok(self.diag_status),
            Some(RegAddr::Xfer2) => Ok((self.xfer >> 16) as u8),
            Some(RegAddr::Xfer1) => Ok((self.xfer >> 8) as u8),
            Some(RegAddr::Xfer0) => Ok(self.xfer as u8),
            _ => Ok(0),
        }
    }

    fn write_8(self: &mut Self, _bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        match FromPrimitive::from_usize(address) {
            Some(RegAddr::Data1) => {
                info!("(WRITE) DATA1 = {:02x}", value);
                self.data1 = value;
            }
            Some(RegAddr::Command) => {
                info!("(WRITE) COMMAND = {:02x}", value);
                self.command = value;
                self.handle_command();
            }
            Some(RegAddr::Control) => {
                info!("(WRITE) CONTROL = {:02x}", value);
                self.control = value;
            }
            Some(RegAddr::DestId) => {
                info!("(WRITE) DEST_ID = {:02x}", value);
                self.dest_id = value;
            }
            Some(RegAddr::Xfer2) => {
                info!("(WRITE) XFER2 = {:02x}", value);
                self.xfer &= !((value as u32) << 16);
                self.xfer |= (value as u32) << 16;
            }
            Some(RegAddr::Xfer1) => {
                info!("(WRITE) XFER2 = {:02x}", value);
                self.xfer &= !((value as u32) << 16);
                self.xfer |= (value as u32) << 16;
            }
            _ => {
                info!("(WRITE 8) addr={:08x} val={:02x}", address, value);
            }
        }

        Ok(())
    }
}
