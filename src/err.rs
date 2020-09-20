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

use std::error::Error;
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum BusError {
    Access,
    Alignment,
    ReadOnly,
}

impl fmt::Debug for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BusError::Access => write!(f, "Access Error"),
            BusError::Alignment => write!(f, "Alignment Error"),
            BusError::ReadOnly => write!(f, "Read Only Error"),
        }
    }
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BusError::Access => write!(f, "Access Error"),
            BusError::Alignment => write!(f, "Alignment Error"),
            BusError::ReadOnly => write!(f, "Read Only Error"),
        }
    }
}

impl Error for BusError {
    fn description(&self) -> &str {
        match *self {
            BusError::Access => "Access Error",
            BusError::Alignment => "Alignment Error",
            BusError::ReadOnly => "Read Only Error",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

#[derive(PartialEq, Eq)]
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
