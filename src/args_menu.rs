// SPDX-License-Identifier: MIT

use crate::args::{self, MENU};
use crate::errors::{Message, mes};
use crate::{
    config::{self, Config},
    formats, term,
};
use clap::error::ErrorKind;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::Print,
    terminal,
};
use std::ffi::OsString;
use std::io;

const CLAP_ERROR_PREFIX: &str = "error:";

pub struct ArgsMenu<'a, 'b> {
    term: &'a mut term::Term<'b>,
    conf: &'a Config,
    input: String,
    start: usize,
    cursor_index: usize,
    center_y: u16,
    center_x: u16,
    text_box_width: u16,
    start_x: u16,
}

enum Action {
    Append,
    Remove,
    MoveCursor,
    ClearForward(usize),
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
                if let ErrorKind::DisplayHelp = e.kind() {
                    "can't display help in menu".to_string()
                } else if let ErrorKind::DisplayVersion = e.kind() {
                    "can't display version in menu".to_string()
                } else {
                    s.strip_prefix(CLAP_ERROR_PREFIX).unwrap_or(&s).to_string()
                }
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
            cursor_index: 0,
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
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            if c == 'c' {
                                quit = true;
                                break;
                            } else if c == 'z' {
                                menu.suspend()?;
                                menu.resume()?;
                            } else if c == 'f' {
                                menu.cursor_forward()?;
                            } else if c == 'b' {
                                menu.cursor_backward()?;
                            } else if c == 'd' {
                                if !menu.input.is_empty() && menu.cursor_index < menu.input.len() {
                                    menu.input.remove(menu.cursor_index);
                                    menu.cursor_index += 1;
                                    menu.update_window(Action::Remove)?;
                                }
                            } else if c == 'a' {
                                menu.cursor_index = 0;
                                menu.start = 0;
                                menu.update_window(Action::MoveCursor)?;
                            } else if c == 'e' {
                                menu.cursor_index = menu.input.len();
                                menu.start = menu
                                    .input
                                    .len()
                                    .saturating_sub(menu.text_box_width as usize - 1);
                                menu.update_window(Action::MoveCursor)?;
                            } else if c == 'k' {
                                let removed = menu.input.len() - menu.cursor_index;
                                menu.input.truncate(menu.cursor_index);
                                menu.update_window(Action::ClearForward(removed))?;
                            }
                        } else {
                            menu.input.insert(menu.cursor_index, c);
                            menu.update_window(Action::Append)?;
                        }
                    }
                    KeyCode::Left => {
                        menu.cursor_backward()?;
                    }
                    KeyCode::Right => {
                        menu.cursor_forward()?;
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        if !menu.input.is_empty() && menu.cursor_index > 0 {
                            menu.input.remove(menu.cursor_index - 1);
                            menu.update_window(Action::Remove)?;
                        }
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

    fn cursor_backward(&mut self) -> io::Result<()> {
        if self.cursor_index > 0 {
            self.cursor_index -= 1;
            if self.cursor_index + 1 == self.start && self.start > 0 {
                self.start -= 1;
            }
            self.update_window(Action::MoveCursor)?;
        }
        Ok(())
    }

    fn cursor_forward(&mut self) -> io::Result<()> {
        if self.cursor_index < self.input.len() {
            self.cursor_index += 1;
            if self.cursor_index - self.start == self.text_box_width as usize
                && self.input.len() >= self.text_box_width as usize
            {
                self.start += 1;
            }
            self.update_window(Action::MoveCursor)?;
        }
        Ok(())
    }

    fn update_window(&mut self, action: Action) -> io::Result<()> {
        let mut clear = "".to_string();
        match action {
            Action::Append => {
                self.cursor_index += 1;
                if self.input.len() >= self.text_box_width as usize {
                    self.start += 1;
                }
            }
            Action::Remove => {
                self.start = self.start.saturating_sub(1);
                self.cursor_index -= 1;
                clear = " ".to_string();
            }
            Action::MoveCursor => {}
            Action::ClearForward(c) => {
                clear = " ".repeat(c.min(self.text_box_width as usize));
            }
        }
        let cursor_pos_x = self.cursor_pos_x();

        execute!(
            self.term,
            cursor::MoveTo(self.start_x + 1, self.center_y),
            Print(
                &self.input[self.start
                    ..(self.start + self.text_box_width as usize - 1).min(self.input.len())]
            ),
            Print(clear),
            cursor::MoveTo(cursor_pos_x, self.center_y),
        )
    }

    fn resize(&mut self, new_height: u16, new_width: u16) -> io::Result<()> {
        self.term.set_dims(new_height, new_width);
        self.set_locations();
        self.start = self
            .input
            .len()
            .saturating_sub(self.text_box_width as usize);
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
        self.start_x + (self.cursor_index - self.start) as u16 + 1
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
