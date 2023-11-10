//! NCR 5385 SCSI controller
use std::time::Duration;

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

use log::info;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

/// The hardwired SCSI ID of the host controller
const HOST_ID: u8 = 7;

/// The SCSI controller CPU interrupt level
const SCSI_INT: u8 = 3;

/// Aux Status Flags
const AUX_IO: u8 = 0x08; // "Input / Output"
const AUX_CD: u8 = 0x10; // "Command / Data"
const AUX_MSG: u8 = 0x20; // "Message"
const AUX_DF: u8 = 0x80; // Data register full

/// Interrupt Flags
const INT_FC: u8 = 0x01; // "Function Complete"
const INT_BUS: u8 = 0x02; // "Bus Service"

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

#[allow(dead_code)]
enum ControllerState {
    Disconnected,
    Target,
    Initiator,
}

/// SCSI BUS Commands
#[derive(FromPrimitive, Debug)]
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

#[derive(Copy, Clone, Debug)]
enum ScsiDeviceState {
    Unselected,
    Selected,
    Command,
    DataOut,
}

#[derive(Copy, Clone, Debug)]
struct ScsiDevice {
    state: ScsiDeviceState,
}

impl ScsiDevice {
    fn reset(&mut self) {
        self.state = ScsiDeviceState::Unselected
    }
}

impl Default for ScsiDevice {
    fn default() -> ScsiDevice {
        ScsiDevice {
            state: ScsiDeviceState::Unselected,
        }
    }
}

pub struct Scsi {
    address: u16,
    address_msb: bool,
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
    cmd_ptr: usize,
    controller_state: ControllerState,
    scsi_cmd: [u8; 16],
    atn: bool,
    devices: [ScsiDevice; 8],
}

impl Scsi {
    pub fn new() -> Self {
        Scsi {
            address: 0,
            address_msb: false,
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
            cmd_ptr: 0,
            scsi_cmd: [0; 16],
            controller_state: ControllerState::Disconnected,
            atn: false,
            devices: [ScsiDevice::default(); 8],
        }
    }

    fn reset(&mut self) {
        info!("COMMAND RESET.");

        self.address = 0;
        self.address_msb = false;
        self.data1 = 0;
        self.command = 0;
        self.control = 0;
        self.dest_id = 0;
        self.aux_stat = 0x02; // From datasheet
        self.id = HOST_ID;
        self.interrupt = 0;
        self.source_id = 0x07; // From datasheet
        self.data2 = 0;
        self.diag_status = 0x80; // From datasheet
        self.xfer = 0;
        self.cmd_ptr = 0;
        self.scsi_cmd.iter_mut().for_each(|d| *d = 0);
        self.controller_state = ControllerState::Disconnected;
        self.atn = false;

        self.devices.iter_mut().for_each(|d| d.reset());
    }

    fn disconnect(&mut self) {
        info!("COMMAND DISCONNECT. Probably ignoring.");
    }

    /// Select a target device
    fn select(&mut self, atn: bool) {
        info!("COMMAND SELECT. atn={}", atn);
        self.controller_state = ControllerState::Initiator;
        self.atn = atn;

        // Set the target to command
        self.devices[(self.dest_id & 0x7) as usize].state = ScsiDeviceState::Selected;

        self.interrupt = INT_FC;
        self.aux_stat = AUX_CD; // I/O=0, C/D=1, MSG=0 == Command
        self.source_id = 0x80 | self.dest_id; // Bit 7 indicates valid ID
                                              // from destination device
        schedule!(ServiceKey::Scsi, Duration::from_millis(250));
        set_irq(SCSI_INT);
    }

    fn transfer_info(&mut self) {
        info!("COMMAND TRANSFER INFO.");
        self.cmd_ptr = 0;

        schedule!(ServiceKey::Scsi, Duration::from_millis(100));
    }

    fn transfer_pad(&mut self) {
        info!("COMMAND TRANSFER PAD.");
    }

