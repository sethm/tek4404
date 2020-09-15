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

#[no_mangle]
pub fn m68k_read_memory_8(address: c_uint) -> c_uint {
    println!("[READ08] addr=0x{:08x}", address);
    return 0;
}

#[no_mangle]
pub fn m68k_read_memory_16(address: c_uint) -> c_uint {
    println!("[READ16] addr=0x{:08x}", address);
    return 0x4e71;
}

#[no_mangle]
pub fn m68k_read_memory_32(address: c_uint) -> c_uint {
    println!("[READ32] addr=0x{:08x}", address);
    if address == 0 {
        println!("[READ32]    ... STACK = 0x10000");
        return 0x10000;
    } else if address == 4 {
        println!("[READ32]    ...    PC = 0x30000");
        return 0x30000;
    } else {
        return 0x4e71;
    }
}

#[no_mangle]
pub fn m68k_write_memory_8(address: c_uint, value: c_uint) {
    println!("[WRITE08] addr=0x{:08x} val=0x{:02x}", address, value);
}

#[no_mangle]
pub fn m68k_write_memory_16(address: c_uint, value: c_uint) {
    println!("[WRITE16] addr=0x{:08x} val=0x{:04x}", address, value);
}

#[no_mangle]
pub fn m68k_write_memory_32(address: c_uint, value: c_uint) {
    println!("[WRITE32] addr=0x{:08x} val=0x{:08x}", address, value);
}
