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

const CMD_SIZE: usize = 12;

const ID_VALID: u8 = 0b10000000;

const AUX_CZ: u8 = 0b00000010;
#[allow(dead_code)]
const AUX_IO: u8 = 0b00001000;
const AUX_CD: u8 = 0b00010000;
#[allow(dead_code)]
const AUX_MSG: u8 = 0b00100000;

#[allow(dead_code)]
const INT_FC: u8 = 0b00000001;
#[allow(dead_code)]
const INT_BUS_SVC: u8 = 0b00000010;
#[allow(dead_code)]
const INT_DIS: u8 = 0b00000100;
#[allow(dead_code)]
const INT_SELECTED: u8 = 0b00001000;

//
// MSG  C/D  I/O
// -------------
//  0    0    0      Data Out
//  0    0    1      Data In
//  0    1    0      Command
//  0    1    1      Status
//  1    0    0      [unused]
//  1    0    1      [unused]
//  1    1    0      Message Out
//  1    1    1      Message In
#[allow(dead_code)]
const PHASE_DATO: u8 = 0b00000000;
#[allow(dead_code)]
const PHASE_DATI: u8 = 0b00001000;
const PHASE_CMND: u8 = 0b00010000;
#[allow(dead_code)]
const PHASE_STAT: u8 = 0b00011000;
#[allow(dead_code)]
const PHASE_MSGO: u8 = 0b00110000;
#[allow(dead_code)]
const PHASE_MSGI: u8 = 0b00111000;

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
    BusFree,
    // Arbitration is implied in BusFree -> Selected
    Selected,
    Command,
    Data,
    Message,
    Status,
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
    cmd: [u8; CMD_SIZE],
    cmd_ptr: usize,
}

impl Scsi {
    pub fn new() -> Self {
        Scsi {
            state: State::BusFree,
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
            cmd: [0; CMD_SIZE],
            cmd_ptr: 0,
        }
    }

    /// Controller Reset
    fn reset(&mut self) {
        info!("RESET");

        self.data1 = 0;
        self.control = 0;
        self.dest_id = 0;
        self.aux_stat = AUX_CZ;
        self.interrupt = 0;
        self.source_id = 0;
        self.data2 = 0;
        self.diag_status = DIAG_COMPLETE;
        self.xfer = 0;
        self.cmd_ptr = 0;
    }

    /// Select (with or without attention)
    // - Causes interrupt.
    // - Success: INT_REGISTER = 0b00000001, AUX_STATUS = C/D
    // - Failure: INT_REGISTER = 0b01000000, AUX_STATUS = C/D (I think?)
    fn select(&mut self, atn: bool) {
        info!("SELECT (atn={}, timeout={})", atn, self.xfer);

        self.state = State::Selected;

        self.aux_stat = AUX_CD;
        self.interrupt = INT_FC;
        self.source_id = ID_VALID | self.dest_id;

        schedule!(ServiceKey::Scsi, Duration::from_millis(750));
    }

    fn transfer_info(&mut self) {
        info!(
            "(COMMAND) Transfer Info. XFER={} ({:x})",
            self.xfer, self.xfer
        );
        self.cmd_ptr = 0;

        schedule!(ServiceKey::Scsi, Duration::from_millis(750));
    }

    /// Process the last command.
    fn handle_command(&mut self) {
        let c = self.command;

        match FromPrimitive::from_u8(c) {
            Some(Command::ChipReset) => self.reset(),
            Some(Command::SelectWithAtn) => self.select(true),
            Some(Command::SelectWithoutAtn) => self.select(false),
            Some(Command::TransferInfo) => self.transfer_info(),
            _ => {
                info!("Unhandled scsi command: 0x{:02x} (b{:06b})", c, c);
            }
        }
    }
}

