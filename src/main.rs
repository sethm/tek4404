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
mod err;
mod bus;
mod cpu;
mod mem;

#[macro_use]
extern crate lazy_static;
extern crate strum;
extern crate strum_macros;

use crate::log::*;
use clap::Clap;

#[derive(Clap)]
struct Opts {
    #[clap(short, long, default_value="100")]
    steps: u32,
    #[clap(short, long, default_value="none")]
    loglvl: LogLevel,
}

fn main() {
    let opts: Opts = Opts::parse();

    log::init(opts.loglvl);

    info!("RESET");
    bus::reset();
    cpu::init();
    cpu::reset();
    info!("BOOT");
    let cyc = cpu::execute(opts.steps);
    info!("{} CYCLES COMPLETED.", cyc);
}
