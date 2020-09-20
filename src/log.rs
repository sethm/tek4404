/// Copyright 2020 Seth Morabito <web@loomcom.com>
///
/// Permission is hereby granted, free of charge, to any person
/// obtaining a copy of this software and associated documentation
/// files (the "Software"), to deal in the Software without
/// restriction, including without limitation the rights to use, copy,
/// modify, merge, publish, distribute, sublicense, and/or sell copies
/// of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be
/// included in all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
/// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
/// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
/// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
/// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
/// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
/// DEALINGS IN THE SOFTWARE.
use clap::Clap;
use std::sync::Mutex;
use strum_macros::{Display, EnumString};

#[derive(Clap, Eq, PartialEq, Ord, PartialOrd, Display, EnumString)]
pub enum LogLevel {
    #[strum(serialize = "trace")]
    Trace,
    #[strum(serialize = "debug")]
    Debug,
    #[strum(serialize = "info")]
    Info,
    #[strum(serialize = "warn")]
    Warn,
    #[strum(serialize = "error")]
    Error,
    #[strum(serialize = "none")]
    None,
}

pub struct Logger {
    pub log_level: LogLevel,
}

type LoggerRef = Mutex<Logger>;

lazy_static! {
    pub static ref LOGGER: LoggerRef = Mutex::new(Logger {
        log_level: LogLevel::None
    });
}

pub fn init(log_level: LogLevel) {
    LOGGER.lock().unwrap().log_level = log_level;
}

macro_rules! log_common {
    ($level:expr, $($msg:expr),+) => {{
        {
            let cur_level = &LOGGER.lock().unwrap().log_level;
            if cur_level <= &$level {
                let full_mod: &str = module_path!();
                let display: Vec<&str> = full_mod.split("::").collect();
                let name = $level.to_string().to_uppercase();
                match display.last() {
                    Some(e) => print!("[{:^5}][{:>7}] ", name, e),
                    None => print!("[{:^5}][    ???] ", name),
                }
                println!($($msg),+);
            }
        }
    }}
}

#[macro_export]
macro_rules! trace {
    ($($msg:expr),+) => {{
        use crate::log::*;

        log_common!(LogLevel::Trace, $($msg),+);
    }};
}

#[macro_export]
macro_rules! debug {
    ($($msg:expr),+) => {{
        use crate::log::*;

        log_common!(LogLevel::Debug, $($msg),+);
    }};
}

#[macro_export]
macro_rules! info {
    ($($msg:expr),+) => {{
        use crate::log::*;

        log_common!(LogLevel::Info, $($msg),+);
    }};
}
