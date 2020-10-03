//! Main CPU functions
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
use crate::bus;
use crate::err::SimError;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_uint};

const M68K_CPU_TYPE_68010: c_uint = 2;

type InstructionHook = extern "C" fn(pc: c_uint);

extern "C" {
    pub fn m68k_set_cpu_type(cpu_type: c_uint);
    pub fn m68k_init();
    pub fn m68k_pulse_reset();
    pub fn m68k_pulse_bus_error();
    pub fn m68k_execute(num_cycles: c_int) -> c_int;
    pub fn m68k_disassemble(buf: *mut c_char, pc: c_uint, cpu_type: c_uint) -> c_uint;
    pub fn m68k_set_instr_hook_callback(hook: InstructionHook);
}

pub struct Cpu {}

// TODO: Interrupts.
//
// Levels:
//    1: TIMER
//    2: DMA
//    3: SCSI
//    4: SPARE
//    5: UART
//    6: VSYNC
//    7: DEBUG
//

impl Cpu {
    pub fn new(rom_file: &str) -> Self {
        match bus::load_rom(rom_file) {
            Ok(()) => {
                info!("Initializing CPU.");
                init();
                info!("Resetting CPU.");
                reset();
            }
            Err(SimError::Init(msg)) => {
                panic!(msg);
            }
        }

        Cpu {}
    }

    pub fn execute(&mut self, cycles: i32) {
        let _ = unsafe { m68k_execute(cycles as c_int) };
    }
}

pub fn bus_error() {
    unsafe {
        info!("Bus Error: BEFORE");
        m68k_pulse_bus_error();
        info!("Bus Error: AFTER PULSE");
    }
}

fn init() {
    unsafe {
        m68k_init();
        m68k_set_cpu_type(M68K_CPU_TYPE_68010);
        m68k_set_instr_hook_callback(instruction_hook);
    }
}

fn reset() {
    unsafe {
        m68k_pulse_reset();
    }
}

#[no_mangle]
extern "C" fn instruction_hook(pc: c_uint) {
    if crate::log::is_debug() {
        let mut c_arr: [c_char; 256] = [0; 256];
        let c_ptr = c_arr.as_mut_ptr();

        unsafe {
            m68k_disassemble(c_ptr, pc, M68K_CPU_TYPE_68010);
            trace!("{:08x}:    {}", pc, CStr::from_ptr(c_ptr).to_str().unwrap());
        }
    }
}
