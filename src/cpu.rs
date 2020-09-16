///
/// Tektronix 4404 CPU Implementation
///
use std::os::raw::{c_int, c_uint};

const M68K_CPU_TYPE_68010: c_uint = 2;

extern "C" {
    pub fn m68k_set_cpu_type(cpu_type: c_uint);
    pub fn m68k_init();
    pub fn m68k_pulse_reset();
    pub fn m68k_execute(num_cycles: c_int) -> c_int;
}

pub fn init() {
    unsafe {
        m68k_init();
        m68k_set_cpu_type(M68K_CPU_TYPE_68010);
        m68k_pulse_reset();
    }
}

pub fn execute(num_cycles: u32) -> u32 {
    unsafe {
        let s = m68k_execute(num_cycles as c_int);

        s as u32
    }
}
