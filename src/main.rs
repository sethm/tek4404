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
