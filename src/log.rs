use clap::Clap;
use strum_macros::{Display, EnumString};
use std::sync::Mutex;

#[derive(Clap, Eq, PartialEq, Ord, PartialOrd, Display, EnumString)]
pub enum LogLevel {
    #[strum(serialize="trace")]
    Trace,
    #[strum(serialize="debug")]
    Debug,
    #[strum(serialize="info")]
    Info,
    #[strum(serialize="warn")]
    Warn,
    #[strum(serialize="error")]
    Error,
    #[strum(serialize="none")]
    None,
}

pub struct Logger {
    pub log_level: LogLevel,
}

type LoggerRef = Mutex<Logger>;

lazy_static! {
    pub static ref LOGGER: LoggerRef = Mutex::new(Logger {log_level: LogLevel::None});
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
