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
use crate::cpu::set_irq;
use crate::err::BusError;
use crate::service::ServiceKey;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use tokio::time::Duration;

const HOST_ID: u8 = 7;
const DIAG_COMPLETE: u8 = 0x80;
const SCSI_INT: u8 = 3;

const ID_VALID: u8 = 0b10000000;

const AUX_COUNT_ZERO: u8 = 0b0000010;

#[allow(dead_code)]
const INT_FC: u8 = 0b00000001;
#[allow(dead_code)]
const INT_BUS_SVC: u8 = 0b00000010;
#[allow(dead_code)]
const INT_DIS: u8 = 0b00000100;
#[allow(dead_code)]
const INT_SELECTED: u8 = 0b00001000;

/// Register I/O Addresses
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

/// SCSI Bus Phase
#[derive(FromPrimitive)]
enum Phase {
    DataOut = 0,
    DataIn = 1,
    Command = 2,
    Status = 3,
    MessageOut = 6,
    MessageIn = 8,
}

/// Controller State
#[allow(dead_code)]
enum State {
    Disconnected,
    ConnectedAsTarget,
    ConnectedAsInitiator,
}

/// SCSI Commands
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
}

impl Scsi {
    pub fn new() -> Self {
        Scsi {
            state: State::Disconnected,
            address: 0,
            data1: 0,
            command: 0,
            control: 0,
            dest_id: 0,
            aux_stat: 0,
            id: HOST_ID,
            interrupt: 0,
            source_id: 0,
            data2: 0,
            diag_status: 0,
            xfer: 0,
        }
    }

    /// Controller Reset
    fn reset(&mut self) {
        info!("RESET");

        self.data1 = 0;
        self.control = 0;
        self.dest_id = 0;
        self.aux_stat = AUX_COUNT_ZERO;
        self.interrupt = 0;
        self.source_id = 0;
        self.data2 = 0;
        self.diag_status = DIAG_COMPLETE;
        self.xfer = 0;
    }

    /// Select (with or without attention)
    fn select(&mut self, bus: &mut Bus, atn: bool) {
        info!("SELECT (atn={}, timeout={})", atn, self.xfer);

        self.state = State::ConnectedAsInitiator;
        self.aux_stat = 0;
        self.interrupt = INT_FC;
        self.source_id = ID_VALID;

        bus.schedule(ServiceKey::Scsi, Duration::from_millis(750));
    }

    /// Process the last command.
    fn handle_command(&mut self, bus: &mut Bus) {
        let c = self.command;

        match FromPrimitive::from_u8(c) {
            Some(Command::ChipReset) => self.reset(),
            Some(Command::SelectWithAtn) => self.select(bus, true),
            Some(Command::SelectWithoutAtn) => self.select(bus, false),
            _ => {
                info!("Unhandled scsi command: 0x{:02x} (b{:06b})", c, c);
            }
        }
    }
}

impl IoDevice for Scsi {
    fn read_8(self: &mut Self, bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        info!("(READ) addr={:08x}", address);
        match FromPrimitive::from_usize(address) {
            Some(RegAddr::Data1) => Ok(self.data1),
            Some(RegAddr::Command) => Ok(self.command),
            Some(RegAddr::Control) => Ok(self.control),
            Some(RegAddr::DestId) => Ok(self.dest_id),
            Some(RegAddr::AuxStatus) => {
                info!("SCSI Aux Stat Read: {:02x}", self.aux_stat);
                Ok(self.aux_stat)
            }
            Some(RegAddr::Id) => {
                info!("SCSI ID: {}", self.id);
                Ok(self.id)
            }
            Some(RegAddr::Interrupt) => {
                let irq = self.interrupt;

                if irq == INT_FC {
                    // YES!!! THIS WORKS. IT WANTED COMMAND PHASE!!!!
                    // NEXT IS COMMAND 010100 = TRANSFER INFO
                    self.aux_stat = 0b00010000; // C/D=0,MSG=1
                    self.interrupt = 0x2;
                    bus.schedule(ServiceKey::Scsi, Duration::from_millis(750));
                }

                info!("SCSI Interrupt Read: ({:02x})", irq);

                Ok(irq)
            }
            Some(RegAddr::SourceId) => {
                info!("SCSI Source Id: {}", self.source_id);
                Ok(self.source_id)
            }
            Some(RegAddr::Data2) => {
                info!("SCSI Data2: {:02x}", self.data2);
                Ok(self.data2)
            }
            Some(RegAddr::DiagStatus) => {
                info!("SCSI Diag Status: {:02x}", self.diag_status);
                Ok(self.diag_status)
            }
            Some(RegAddr::Xfer2) => Ok((self.xfer >> 16) as u8),
            Some(RegAddr::Xfer1) => Ok((self.xfer >> 8) as u8),
            Some(RegAddr::Xfer0) => Ok(self.xfer as u8),
            _ => {
                info!("READ: Unhandled.");
                Ok(0)
            }
        }
    }

    fn write_8(self: &mut Self, bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        match FromPrimitive::from_usize(address) {
            Some(RegAddr::Data1) => {
                info!("(WRITE) DATA1 = {:02x}", value);
                self.data1 = value;
            }
            Some(RegAddr::Command) => {
                info!("(WRITE) COMMAND = {:02x}", value);
                self.command = value;
                self.handle_command(bus);
            }
            Some(RegAddr::Control) => {
                info!("(WRITE) CONTROL = {:02x}", value);
                self.control = value;
            }
            Some(RegAddr::DestId) => {
                info!("(WRITE) DEST_ID = {:02x}", value);
                self.dest_id = value;
            }
            Some(RegAddr::Id) => {
                info!("(WRITE) ID = {:02x}", value);
                self.id = value;
            }
            Some(RegAddr::Xfer2) => {
                info!("(WRITE) XFER2 = {:02x}", value);
                self.xfer &= !((value as u32) << 16);
                self.xfer |= (value as u32) << 16;
            }
            Some(RegAddr::Xfer1) => {
                info!("(WRITE) XFER1 = {:02x}", value);
                self.xfer &= !((value as u32) << 8);
                self.xfer |= (value as u32) << 8;
            }
            Some(RegAddr::Xfer0) => {
                info!("(WRITE) XFER0 = {:02x}", value);
                self.xfer &= !(value as u32);
                self.xfer |= value as u32;
            }
            _ => {
                info!("(WRITE 8) addr={:08x} val={:02x}", address, value);
            }
        }

        Ok(())
    }

    fn service(&mut self) {
        info!(">>> SCSI SERVICE ROUTINE BEING CALLED <<<");
        set_irq(SCSI_INT);
    }
}