    fn handle_command(&mut self, command: u8) -> Result<(), BusError> {
        let dma_mode = command & 0x80 == 0x80;
        let sbt = command & 0x40 == 0x40;

        if let Some(cmd) = Command::from_u8(command & 0x1f) {
            info!(
                "Handle Command: DMA_MODE={}, SBT={}, CMD={:?}",
                dma_mode, sbt, cmd
            );
            match cmd {
                Command::ChipReset => self.reset(),
                Command::Disconnect => self.disconnect(),
                Command::SelectWithoutAtn => self.select(false),
                Command::SelectWithAtn => self.select(true),
                Command::TransferInfo => self.transfer_info(),
                Command::TransferPad => self.transfer_pad(),
                _ => info!("Command {:?} not yet handled.", cmd),
            }
        } else {
            info!("What kind of command is 0x{:02x} ???", command);
        };

        Ok(())
    }
}

impl IoDevice for Scsi {
    fn read_8(&mut self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        match FromPrimitive::from_usize(address) {
            Some(RegAddr::Data1) => {
                info!("(READ) DATA1: 0x{:02x}", self.data1);
                self.aux_stat &= !AUX_DF;
                Ok(self.data1)
            }
            Some(RegAddr::Command) => {
                info!("(READ) COMMAND: 0x{:02x}", self.command);
                Ok(self.command)
            }
            Some(RegAddr::Control) => {
                info!("(READ) CONTROL: 0x{:02x}", self.control);
                Ok(self.control)
            }
            Some(RegAddr::DestId) => {
                info!("(READ) DEST_ID: {}", self.dest_id);
                Ok(self.dest_id)
            }
            Some(RegAddr::AuxStatus) => {
                info!("(READ) AUX_STAT: 0x{:02x}", self.aux_stat);
                Ok(self.aux_stat)
            }
            Some(RegAddr::Id) => {
                info!("(READ) ID: {}", self.id);
                Ok(self.id)
            }
            Some(RegAddr::Interrupt) => {
                info!("(READ) INTERRUPT: 0x{:02x}", self.interrupt);
                Ok(self.interrupt)
            }
            Some(RegAddr::SourceId) => {
                info!("(READ) SOURCE_ID: {}", self.source_id);
                Ok(self.source_id)
            }
            Some(RegAddr::Data2) => {
                info!("(READ) DATA2: 0x{:02x}", self.data2);
                Ok(self.data2)
            }
            Some(RegAddr::DiagStatus) => {
                info!("(READ) DIAG_STATUS: 0x{:02x}", self.diag_status);
                Ok(self.diag_status)
            }
            Some(RegAddr::Xfer2) => {
                info!(
                    "(READ) XFER2: 0x{:02x} (xfer=0x{:06x})",
                    (self.xfer >> 16) as u8,
                    self.xfer
                );
                Ok((self.xfer >> 16) as u8)
            }
            Some(RegAddr::Xfer1) => {
                info!(
                    "(READ) XFER1: 0x{:02x} (xfer=0x{:06x})",
                    (self.xfer >> 8) as u8,
                    self.xfer
                );
                Ok((self.xfer >> 8) as u8)
            }
            Some(RegAddr::Xfer0) => {
                info!(
                    "(READ) XFER1: 0x{:02x} (xfer=0x{:06x})",
                    self.xfer as u8, self.xfer
                );
                Ok(self.xfer as u8)
            }
            _ => {
                info!("READ: Unhandled.");
                Ok(0)
            }
        }
    }

