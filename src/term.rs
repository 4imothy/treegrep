// SPDX-License-Identifier: MIT

use crossterm::{
    cursor, event, execute, queue, style,
    terminal::{self, ClearType},
};
use std::{
    io::{self, StdoutLock, Write},
    panic,
    sync::atomic::{AtomicU16, Ordering},
};

pub static TERM_WIDTH: AtomicU16 = AtomicU16::new(0);

pub struct Term<'a> {
    pub height: u16,
    out: StdoutLock<'a>,
    in_alternate_screen: bool,
    panic_hook_set: bool,
    with_mouse: bool,
    with_alternate_screen: bool,
}

impl<'a> Term<'a> {
    pub fn new(
        out: StdoutLock<'a>,
        need_ui: bool,
        with_mouse: bool,
        with_alternate_screen: bool,
    ) -> io::Result<Term<'a>> {
        let (width, height) = if need_ui { terminal::size()? } else { (0, 0) };
        TERM_WIDTH.store(width, Ordering::SeqCst);
        Ok(Term {
            out,
            height,
            in_alternate_screen: false,
            panic_hook_set: false,
            with_mouse,
            with_alternate_screen,
        })
    }

    pub fn set_dims(&mut self, new_height: u16, new_width: u16) {
        self.height = new_height;
        TERM_WIDTH.store(new_width, Ordering::SeqCst);
    }

    pub fn clear(&mut self) -> io::Result<()> {
        queue!(self, terminal::Clear(ClearType::All))
    }

    pub fn claim(&mut self) -> io::Result<()> {
        execute!(self, cursor::Hide, terminal::DisableLineWrap)?;
        if self.with_alternate_screen {
            execute!(self, terminal::EnterAlternateScreen)?;
        } else {
            execute!(self, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        }
        if self.with_mouse {
            execute!(self, event::EnableMouseCapture)?;
        }
        self.in_alternate_screen = true;
        if !self.panic_hook_set {
            let default_hook = panic::take_hook();
            let with_mouse = self.with_mouse;
            let with_alternate_screen = self.with_alternate_screen;
            panic::set_hook(Box::new(move |info| {
                Term::exit(&mut io::stderr(), with_mouse, with_alternate_screen).ok();
                default_hook(info);
            }));
            self.panic_hook_set = true;
        }

        terminal::enable_raw_mode()
    }

    fn exit<W>(w: &mut W, with_mouse: bool, with_alternate_screen: bool) -> io::Result<()>
    where
        W: Write,
    {
        terminal::disable_raw_mode()?;
        execute!(
            w,
            style::ResetColor,
            cursor::SetCursorStyle::DefaultUserShape,
            terminal::EnableLineWrap,
            cursor::Show,
        )?;
        if with_alternate_screen {
            execute!(w, terminal::LeaveAlternateScreen)?;
        }
        if with_mouse {
            execute!(w, event::DisableMouseCapture)?;
        }
        Ok(())
    }

    pub fn width(&self) -> u16 {
        TERM_WIDTH.load(Ordering::SeqCst)
    }

    pub fn give(&mut self) -> io::Result<()> {
        self.flush()?;
        Term::exit(self, self.with_mouse, self.with_alternate_screen)?;
        self.in_alternate_screen = false;
        if self.panic_hook_set {
            let _ = panic::take_hook();
            self.panic_hook_set = false;
        }
        Ok(())
    }

    pub fn suspend(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            self.give()?;
            signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP).unwrap();
        }
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
