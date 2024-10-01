// SPDX-License-Identifier: MIT

use crossterm::{
    cursor, execute, style,
    terminal::{self, ClearType},
};
use std::io::{self, StdoutLock, Write};
use std::panic;

pub struct Term<'a> {
    pub height: u16,
    pub width: u16,
    out: StdoutLock<'a>,
    in_alternate_screen: bool,
}

impl<'a> Term<'a> {
    pub fn new(out: StdoutLock<'a>) -> io::Result<Term<'a>> {
        let (width, height) = terminal::size()?;
        Ok(Term {
            out,
            height,
            width,
            in_alternate_screen: false,
        })
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
        self.in_alternate_screen = true;
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            Term::exit().ok();
            default_hook(info);
        }));

        terminal::enable_raw_mode()
    }

    fn exit() -> io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            io::stderr(),
            style::ResetColor,
            cursor::SetCursorStyle::DefaultUserShape,
            terminal::LeaveAlternateScreen,
            terminal::EnableLineWrap,
            cursor::Show,
        )
    }

    pub fn give(&mut self) -> io::Result<()> {
        self.flush()?;
        Term::exit()?;
        self.in_alternate_screen = false;
        let _ = panic::take_hook();
        Ok(())
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