    fn write_8(&mut self, _bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        match FromPrimitive::from_usize(address) {
            Some(RegAddr::Address) => {
                if self.address_msb {
                    self.address &= 0x00ff;
                    self.address |= (value as u16) << 8;
                    self.address_msb = false;
                } else {
                    self.address &= 0xff00;
                    self.address |= value as u16;
                    self.address_msb = true;
                }
                // I believe writes to this are: LSB first, then MSB.
                info!(
                    "(WRITE) ADDRESS = {:02x} (address now is: {:04x})",
                    value, self.address
                );
            }
            Some(RegAddr::Data1) => {
                info!("(WRITE)    CMD[{:02}] = {:02x}", self.cmd_ptr, value);
                self.scsi_cmd[self.cmd_ptr] = value;
                self.cmd_ptr += 1;
            }
            Some(RegAddr::Command) => {
                info!("(WRITE) COMMAND = {:02x}", value);
                self.cmd_ptr = 0;
                return self.handle_command(value);
            }
            Some(RegAddr::Control) => {
                let parity = value & 0x4 == 0x4;
                let reselect = value & 0x2 == 0x2;
                let select = value & 0x1 == 0x1;
                info!(
                    "(WRITE) CONTROL: Parity={}, Reselect={}, Select={}",
                    parity, reselect, select
                );
                self.control = value;
            }
            Some(RegAddr::DestId) => {
                info!("(WRITE) DEST_ID = {:02x}", value);
                self.dest_id = value;
            }
            Some(RegAddr::Id) => {
                info!("(WRITE) ID = {:02x}", value);
            }
            Some(RegAddr::Xfer2) => {
                self.xfer &= 0x00ffff;
                self.xfer |= (value as u32) << 16;
                info!("(WRITE) XFER2 = {:02x} (xfer={:06x})", value, self.xfer);
            }
            Some(RegAddr::Xfer1) => {
                self.xfer &= 0xff00ff;
                self.xfer |= (value as u32) << 8;
                info!("(WRITE) XFER1 = {:02x} (xfer={:06x})", value, self.xfer);
            }
            Some(RegAddr::Xfer0) => {
                self.xfer &= 0xffff00;
                self.xfer |= value as u32;
                info!("(WRITE) XFER0 = {:02x} (xfer={:06x})", value, self.xfer);
            }
            _ => {
                info!("(WRITE 8) addr={:08x} val={:02x}", address, value);
            }
        }

        Ok(())
    }

    fn service(&mut self) {
        info!(
            "Servicing SCSI Controller. Current Target ID={}",
            self.dest_id
        );

        let id = (self.dest_id & 0x7) as usize;
        let cur_state = self.devices[id].state;

        match cur_state {
            ScsiDeviceState::Selected => {
                info!("[service] Selected -> Command (dest_id={})", self.dest_id);

                self.interrupt = INT_BUS;
                self.aux_stat = AUX_CD; // "COMMAND" phase, initiator to target

                self.devices[id].state = ScsiDeviceState::Command;
                set_irq(SCSI_INT);
            }
            ScsiDeviceState::Command => {
                info!("[service] Command -> Data Out (dest_id={})", self.dest_id);
                info!("[service]  ... cmd_ptr={}", self.cmd_ptr);
                info!(
                    "[service]  ... cmd={:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                    self.scsi_cmd[0],
                    self.scsi_cmd[1],
                    self.scsi_cmd[2],
                    self.scsi_cmd[3],
                    self.scsi_cmd[4],
                    self.scsi_cmd[5],
                );

                self.data1 = 0;

                self.interrupt = INT_BUS | INT_FC;

                if self.aux_stat & (AUX_MSG | AUX_CD | AUX_IO) == (AUX_CD | AUX_IO) {
                    // STATUS -> DATA_IN
                    info!(">>> aux_stat == {:02x}, Switching to AUX_IO", self.aux_stat);
                    self.aux_stat = AUX_DF | AUX_IO;
                } else {
                    // DATA_IN -> STATUS
                    info!(
                        ">>> aux_stat == {:02x}, Switching to AUX_DF | AUX_CD | AUX_IO",
                        self.aux_stat
                    );
                    self.aux_stat = AUX_DF | AUX_CD | AUX_IO;
                }

                set_irq(SCSI_INT);
            }
            ScsiDeviceState::DataOut => {
                info!("[service] Data Out (dest_id={})", self.dest_id);

                self.interrupt = INT_BUS | INT_FC;
                info!(">>> UHHHH WHAT");
                self.aux_stat = AUX_DF;
            }
            _ => {
                info!(
                    "[service] Unhandled State: {:?} (dest_id={})",
                    cur_state, self.dest_id
                );
            }
        }
    }
}