impl IoDevice for Scsi {
    fn read_8(self: &mut Self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        match FromPrimitive::from_usize(address) {
            Some(RegAddr::Data1) => {
                info!("(READ) DATA1={:02x}", self.data1);
                Ok(self.data1)
            }
            Some(RegAddr::Command) => {
                info!("(READ) COMMAND={:02x}", self.command);
                Ok(self.command)
            }
            Some(RegAddr::Control) => {
                info!("(READ) CONTROL={:02x}", self.control);
                Ok(self.control)
            }
            Some(RegAddr::DestId) => {
                info!("(READ) DEST_ID={}", self.dest_id);
                Ok(self.dest_id)
            }
            Some(RegAddr::AuxStatus) => {
                info!("(READ) AUX_STAT: {:02x}", self.aux_stat);
                Ok(self.aux_stat)
            }
            Some(RegAddr::Id) => {
                info!("(READ) ID: {}", self.id);
                Ok(self.id)
            }
            Some(RegAddr::Interrupt) => {
                let irq = self.interrupt;
                info!("(READ) INTERRUPT: ({:02x})", irq);
                Ok(irq)
            }
            Some(RegAddr::SourceId) => {
                info!("(READ) SOURCE_ID: {}", self.source_id);
                Ok(self.source_id)
            }
            Some(RegAddr::Data2) => {
                info!("(READ) DATA2: {:02x}", self.data2);
                Ok(self.data2)
            }
            Some(RegAddr::DiagStatus) => {
                info!("(READ) DIAG_STATUS: {:02x}", self.diag_status);
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

    fn write_8(self: &mut Self, _bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        match FromPrimitive::from_usize(address) {
            Some(RegAddr::Address) => {
                info!("(WRITE) ADDRESS = {:02x}", value);
            }
            Some(RegAddr::Data1) => {
                info!("(WRITE)    CMD[{:02}] = {:02x}", self.cmd_ptr, value);
                self.cmd[self.cmd_ptr] = value;
                if self.cmd_ptr < CMD_SIZE {
                    self.cmd_ptr += 1;
                    if self.cmd_ptr == (self.xfer as usize) {
                        info!(
                            ">>> EXECUTING {} BYTE SCSI COMMAND: {:02x}",
                            self.xfer, self.cmd[0]
                        );
                    }
                }
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
            Some(RegAddr::Id) => {
                info!("(WRITE) ID = {:02x}", value);
                self.id = value;
            }
            Some(RegAddr::Xfer2) => {
                info!("(WRITE) XFER2 = {:02x}", value);
                self.xfer &= !(0xff << 16);
                self.xfer |= (value as u32) << 16;
            }
            Some(RegAddr::Xfer1) => {
                info!("(WRITE) XFER1 = {:02x}", value);
                self.xfer &= !(0xff << 8);
                self.xfer |= (value as u32) << 8;
            }
            Some(RegAddr::Xfer0) => {
                info!("(WRITE) XFER0 = {:02x}", value);
                self.xfer &= !(0xff);
                self.xfer |= value as u32;
            }
            _ => {
                info!("(WRITE 8) addr={:08x} val={:02x}", address, value);
            }
        }

        Ok(())
    }

    fn service(&mut self) {
        match self.state {
            State::BusFree => {
                info!("[BUSFREE->SELECTED]");
                self.state = State::Selected;
                self.interrupt = INT_FC;
                self.aux_stat = PHASE_CMND;
                // Schedule again for next phase transition
                schedule!(ServiceKey::Scsi, Duration::from_millis(750));
                set_irq(SCSI_INT);
            }
            // A target has been selected and we are an initiator.
            State::Selected => {
                // This will vary depending on what the selected target
                // device is. For now, we're pretending that ID 0 is a
                // hard disk device.
                info!("[SELECTED->COMMAND]");
                self.state = State::Command;
                self.interrupt = INT_BUS_SVC;
                self.aux_stat = PHASE_CMND;
                set_irq(SCSI_INT);
            }
            State::Command => {
                info!("[COMMAND->DATI]");
                self.state = State::Data;
                self.interrupt = INT_BUS_SVC;
                self.aux_stat = PHASE_DATO;
                set_irq(SCSI_INT);
            }
            _ => info!("[???->???]"),
        }
    }
}
