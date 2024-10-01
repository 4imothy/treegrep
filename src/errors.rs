// SPDX-License-Identifier: MIT

use std::error::Error;
use std::fmt;

pub const SUBMIT_ISSUE: &str = "please submit an issue, github.com/4imothy/treegrep/issues";

pub struct Message {
    pub mes: String,
}

macro_rules! mes {
    ($($arg:tt)*) => {{
        Message {
            mes: format!($($arg)*),
        }
    }};
}

pub(crate) use mes;

impl Error for Message {}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.mes)
    }
}
