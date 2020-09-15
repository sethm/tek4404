mod cpu;

const CYCLES: u32 = 80;

fn main() {
    println!("[  MAIN] Resetting CPU...");
    cpu::init();
    println!("[  MAIN] EXECUTING...");
    let cycles_done = cpu::execute(CYCLES);
    println!("[  MAIN] Execution consumed {} cycles.", cycles_done);
}
