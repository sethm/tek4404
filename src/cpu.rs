///
/// Tektronix 4404 CPU Implementation
///
use std::os::raw::{c_int, c_uint, c_char};
use std::ffi::CString;

const M68K_CPU_TYPE_68010: c_uint = 2;

type InstructionHook = extern "C" fn(pc: c_uint);

extern "C" {
    pub fn m68k_set_cpu_type(cpu_type: c_uint);
    pub fn m68k_init();
    pub fn m68k_pulse_reset();
    pub fn m68k_pulse_bus_error();
    pub fn m68k_execute(num_cycles: c_int) -> c_int;
    pub fn m68k_disassemble(buf: *mut c_char, pc: c_uint, cpu_type: c_uint);
    pub fn m68k_set_instr_hook_callback(hook: InstructionHook);
}

pub fn bus_error() {
    unsafe {
        m68k_pulse_bus_error();
    }
}

pub fn init() {
    unsafe {
        m68k_init();
        m68k_set_cpu_type(M68K_CPU_TYPE_68010);
        m68k_set_instr_hook_callback(instruction_hook);
    }
}

pub fn reset() {
    unsafe {
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
extern "C" fn instruction_hook(pc: c_uint) {
    unsafe {
        let buf = Vec::<u8>::with_capacity(256);
        let asm = CString::new(buf).unwrap();
        let asm_buf = asm.into_raw();
        m68k_disassemble(asm_buf, pc, M68K_CPU_TYPE_68010);
        let s = CString::from_raw(asm_buf).into_string().unwrap();

        debug!("{:08x}: \t\t{}", pc, s);
    }
}
