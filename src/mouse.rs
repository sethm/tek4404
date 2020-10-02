use crate::bus::*;

use std::ops::RangeInclusive;

pub struct Mouse {}

impl Mouse {
    pub fn new() -> Self {
        Mouse {}
    }
}

impl IoDevice for Mouse {
    fn range(&self) -> RangeInclusive<usize> {
        MOUSE_START..=MOUSE_END
    }
}
