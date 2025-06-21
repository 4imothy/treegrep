// SPDX-License-Identifier: MIT

use crate::args::{self, MENU};
use crate::errors::{mes, Message};
use crate::{
    config::{self, Config},
    formats, term,
};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::Print,
    terminal,
};
use std::ffi::OsString;
use std::io::{self, Write};

pub struct ArgsMenu<'a, 'b> {
    term: &'a mut term::Term<'b>,
    conf: &'a Config,
    input: String,
    start: usize,
    center_y: u16,
    center_x: u16,
    text_box_width: u16,
    start_x: u16,
}

pub fn launch(term: &mut term::Term, conf: Config) -> Result<Option<Config>, Message> {
    let input = ArgsMenu::enter(term, &conf).map_err(|e| {
        let _ = term.give();
        mes!("{}", e.to_string())
    })?;
    if let Some(i) = input {
        let mut input_vec: Vec<OsString> = shlex::split(&i)
            .unwrap()
            .into_iter()
            .map(OsString::from)
            .collect();
        input_vec.push(OsString::from(format!("--{}", args::SELECT.id)));
        let res = config::get_matches(input_vec, true);
        match res {
            Ok((matches, all_args)) => {
                let (bold, colors) = Config::get_styling(&matches);
                let mut c = match Config::get_config(matches, all_args, bold, colors) {
                    Ok(c) => c,
                    Err(e) => {
                        view_error(
                            term,
                            format!("{} {}", formats::error_prefix(bold, colors), e),
                        )
                        .map_err(|e| mes!("{}", e.to_string()))?;
                        return Ok(None);
                    }
                };
                c.repeat_file = conf.repeat_file;
                if let Some(new_c) = c.handle_repeat()? {
                    c = new_c;
                }
                c.menu = true;
                c.selection_file = conf.selection_file;
                if c.completion_target.is_some() {
                    Err(mes!("can't generate completions in {}", MENU.id))
                } else {
                    Ok(Some(c))
                }
            }
            Err(e) => {
                view_error(term, e.to_string()).map_err(|e| mes!("{}", e.to_string()))?;
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

impl<'a, 'b> ArgsMenu<'a, 'b> {
    pub fn enter(term: &mut term::Term, conf: &Config) -> io::Result<Option<String>> {
        term.claim()?;

        let quit;
        let mut menu = ArgsMenu {
            term,
            conf,
            input: String::new(),
            start: 0,
            center_y: 0,
            center_x: 0,
            text_box_width: 0,
            start_x: 0,
        };
        menu.set_locations();
        menu.draw()?;
        loop {
            let event = event::read()?;
            match event {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind: KeyEventKind::Press,
                    ..
                }) => match code {
                    KeyCode::Enter => {
                        quit = menu.input.is_empty();
                        break;
                    }
                    KeyCode::Char(c) => {
                        if c == 'c' && modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            quit = true;
                            break;
                        } else if c == 'z'
                            && modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                        {
                            menu.suspend()?;
                            menu.resume()?;
                        } else {
                            menu.input.push(c);
                            menu.update_window(true)?;
                        }
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        menu.input.pop();
                        menu.update_window(false)?;
                    }
                    _ => {}
                },
                Event::Resize(new_width, new_height) => {
                    if menu.term.height != new_height || menu.term.width() != new_width {
                        menu.resize(new_height, new_width)?;
                    }
                }
                _ => {}
            }
        }
        menu.term.clear()?;
        if quit {
            Ok(None)
        } else {
            Ok(Some(menu.input))
        }
    }

    fn update_window(&mut self, append: bool) -> io::Result<()> {
        if !append {
            self.start = self.start.saturating_sub(1);
        } else if self.input.len() > self.text_box_width as usize - 1 {
            self.start += 1;
        }

        execute!(
            self.term,
            cursor::MoveTo(self.start_x + 1, self.center_y),
            Print(&self.input[self.start..]),
            Print(if append { "" } else { " " })
        )
    }

    fn resize(&mut self, new_height: u16, new_width: u16) -> io::Result<()> {
        self.term.set_dims(new_height, new_width);
        self.set_locations();
        self.start = self
            .input
            .len()
            .saturating_sub(self.text_box_width as usize - 1);
        self.draw()
    }

    fn set_locations(&mut self) {
        self.center_y = self.term.height / 2;
        self.center_x = self.term.width() / 2;
        self.text_box_width = self.term.width() / 2;
        self.start_x = self.center_x - self.text_box_width / 2;
    }

    fn suspend(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            self.term.give()?;
            signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP).unwrap();
        }
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            let (width, height) = terminal::size()?;
            self.term.set_dims(height, width);
            self.term.claim()?;
            self.draw()?;
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        self.term.clear()?;

        execute!(
            self.term,
            cursor::MoveTo(self.start_x, self.center_y - 1),
            Print(format!(
                "{}{}{}",
                self.conf.c.tl,
                formats::repeat(self.conf.c.h, self.text_box_width as usize),
                self.conf.c.tr
            )),
            cursor::MoveTo(self.start_x, self.center_y),
            Print(format!(
                "{}{:w$}{}",
                self.conf.c.v,
                &self.input[self.start..],
                self.conf.c.v,
                w = self.text_box_width as usize
            )),
            cursor::MoveTo(self.start_x, self.center_y + 1),
            Print(format!(
                "{}{}{}",
                self.conf.c.bl,
                formats::repeat(self.conf.c.h, self.text_box_width as usize),
                self.conf.c.br,
            )),
            cursor::MoveTo(self.start_x + self.input.len() as u16, self.center_y),
        )
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
        {
            if c == 'q' || (c == 'c' && modifiers.contains(KeyModifiers::CONTROL)) {
                break;
            }
        };
    }
    Ok(())
}
