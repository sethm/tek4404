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
mod video;

#[macro_use]
extern crate lazy_static;
extern crate strum;
extern crate strum_macros;

use acia::{Acia, AciaServer, AciaState};
use bus::MemoryDevice;
use cpu::Cpu;
use log::*;
use mem::Memory;

use clap::Clap;
use tokio::time::{delay_for, Duration};

use std::error::Error;
use std::sync::{Arc, Mutex, RwLock};

use sdl2::pixels::PixelFormatEnum;

#[derive(Clap)]
#[clap(
    version = "0.1.0",
    author = "Seth Morabito <web@loomcom.com>",
    about = "Tektronix 4404 Emulator"
)]
struct Opts {
    #[clap(short, long)]
    bootrom: String,
    #[clap(short, long, default_value = "0.0.0.0", about = "Address to bind to")]
    address: String,
    #[clap(short, long, default_value = "9090", about = "Port to bind to")]
    port: String,
    #[clap(short, long, default_value = "2500", about = "CPU cycles per loop")]
    cycles: u32,
    #[clap(
        short,
        long,
        default_value = "20",
        about = "Idle time between CPU loops (in ms)"
    )]
    idle: u64,
    #[clap(
        short,
        long,
        default_value = "info",
        about = "Log level [io, trace, debug, info, warn, error]"
    )]
    loglvl: LogLevel,
}

///  Update the framebuffer vector based on current state of Video RAM
fn update_framebuffer(vm: &MemoryDevice, fb: &mut Vec<u8>) {
    let mut index: usize = 0;
    let mem = &vm.read().unwrap().mem;

    for b in mem {
        for i in 0..=7 {
            if (b >> 7 - i) & 1 == 1 {
                fb[index] = 0x51;
            } else {
                fb[index] = 0x3f;
            }
            index += 1;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    log::init(opts.loglvl.clone());

    info!("INITIALIZING");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Tektronix 4404", 1024, 1024)
        .build()
        .unwrap();

    let mut fb: Vec<u8> = vec![0; 1024 * 1024];
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_target(PixelFormatEnum::RGB332, 1024, 1024)
        .expect("Unable to create texture");

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut cpu = Cpu::new(opts.bootrom.as_str(), opts.cycles);
    let video_ram = Arc::new(RwLock::new(
        Memory::new(
            bus::VIDEO_RAM_START,
            bus::VIDEO_RAM_END,
            bus::VIDEO_RAM_SIZE,
            false,
        )
        .unwrap(),
    ));
    let acia_state = Arc::new(Mutex::new(AciaState::new()));
    let acia = Acia::new(acia_state.clone());

    bus::BUS
        .lock()
        .unwrap()
        .set_acia(Arc::new(RwLock::new(acia)));

    bus::BUS.lock().unwrap().set_video_ram(video_ram.clone());

    loop {
        tokio::join!(
            async {
                loop {
                    cpu.step();
                    delay_for(Duration::from_millis(opts.idle)).await;
                }
            },
            AciaServer::run(
                acia_state.clone(),
                opts.address.as_str(),
                opts.port.as_str()
            ),
            async {
                loop {
                    for event in event_pump.poll_iter() {
                        match event {
                            _ => {
                                println!("SOME OTHER EVENT");
                            }
                        }
                    }

                    update_framebuffer(&video_ram, &mut fb);
                    texture
                        .update(None, &fb, 1024)
                        .expect("Couldn't copy framebuffer to texture");

                    canvas.clear();
                    canvas
                        .copy(&texture, None, None)
                        .expect("Couldn't copy texture to canvas.");
                    canvas.present();

                    delay_for(Duration::from_millis(33)).await;
                }
            }
        );
    }
}
