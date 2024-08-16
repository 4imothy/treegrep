// SPDX-License-Identifier: CC-BY-4.0

use crossterm::{
    cursor, execute,
    terminal::{self, ClearType},
};
use std::io::{self, StdoutLock, Write};

pub struct Term<'a> {
    out: StdoutLock<'a>,
    pub height: u16,
    pub width: u16,
}

impl<'a> Term<'a> {
    pub fn new(out: StdoutLock<'a>) -> io::Result<Term<'a>> {
        let (width, height) = terminal::size()?;
        Ok(Term { out, height, width })
    }

    pub fn set_dims(&mut self, dims: (u16, u16)) {
        (self.width, self.height) = dims;
    }

    pub fn clear(&mut self) -> io::Result<()> {
        execute!(self, terminal::Clear(ClearType::All))
    }

    pub fn claim(&mut self) -> io::Result<()> {
        execute!(
            self,
            cursor::Hide,
            terminal::EnterAlternateScreen,
            terminal::DisableLineWrap,
        )?;
        terminal::enable_raw_mode()
    }
}

impl<'a> Write for Term<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}
