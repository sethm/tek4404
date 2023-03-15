//! Debugging serial port
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
use arraydeque::{ArrayDeque, Saturating};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use crate::bus::*;
use crate::err::*;

const DATA_REG: usize = 0x78c000;
const STAT_REG: usize = 0x78c002;
const CMD_REG: usize = 0x78c004;
const CTRL_REG: usize = 0x78c006;

pub type SharedAciaState = Arc<Mutex<AciaState>>;

/// A Telnet protocol handshake.
///
/// This will negotiate what features we support when a Telnet client
/// connects. This forces character mode and tells the client we will
/// echo input. (IAC WILL ECHO, IAC WILL SUPPRESS-GO-AHEAD, IAC WONT
/// LINEMODE)
const HANDSHAKE: [u8; 9] = [255, 251, 1, 255, 251, 3, 255, 252, 34];

enum TelnetState {
    Data,
    OptionName,
    OptionValue,
}

/// State shared between the ACIA and the ACIA Telnet Server
pub struct AciaState {
    ts: TelnetState,
    pub connected: bool,
    pub tx_data: ArrayDeque<[u8; 8], Saturating>,
    pub rx_data: ArrayDeque<[u8; 8], Saturating>,
    pub waker: Option<Waker>,
}

impl AciaState {
    pub fn new() -> Self {
        AciaState {
            ts: TelnetState::Data,
            connected: false,
            tx_data: ArrayDeque::new(),
            rx_data: ArrayDeque::new(),
            waker: None,
        }
    }
}

/// Future that will asynchronously read one byte from an Acia's
/// Transmit Data register.
pub struct AciaTransmit {
    state: SharedAciaState,
}

impl AciaTransmit {
    pub fn new(state: SharedAciaState) -> Self {
        AciaTransmit { state }
    }
}

impl Future for AciaTransmit {
    type Output = Result<u8, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.get_mut().state.lock().unwrap();

        // Tell the caller that we're no longer connected.
        if !state.connected {
            return Poll::Ready(Err(()));
        }

        match state.tx_data.pop_front() {
            Some(c) => Poll::Ready(Ok(c)),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

pub struct AciaServer {}

impl AciaServer {
    pub async fn run(state: SharedAciaState, bind: &str, port: &str) {
        let addr = format!("{bind}:{port}");

        info!("Listening for ACIA debug connections on {}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let state = state.clone();
            let (mut socket, peer) = listener.accept().await.unwrap();

            if state.lock().unwrap().connected {
                socket
                    .write_all(b"Already connected. Goodbye.\r\n")
                    .await
                    .expect("ACIA socket write failed.");
                socket
                    .shutdown()
                    .await
                    .expect("ACIA socket shutdown failed.");
                continue;
            }

            socket
                .write_all(b"*** Welcome to the Tektronix 4404 simulator Debug ACIA ***\r\n")
                .await
                .expect("ACIA socket write failed.");

            tokio::spawn(async move {
                AciaServer::process(state, socket, peer).await;
            });
        }
    }

    async fn process(state: SharedAciaState, mut socket: TcpStream, peer: SocketAddr) {
        info!("Accepted connection from {}", peer);
        state.lock().unwrap().connected = true;

        socket.write_all(&HANDSHAKE).await.unwrap();

        let (mut reader, mut writer) = socket.into_split();

        let read_state = state.clone();
        let write_state = state.clone();

        tokio::join!(
            async move {
                let mut buf: [u8; 32] = [0; 32];
                loop {
                    let n = match reader.read(&mut buf).await {
                        Ok(n) if n == 0 => {
                            error!("Read 0 bytes... bye.");
                            write_state.lock().unwrap().connected = false;
                            return;
                        }
                        Ok(n) => n,
                        Err(e) => {
                            error!("failed to read from socket; err = {:?}", e);
                            write_state.lock().unwrap().connected = false;
                            return;
                        }
                    };
                    for n in &buf[0..n] {
                        let write_state = &mut write_state.lock().unwrap();

                        match write_state.ts {
                            TelnetState::Data => {
                                if *n == 255 {
                                    write_state.ts = TelnetState::OptionName;
                                } else {
                                    info!(">>> input (tcp to acia): queueing {:02x}", n);
                                    let _ = write_state.rx_data.push_back(*n);
                                }
                            }
                            TelnetState::OptionName => {
                                write_state.ts = TelnetState::OptionValue;
                            }
                            TelnetState::OptionValue => {
                                write_state.ts = TelnetState::Data;
                            }
                        }
                    }
                }
            },
            async move {
                let mut buf: [u8; 1] = [0; 1];
                while let Ok(c) = AciaTransmit::new(read_state.clone()).await {
                    info!("<<< output (acia to tcp): sending out {:02x}", c);
                    buf[0] = c;
                    if let Err(e) = writer.write_all(&buf).await {
                        error!("failed to write to socket; err = {:?}", e);
                        read_state.lock().unwrap().connected = false;
                        return;
                    }
                }
                error!("No longer connected...");
                read_state.lock().unwrap().connected = false;
            }
        );
    }
}

/// The ACIA itself
pub struct Acia {
    pub state: SharedAciaState,
    data: u8,
    control: u8,
    command: u8,
    status: u8,
}

impl Acia {
    pub fn new(state: SharedAciaState) -> Acia {
        Acia {
            state,
            data: 0,
            control: 0,
            command: 0,
            status: 0,
        }
    }

    fn handle_command(&mut self) {
        self.status = 0;
    }
}

impl IoDevice for Acia {
    fn read_8(&mut self, _: &mut Bus, address: usize) -> std::result::Result<u8, BusError> {
        let result = match address {
            DATA_REG => {
                if let Some(c) = self.state.lock().unwrap().rx_data.pop_front() {
                    self.data = c;
                }

                debug!(
                    "ACIA Receive: ({})",
                    match self.data {
                        0x20..=0x7f => self.data as char,
                        _ => '.',
                    }
                );

                self.data
            }
            STAT_REG => {
                let mut result = self.status;
                let state = self.state.lock().unwrap();

                if !state.rx_data.is_empty() {
                    result |= 0x8;
                }

                if state.tx_data.is_empty() {
                    result |= 0x10;
                }

                if !state.connected {
                    result |= 0x60; // DCD/DSR high (not connected)
                }

                result
            }
            CMD_REG => self.command,
            CTRL_REG => self.control,
            _ => 0,
        };
        Ok(result)
    }

    fn write_8(
        &mut self,
        _: &mut Bus,
        address: usize,
        data: u8,
    ) -> std::result::Result<(), BusError> {
        match address {
            DATA_REG => {
                debug!(
                    "ACIA Transmit: ({})",
                    match data {
                        0x20..=0x7f => data as char,
                        _ => '.',
                    }
                );
                self.data = data;
                let _ = self.state.lock().unwrap().tx_data.push_back(data);
                if let Some(waker) = &self.state.lock().unwrap().waker {
                    waker.wake_by_ref();
                }
            }
            STAT_REG => {
                debug!("Clearing ACIA state.");
                let mut shared_state = self.state.lock().unwrap();
                shared_state.tx_data.clear();
                shared_state.rx_data.clear();
                self.data = 0;
            }
            CMD_REG => {
                info!("ACIA Command {:02x}", data);
                self.command = data;
                self.handle_command();
            }
            CTRL_REG => {
                info!("ACIA Control {:02x}", data);
                self.control = data
            }
            _ => {}
        }
        Ok(())
    }
}
