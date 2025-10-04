// SPDX-License-Identifier: MIT

use crate::term;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    queue,
    style::Print,
};
use std::error::Error;
use std::fmt;
use std::io;
use std::io::Write;

pub const SUBMIT_ISSUE: &str = "please submit an issue, github.com/4imothy/treegrep/issues";

pub struct Message {
    pub mes: String,
}

macro_rules! mes {
    ($($arg:tt)*) => {{
        Message {
            mes: {
                if cfg!(debug_assertions) {
                format!("{}:{}: {}", file!(), line!(), format!($($arg)*))
                } else {
                format!($($arg)*)
                }
            },
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

pub fn view_error(term: &mut term::Term, mes: String) -> io::Result<()> {
    term.clear()?;
    for (i, line) in mes.lines().enumerate() {
        queue!(term, cursor::MoveTo(0, i as u16), Print(line))?;
    }
    term.flush()?;
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
            && (c == 'q' || (c == 'c' && modifiers.contains(KeyModifiers::CONTROL)))
        {
            break;
        };
    }
    term.give()
}
