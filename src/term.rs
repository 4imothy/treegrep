// SPDX-License-Identifier: MIT

use crossterm::{
    cursor, event, execute, style,
    terminal::{self, ClearType},
};
use std::io::{self, StdoutLock, Write};
use std::panic;
use std::sync::atomic::{AtomicU16, Ordering};

pub static TERM_WIDTH: AtomicU16 = AtomicU16::new(0);

pub struct Term<'a> {
    pub height: u16,
    out: StdoutLock<'a>,
    in_alternate_screen: bool,
}

impl<'a> Term<'a> {
    pub fn new(out: StdoutLock<'a>) -> io::Result<Term<'a>> {
        let (width, height) = terminal::size()?;
        TERM_WIDTH.store(width, Ordering::SeqCst);
        Ok(Term {
            out,
            height,
            in_alternate_screen: false,
        })
    }

    pub fn set_dims(&mut self, new_height: u16, new_width: u16) {
        self.height = new_height;
        TERM_WIDTH.store(new_width, Ordering::SeqCst);
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
            event::EnableMouseCapture
        )?;
        self.in_alternate_screen = true;
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            Term::exit(&mut io::stderr()).ok();
            default_hook(info);
        }));

        terminal::enable_raw_mode()
    }

    fn exit<W>(w: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        terminal::disable_raw_mode()?;
        execute!(
            w,
            style::ResetColor,
            cursor::SetCursorStyle::DefaultUserShape,
            terminal::LeaveAlternateScreen,
            terminal::EnableLineWrap,
            event::DisableMouseCapture,
            cursor::Show,
        )
    }

    pub fn width(&self) -> u16 {
        TERM_WIDTH.load(Ordering::SeqCst)
    }

    pub fn give(&mut self) -> io::Result<()> {
        self.flush()?;
        Term::exit(self)?;
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
