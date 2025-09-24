// SPDX-License-Identifier: MIT

use crate::args::{self, MENU};
use crate::errors::{Message, mes};
use crate::{
    config::{self, Config},
    formats, term,
};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::Print,
    terminal,
};
use std::ffi::OsString;
use std::io;

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
    let input = ArgsMenu::enter(term, &conf).map_err(|e| mes!("{}", e.to_string()))?;
    if let Some(i) = input {
        let mut input_vec: Vec<OsString> = shlex::split(&i)
            .ok_or_else(|| mes!("erroneous arguments"))?
            .into_iter()
            .map(OsString::from)
            .collect();
        input_vec.push(OsString::from(format!("--{}", args::SELECT.id)));
        let res = config::get_matches(input_vec, true);
        match res {
            Ok((matches, all_args)) => {
                let (bold, colors) = Config::get_styling(&matches);
                let mut c = Config::get_config(matches, all_args, bold, colors)?;
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
            Err(e) => Err(mes!("{}", {
                let s = e.to_string();
                s.strip_prefix("error:").unwrap_or(&s).to_string()
            })),
        }
    } else {
        Ok(None)
    }
}

impl<'a, 'b> ArgsMenu<'a, 'b> {
    pub fn enter(term: &mut term::Term, conf: &Config) -> io::Result<Option<String>> {
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
        execute!(menu.term, cursor::Show)?;
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
        execute!(menu.term, cursor::Hide)?;
        menu.term.clear()?;
        if quit { Ok(None) } else { Ok(Some(menu.input)) }
    }

    fn max_viewable_len(&self) -> u16 {
        self.text_box_width.saturating_sub(1)
    }

    fn update_window(&mut self, append: bool) -> io::Result<()> {
        if !append {
            self.start = self.start.saturating_sub(1);
        } else if self.input.len() > self.text_box_width as usize - 1 {
            self.start += 1;
        }
        let cursor_pos_x = self.cursor_pos_x();

        execute!(
            self.term,
            cursor::MoveTo(self.start_x + 1, self.center_y),
            Print(&self.input[self.start..]),
            Print(if append { "" } else { " " }),
            cursor::MoveTo(cursor_pos_x, self.center_y),
        )
    }

    fn resize(&mut self, new_height: u16, new_width: u16) -> io::Result<()> {
        self.term.set_dims(new_height, new_width);
        self.set_locations();
        self.start = self
            .input
            .len()
            .saturating_sub(self.max_viewable_len() as usize);
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
            self.term.claim()?;
            self.resize(height, width)?;
        }
        Ok(())
    }

    fn cursor_pos_x(&self) -> u16 {
        self.start_x + self.input.len().min(self.max_viewable_len() as usize) as u16 + 1
    }

    fn draw(&mut self) -> io::Result<()> {
        self.term.clear()?;
        let cursor_pos_x = self.cursor_pos_x();

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
            cursor::MoveTo(cursor_pos_x, self.center_y),
        )
    }
}
