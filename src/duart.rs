//! Keyboard and RS-232 serial

use crate::bus::*;
use crate::err::*;

use std::collections::VecDeque;
use std::ops::RangeInclusive;
use std::result::Result;
use std::time::Duration;

const DELAY_RATES_A: [u32; 13] = [
    160000000, 72727272, 59259260, 40000000, 26666668, 13333334, 6666667, 7619047, 3333333,
    1666666, 1111111, 833333, 208333,
];

// Delay rates, in nanoseconds, selected when ACR[7] = 1
const DELAY_RATES_B: [u32; 13] = [
    106666672, 72727272, 59259260, 53333336, 26666668, 13333334, 6666667, 4000000, 3333333,
    1666666, 4444444, 833333, 416666,
];

// Port A: Keyboard Interface
// Port B: RS-232 Serial
const PORT_A: usize = 0;
const PORT_B: usize = 1;

//
// Registers
//
const MR12A: usize = 0x7b4000;
const CSRA: usize = 0x7b4002;
const CRA: usize = 0x7b4004;
const THRA: usize = 0x7b4006;
const IPCR_ACR: usize = 0x7b4008;
const ISR_MASK: usize = 0x7b400a;
const MR12B: usize = 0x7b4010;
const CSRB: usize = 0x7b4012;
const CRB: usize = 0x7b4014;
const THRB: usize = 0x7b4016;
const IP_OPCR: usize = 0x7b401a;
const OPBITS_SET: usize = 0x7b401c;
const OPBITS_RESET: usize = 0x7b401e;

//
// Port Configuration Bits
//
const CNF_ETX: u8 = 0x01;
const CNF_ERX: u8 = 0x02;

//
// Status Flags
//
const STS_RXR: u8 = 0x01;
const STS_TXR: u8 = 0x04;
const STS_TXE: u8 = 0x08;
const STS_OER: u8 = 0x10;
const STS_PER: u8 = 0x20;
const STS_FER: u8 = 0x40;

//
// Commands
//
const CMD_ERX: u8 = 0x01;
const CMD_DRX: u8 = 0x02;
const CMD_ETX: u8 = 0x04;
const CMD_DTX: u8 = 0x08;

//
// Interrupt Status Register
//
const ISTS_TAI: u8 = 0x01;
const ISTS_RAI: u8 = 0x02;
const ISTS_TBI: u8 = 0x10;
const ISTS_RBI: u8 = 0x20;
const ISTS_IPC: u8 = 0x80;

//
// Interrupt Masks
//
const KEYBOARD_INT: u8 = 0x04;
const TX_INT: u8 = 0x10;
const RX_INT: u8 = 0x20;

#[allow(dead_code)]
struct Port {
    mode: [u8; 2],
    stat: u8,
    conf: u8,
    rx_data: u8,
    tx_data: u8,
    mode_ptr: usize,
    rx_queue: VecDeque<u8>,
    tx_queue: VecDeque<u8>,
    char_delay: Duration,
}

impl Port {
    fn new() -> Port {
        Port {
            mode: [0; 2],
            stat: 0,
            conf: 0,
            rx_data: 0,
            tx_data: 0,
            mode_ptr: 0,
            rx_queue: VecDeque::new(),
            tx_queue: VecDeque::new(),
            char_delay: Duration::new(0, 1_000_000),
        }
    }
}

impl Default for Port {
    fn default() -> Self {
        Port::new()
    }
}

pub struct Duart {
    ports: [Port; 2],
    acr: u8,
    ipcr: u8,
    inprt: u8,
    outprt: u8,
    istat: u8,
    imr: u8,
    ivec: u8,
}

// NOTES:
//
// The serial DUART is used for keyboard and RS-232 serial
// communications.
//
// Output Port 3: Keyboard Reset.
//
// Output Port 4: Enable Keyboard Receive. While reading a character
// from the keyboard, the 4404 asserts OP4 high. The keyboard will
// not send while OP4 is high.
//
// Input Port 4: Keyboard Ready. The keyboard asserts IP4 HIGH when
// ready to receive a command.

impl Duart {
    pub fn new() -> Duart {
        Duart {
            ports: [Port::new(), Port::new()],
            acr: 0,
            ipcr: 0x40,
            inprt: 0x10, // IP4 high
            outprt: 0,
            istat: 0,
            imr: 0,
            ivec: 0,
        }
    }

    #[allow(dead_code)]
    fn handle_rx(&mut self, port: usize) {
        let mut ctx = &mut self.ports[port];

        let (istat, ivec) = match port {
            0 => (ISTS_RAI, RX_INT),
            _ => (ISTS_RBI, KEYBOARD_INT),
        };

        if let Some(c) = ctx.rx_queue.pop_back() {
            if ctx.conf & CNF_ERX != 0 {
                ctx.rx_data = c;
                ctx.stat |= STS_RXR;
                self.istat |= istat;
                self.ivec |= ivec;
            }
        }
    }

