///
/// Tektronix 4404 Errors
///
use std::error::Error;
use std::fmt;

pub enum BusError {
    Access,
    Alignment,
}

impl fmt::Debug for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BusError::Access => write!(f, "Access Error"),
            BusError::Alignment => write!(f, "Alignment Error"),
        }
    }
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BusError::Access => write!(f, "Access Error"),
            BusError::Alignment => write!(f, "Alignment Error"),
        }
    }
}

impl Error for BusError {
    fn description(&self) -> &str {
        match *self {
            BusError::Access => "Access Error",
            BusError::Alignment => "Alignment Error",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

pub enum SimError {
    Init(String),
}

impl fmt::Debug for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            SimError::Init(s) => write!(f, "Initialization Error: {}", s)
        }
    }
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            SimError::Init(s) => write!(f, "Initialization Error: {}", s)
        }
    }
}

impl Error for SimError {
    fn description(&self) -> &str {
        match *self {
            SimError::Init(_) => "Initialization Error",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}
