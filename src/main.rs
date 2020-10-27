//! Tektronix 4404 Emulator.
//!
//! # About
//!
//! The Tektronix 4404 was a mid 1980s AI workstation that ran
//! Smalltalk-80 and LISP natively. Built around a 68010 CPU, it
//! supported 1-2 MB of RAM, a 45MB SCSI hard disk, serial and
//! parallel IO, and had a 640x480 bitmapped display backed by a
//! 1024x1024 pixel 2-bit framebuffer.
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
#[macro_use]
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
mod service;
mod sound;
mod timer;
mod video;

#[macro_use]
extern crate lazy_static;
extern crate num_derive;
extern crate strum;
extern crate strum_macros;

use acia::{Acia, AciaServer, AciaState};
use bus::*;
use cpu::Cpu;
use duart::Duart;
use log::*;
use mem::Memory;
use scsi::Scsi;
use service::ServiceKey;
use video::Video;

use clap::Clap;
use tokio::time;

use std::error::Error;
use std::sync::{Arc, Mutex};

use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

/// Framebuffer width
const FB_WIDTH: u32 = 1024;
/// Framebuffer height
const FB_HEIGHT: u32 = 1024;
/// Visible window width
const WINDOW_WIDTH: u32 = 640;
/// Visible window height
const WINDOW_HEIGHT: u32 = 480;
/// The number of milliseconds to idle between framebuffer repaints
const DISPLAY_IDLE: u64 = 10;

/// Clap options parsed from the command line
#[derive(Clap)]
#[clap(about = "Tektronix 4404 Emulator")]
struct Opts {
    /// The path to the 32KB boot ROM image
    #[clap(short, long, default_value = "rom/boot.bin")]
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
        default_value = "30000",
        about = "CPU execution steps per loop"
    )]
    steps: u32,
    /// The number of machine cycles to execute each step
    #[clap(
        short,
        long,
        default_value = "16",
        about = "CPU cycles per execution step"
    )]
    cycles: u32,
    /// The amount of time to idle between loops
    #[clap(
        short,
        long,
        default_value = "25",
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
    let mem = &vm.lock().unwrap().mem;

    for b in mem {
        for i in 0..=7 {
            if (b >> (7 - i)) & 1 == 1 {
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

    // Load the ROM boot file.
    let rom = Arc::new(Mutex::new(
        Memory::new(ROM_START, ROM_END, ROM_SIZE, true).unwrap(),
    ));
    let data = std::fs::read(opts.bootrom.as_str())?;
    rom.lock().unwrap().load(&data);

    // Create RAM and other devices, and populate the bus.
    let ram = Arc::new(Mutex::new(
        Memory::new(RAM_START, RAM_END, RAM_SIZE, false).unwrap(),
    ));
    let video_ram = Arc::new(Mutex::new(
        Memory::new(VRAM_START, VRAM_END, VRAM_SIZE, false).unwrap(),
    ));
    let acia_state = Arc::new(Mutex::new(AciaState::new()));
    let acia = Arc::new(Mutex::new(Acia::new(acia_state.clone())));
    let video = Arc::new(Mutex::new(Video::new()));
    let duart = Arc::new(Mutex::new(Duart::new()));
    let scsi = Arc::new(Mutex::new(Scsi::new()));

    // Populate the global bus (this is done in a block so that
    // the bus lock can be dropped immediately)
    {
        let mut bus = BUS.lock().unwrap();

        // The bus can own these devices
        bus.rom = Some(rom);
        bus.ram = Some(ram);
        bus.video = Some(video);

        // The bus must share these devices
        bus.acia = Some(acia.clone());
        bus.video_ram = Some(video_ram.clone());
        bus.duart = Some(duart.clone());
        bus.scsi = Some(scsi.clone());
    }

    let mut cpu = Cpu::new();

    loop {
        tokio::join!(
            async {
                let sleep_time = time::Duration::from_millis(opts.idle);
                loop {
                    for _ in 0..opts.steps {
                        cpu.execute(&opts.cycles);
                    }

                    loop {
                        // Hold the Queue lock for as brief a time as possible
                        // by assigning the result of `take()` to a variable.
                        let next_task = QUEUE.lock().unwrap().take();

                        if let Some(srq) = next_task {
                            match srq.key {
                                ServiceKey::Scsi => scsi.lock().unwrap().service(),
                            }
                        } else {
                            break;
                        }
                    }

                    time::sleep(sleep_time).await;
                }
            },
            AciaServer::run(
                acia_state.clone(),
                opts.address.as_str(),
                opts.port.as_str()
            ),
            async {
                let sleep_time = time::Duration::from_millis(DISPLAY_IDLE);
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
                                duart.lock().unwrap().key_down(&k);
                            }
                            Event::KeyUp {
                                keycode: Some(k), ..
                            } => {
                                duart.lock().unwrap().key_up(&k);
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

                    time::sleep(sleep_time).await;
                }
            }
        );
    }
}
