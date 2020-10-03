//! Tektronix 4404 Emulator.
//!
//! # About
//!
//! The Tektronix 4404 was a mid 1980s AI workstation that ran Smalltalk-80
//! and LISP natively. Built around a 68010 CPU, it supported 1-2 MB of RAM,
//! a 45MB SCSI hard disk, serial and parallel IO, and had a 640x480 bitmapped
//! display backed by a 1024x1024 pixel 2-bit framebuffer.
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
#[macro_use]
mod log;
mod acia;
mod bus;
mod cal;
mod cpu;
mod duart;
mod err;
mod fpu;
mod mem;
mod mmu;
mod mouse;
mod scsi;
mod sound;
mod timer;
mod video;

#[macro_use]
extern crate lazy_static;
extern crate strum;
extern crate strum_macros;

use acia::{Acia, AciaServer, AciaState};
use bus::MemoryDevice;
use cpu::Cpu;
use duart::Duart;
use log::*;
use mem::Memory;
use video::Video;

use clap::Clap;
use tokio::time::{delay_for, Duration};

use std::error::Error;
use std::sync::{Arc, Mutex, RwLock};

use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

/// Number of 68010 machine cycles to execute on each CPU step.
const CYCLES_PER_LOOP: i32 = 60;
/// Framebuffer width
const FB_WIDTH: u32 = 1024;
/// Framebuffer height
const FB_HEIGHT: u32 = 1024;
/// Visible window width
const WINDOW_WIDTH: u32 = 640;
/// Visible window height
const WINDOW_HEIGHT: u32 = 480;

/// Clap options parsed from the command line
#[derive(Clap)]
#[clap(
    version = "0.1.0",
    author = "Seth Morabito <web@loomcom.com>",
    about = "Tektronix 4404 Emulator"
)]
struct Opts {
    /// The path to the 32KB boot ROM image
    #[clap(short, long)]
    bootrom: String,
    /// The address to bind the debug ACIA telnet server to
    #[clap(short, long, default_value = "0.0.0.0", about = "Address to bind to")]
    address: String,
    /// The port to bind the debug ACIA telnet server to
    #[clap(short, long, default_value = "9090", about = "Port to bind to")]
    port: String,
    /// The number of CPU steps to take on each loop
    #[clap(
        short,
        long,
        default_value = "10000",
        about = "CPU execution steps per loop"
    )]
    steps: u32,
    /// The amount of time to idle between loops
    #[clap(
        short,
        long,
        default_value = "20",
        about = "Idle time between CPU loops (in ms)"
    )]
    idle: u64,
    /// The level of logging to display
    #[clap(
        short,
        long,
        default_value = "info",
        about = "Log level [io|trace|debug|info|error|none]"
    )]
    loglvl: LogLevel,
}

/// Update the framebuffer vector based on current state of Video RAM
//
// TODO: It makes much more sense to implement a special memory device
//       for video RAM that reads and writes each pixel as an RGB332 byte,
//       then we don't need this expensive step.
fn update_framebuffer(vm: &MemoryDevice, fb: &mut Vec<u8>) {
    let mut index: usize = 0;
    let mem = &vm.read().unwrap().mem;

    for b in mem {
        for i in 0..=7 {
            if (b >> 7 - i) & 1 == 1 {
                fb[index] = 0;
            } else {
                fb[index] = 255;
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

    let mut cpu = Cpu::new(opts.bootrom.as_str());
    let video_ram = Arc::new(RwLock::new(
        Memory::new(bus::VRAM_START, bus::VRAM_END, bus::VRAM_SIZE, false).unwrap(),
    ));
    let acia_state = Arc::new(Mutex::new(AciaState::new()));
    let acia = Arc::new(RwLock::new(Acia::new(acia_state.clone())));
    let video = Arc::new(RwLock::new(Video::new()));
    let duart = Arc::new(RwLock::new(Duart::new()));

    // Populate the global bus (this is done in a block so that
    // the bus lock can be dropped immediately)
    {
        let mut bus = bus::BUS.lock().unwrap();
        bus.set_acia(acia.clone());
        bus.set_video_ram(video_ram.clone());
        bus.set_video_controller(video.clone());
        bus.set_duart(duart.clone());
    }

    loop {
        tokio::join!(
            async {
                loop {
                    for _ in 0..opts.steps {
                        cpu.execute(CYCLES_PER_LOOP);
                    }
                    delay_for(Duration::from_millis(opts.idle)).await;
                }
            },
            AciaServer::run(
                acia_state.clone(),
                opts.address.as_str(),
                opts.port.as_str()
            ),
            async {
                let sdl_context = sdl2::init().expect("Could not initialize SDL2");
                let video_subsystem = sdl_context.video().expect("Could not get video subsystem");

                let window = video_subsystem
                    .window("Tektronix 4404", WINDOW_WIDTH, WINDOW_HEIGHT)
                    .build()
                    .unwrap();

                let mut fb: Vec<u8> = vec![0; (FB_WIDTH * FB_HEIGHT) as usize];
                let mut canvas = window.into_canvas().present_vsync().build().unwrap();
                let texture_creator = canvas.texture_creator();
                let mut texture = texture_creator
                    .create_texture_target(PixelFormatEnum::RGB332, FB_WIDTH, FB_HEIGHT)
                    .expect("Unable to create texture");

                let mut event_pump = sdl_context.event_pump().unwrap();

                loop {
                    for event in event_pump.poll_iter() {
                        match event {
                            Event::Quit { .. } => {
                                info!("Good bye.");
                                std::process::exit(0);
                            }
                            Event::KeyDown {
                                keycode: Some(k), ..
                            } => {
                                duart.write().unwrap().key_down(&k);
                            }
                            Event::KeyUp {
                                keycode: Some(k), ..
                            } => {
                                duart.write().unwrap().key_up(&k);
                            }
                            _ => {}
                        }
                    }

                    update_framebuffer(&video_ram, &mut fb);
                    texture
                        .update(None, &fb, FB_WIDTH as usize)
                        .expect("Couldn't copy framebuffer to texture");

                    canvas.clear();
                    canvas
                        .copy(
                            &texture,
                            // TODO: Texture source rectangle will
                            // actually be controlled by framebuffer
                            // panning register. It contains a 16-bit
                            // offset into the VRAM where drawing is
                            // to begin.
                            Rect::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT),
                            None,
                        )
                        .expect("Couldn't copy texture to canvas.");
                    canvas.present();

                    // Aim for 30 fps
                    delay_for(Duration::from_millis(33)).await;
                }
            }
        );
    }
}
