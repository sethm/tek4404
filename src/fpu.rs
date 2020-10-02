use crate::bus::*;

use std::ops::RangeInclusive;

pub struct Fpu {}

impl Fpu {
    pub fn new() -> Fpu {
        Fpu {}
    }
}

impl IoDevice for Fpu {
    fn range(&self) -> RangeInclusive<usize> {
        FPU_START..=FPU_END
    }
}
