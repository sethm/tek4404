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

#[macro_use]
mod log;
mod acia;
mod bus;
mod cpu;
mod err;
mod mem;
mod sound;

#[macro_use]
extern crate lazy_static;
extern crate strum;
extern crate strum_macros;

use crate::cpu::Cpu;
use crate::log::*;
use acia::Acia;

use std::error::Error;

use clap::Clap;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[allow(dead_code)]
#[derive(Clap)]
struct Opts {
    #[clap(short, long)]
    bootrom: String,
    #[clap(short, long, default_value = "100")]
    steps: u32,
    #[clap(short, long, default_value = "info")]
    loglvl: LogLevel,
}

#[allow(dead_code)]
async fn acia_listener() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Spawning Debug ACIA listener on 127.0.0.1:9090");
    let mut listener = TcpListener::bind("127.0.0.1:9090").await.expect("");
    loop {
        let (mut socket, _) = listener.accept().await.expect("");
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}

async fn run_cpu(cpu: &mut Cpu) {
    cpu.step().await;
}

use std::sync::{Arc, RwLock};

async fn do_io(acia: &mut Arc<RwLock<Acia>>) {
    acia.write().unwrap().do_io().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    log::init(opts.loglvl.clone());

    info!("INITIALIZING");

    let mut cpu = Cpu::new(opts.bootrom.as_str(), opts.steps);
    let mut acia = bus::BUS.lock().unwrap().acia.clone().unwrap().clone();

    loop {
        tokio::join!(run_cpu(&mut cpu), do_io(&mut acia),);
    }
}
