mod err;
mod bus;
mod cpu;
mod mem;

#[macro_use]
extern crate lazy_static;

const CYCLES: u32 = 180;

fn main() {
    println!("[   MAIN] Tektronix 4404 Starting...");
    println!("[   MAIN] Resetting CPU...");
    bus::reset();
    cpu::init();
    cpu::reset();
    println!("[   MAIN] EXECUTING...");
    let cyc = cpu::execute(CYCLES);
    println!("[   MAIN] {} CYCLES COMPLETED.", cyc);
}