    #[allow(dead_code)]
    fn handle_tx(&mut self, port: usize) {
        let mut ctx = &mut self.ports[port];

        let (tx_istat, rx_istat) = match port {
            0 => (ISTS_TAI, ISTS_RAI),
            _ => (ISTS_TBI, ISTS_RBI),
        };

        if (ctx.conf & CNF_ETX) != 0 && (ctx.stat & STS_TXR) == 0 && (ctx.stat & STS_TXE) == 0 {
            let c = ctx.tx_data;
            ctx.stat |= STS_TXR;
            ctx.stat |= STS_TXE;
            self.istat |= tx_istat;
            // Only RS232 transmit generates an interrupt.
            if port == PORT_B {
                self.ivec |= TX_INT;
            }
            if (ctx.mode[1] >> 6) & 3 == 0x2 {
                // Loopback Mode.
                ctx.rx_data = c;
                ctx.stat |= STS_RXR;
                self.istat |= rx_istat;
                self.ivec |= RX_INT;
            } else {
                ctx.tx_queue.push_front(c);
            }
        }
    }

    pub fn handle_command(&mut self, cmd: u8, port: usize) {
        if cmd == 0 {
            return;
        }

        let mut ctx = &mut self.ports[port];

        info!("DUART: Port {} Command {:02x}", port, cmd);

        // Enable or disable transmitter
        if cmd & CMD_DTX != 0 {
            ctx.conf &= !CNF_ETX;
            ctx.stat &= !STS_TXR;
            ctx.stat &= !STS_TXE;
            if port == PORT_A {
                self.ivec &= !TX_INT;
                self.istat &= !ISTS_TAI;
            }
        } else if cmd & CMD_ETX != 0 {
            ctx.conf |= CNF_ETX;
            ctx.stat |= STS_TXR;
            ctx.stat |= STS_TXE;
            if port == PORT_A {
                self.istat |= ISTS_TAI;
                self.ivec |= TX_INT;
            }
        }

        // Enable or disable receiver
        if cmd & CMD_DRX != 0 {
            ctx.conf &= !CNF_ERX;
            ctx.stat &= !STS_RXR;
            if port == PORT_A {
                self.ivec &= !RX_INT;
                self.istat &= !ISTS_RAI;
            } else {
                self.ivec &= !KEYBOARD_INT;
                self.istat &= !ISTS_RBI;
            }
        } else if cmd & CMD_ERX != 0 {
            ctx.conf |= CNF_ERX;
            ctx.stat |= STS_RXR;
        }

        // Extra commands
        match (cmd >> 4) & 7 {
            1 => ctx.mode_ptr = 0,
            2 => {
                ctx.stat |= STS_RXR;
                ctx.conf |= CNF_ERX;
            }
            3 => {
                ctx.stat |= STS_TXR;
                ctx.stat |= STS_TXE;
                ctx.conf &= !CNF_ETX;
            }
            4 => ctx.stat &= !(STS_FER | STS_PER | STS_OER),
            _ => {}
        }
    }
}

impl IoDevice for Duart {
    fn range(&self) -> RangeInclusive<usize> {
        DUART_START..=DUART_END
    }

    fn read_8(&mut self, _bus: &mut Bus, address: usize) -> Result<u8, BusError> {
        match address {
            MR12A => {
                let mut ctx = &mut self.ports[PORT_A];
                let val = ctx.mode[ctx.mode_ptr];
                ctx.mode_ptr = (ctx.mode_ptr + 1) % 2;
                info!("DUART(READ): MR12A: val={:02x}", val);
                Ok(val)
            }
            CSRA => {
                info!("DUART(READ): CSRA: val={:02x}", self.ports[PORT_A].stat);
                Ok(self.ports[PORT_A].stat)
            }
            THRA => {
                let mut ctx = &mut self.ports[PORT_A];
                ctx.stat &= !STS_RXR;
                self.istat &= !ISTS_RAI;
                self.ivec &= !RX_INT;
                info!("DUART(READ): THRA: val={:02x}", ctx.rx_data);
                Ok(ctx.rx_data)
            }
            IPCR_ACR => {
                let result = self.ipcr;
                self.ipcr &= !0x0f;
                self.ivec = 0;
                self.istat &= !ISTS_IPC;
                info!("DUART(READ): IPCR_ACR: val={:02x}", result);
                Ok(result)
            }
            ISR_MASK => {
                info!("DUART(READ): ISR_MASK: val={:02x}", self.istat);
                Ok(self.istat)
            }
            MR12B => {
                let mut ctx = &mut self.ports[PORT_B];
                let val = ctx.mode[ctx.mode_ptr];
                ctx.mode_ptr = (ctx.mode_ptr + 1) % 2;
                info!("DUART(READ): MR12B: val={:02x}", val);
                Ok(val)
            }
            CSRB => {
                info!("DUART(READ): CSRB: val={:02x}", self.ports[PORT_B].stat);
                Ok(self.ports[PORT_B].stat)
            }
            THRB => {
                let mut ctx = &mut self.ports[PORT_B];
                ctx.stat &= !STS_RXR;
                self.istat &= !ISTS_RBI;
                self.ivec &= !KEYBOARD_INT;
                info!("DUART(READ): THRB: val={:02x}", ctx.rx_data);
                Ok(ctx.rx_data)
            }
            IP_OPCR => {
                info!("DUART(READ): IP_OPCR: val={:02x}", self.inprt);
                Ok(self.inprt)
            }
            _ => {
                info!("DUART(READ): Unhandled. addr={:08x}", address);
                Ok(0)
            }
        }
    }

