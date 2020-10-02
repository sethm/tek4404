use crate::bus::*;

use std::ops::RangeInclusive;

pub struct Timer {}

impl Timer {
    pub fn new() -> Self {
        Timer {}
    }
}

impl IoDevice for Timer {
    fn range(&self) -> RangeInclusive<usize> {
        TIMER_START..=TIMER_END
    }
}