    fn read_16(self: &mut Self, bus: &mut Bus, address: usize) -> Result<u16, BusError> {
        match address {
            MR12A => {
                let ctx = &self.ports[PORT_A];
                let lo: u16 = ctx.mode[0] as u16;
                let hi: u16 = (ctx.mode[1] as u16) << 8;
                info!("DUART(READ16): MR12A: val={:02x}", hi | lo);
                Ok(hi | lo)
            }
            _ => {
                let b = self.read_8(bus, address)?;
                Ok(b as u16)
            }
        }
    }

    fn write_8(self: &mut Self, _bus: &mut Bus, address: usize, value: u8) -> Result<(), BusError> {
        match address {
            MR12A => {
                let mut ctx = &mut self.ports[PORT_A];
                ctx.mode[ctx.mode_ptr] = value;
                ctx.mode_ptr = (ctx.mode_ptr + 1) % 2;
                info!("DUART(WRITE): MR12A: val={:02x}", value);
            }
            CSRA => {
                // Set the baud rate.
                let baud_bits: usize = ((value >> 4) & 0xf) as usize;
                let delay = if self.acr & 0x80 == 0 {
                    DELAY_RATES_A[baud_bits]
                } else {
                    DELAY_RATES_B[baud_bits]
                };
                let mut ctx = &mut self.ports[PORT_A];
                ctx.char_delay = Duration::new(0, delay);
                info!("DUART(WRITE): CSRA: val={:02x}", value);
            }
            CRA => {
                self.handle_command(value, PORT_A);
                info!("DUART(WRITE): CRA: val={:02x}", value);
            }
            THRA => {
                let mut ctx = &mut self.ports[PORT_A];
                ctx.tx_data = value;
                // Update state. Since we're transmitting, the
                // transmitter buffer is not empty.  The actual
                // transmit will happen in the 'service' function.
                ctx.stat &= !(STS_TXE | STS_TXR);
                self.istat &= !ISTS_TAI;
                self.ivec &= !TX_INT;
                info!("DUART(WRITE): THRA: val={:02x}", value);
            }
            IPCR_ACR => {
                self.acr = value;
                info!("DUART(WRITE): IPCR_ACR: val={:02x}", value);
            }
            ISR_MASK => {
                self.imr = value;
                info!("DUART(WRITE): ISR_MASK: val={:02x}", value);
            }
            MR12B => {
                let mut ctx = &mut self.ports[PORT_B];
                ctx.mode[ctx.mode_ptr] = value;
                ctx.mode_ptr = (ctx.mode_ptr + 1) % 2;
                info!("DUART(WRITE): MR12B: val={:02x}", value);
            }
            CRB => {
                self.handle_command(value, PORT_B);
                info!("DUART(WRITE): CRB: val={:02x}", value);
            }
            THRB => {
                // Keyboard transmit requires special handling,
                // because the only things the terminal transmits to
                // the keyboard are status requests, or keyboard beep
                // requests. We ignore status requests, and only
                // put beep requests into the queue.
                let mut ctx = &mut self.ports[PORT_B];

                if (value & 0x08) != 0 {
                    ctx.tx_data = value;
                    ctx.stat &= !(STS_TXE | STS_TXR);
                    self.istat &= !ISTS_TBI;
                }

                info!("DUART(WRITE): THRB: val={:02x}", value);
            }
            IP_OPCR => {
                info!("DUART(WRITE): IP_OPCR: val={:02x}", value);
            }
            OPBITS_SET => {
                self.outprt |= value;
                info!("DUART(WRITE): OPBITS_SET: val={:02x}", value);
            }
            OPBITS_RESET => {
                self.outprt &= !value;
                info!("DUART(WRITE): OPBITS_RESET: val={:02x}", value);
                if value & 0x8 != 0 {
                    // Keyboard Reset.
                    // Transmit something!
                    info!("KEYBOARD RESET.");
                    let mut ctx = &mut self.ports[PORT_A];
                    ctx.rx_data = 0xf0; // Reset
                    ctx.stat |= STS_RXR;
                }
            }
            _ => {
                info!(
                    "DUART(WRITE): UNHANDLED: addr={:08x} val={:02x}",
                    address, value
                );
            }
        }

        Ok(())
    }
}
