// SPDX-License-Identifier: MIT

use crate::args::MENU_HELP;
use crate::{config, formats, term, writer::Entry};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    queue,
    style::{Print, SetBackgroundColor},
    terminal,
};
use std::ffi::OsString;
use std::io::{self, StdoutLock, Write};
use std::process::Command;

const START_X: u16 = formats::SELECTED_INDICATOR_CLEAR.len() as u16;
const START_Y: u16 = 0;

struct PathInfo {
    paths: Vec<usize>,
    prev: usize,
    next: usize,
    passed: bool,
}

impl PathInfo {
    pub fn new(paths: Vec<usize>) -> PathInfo {
        PathInfo {
            paths,
            prev: 0,
            next: 1,
            passed: false,
        }
    }

    pub fn top(&mut self) {
        self.prev = 0;
        self.next = 1;
        self.passed = false;
    }

    pub fn bottom(&mut self) {
        let last_match_is_path = config().files as usize;
        self.prev = self.paths.len() - 1 - last_match_is_path;
        self.next = self.paths.len() - last_match_is_path;
        self.passed = false;
    }

    pub fn down(&mut self, selected_id: usize) {
        if self.passed {
            self.prev += 1;
            self.passed = false;
        }
        if self.next != self.paths.len() && selected_id == *self.paths.get(self.next).unwrap() {
            self.next += 1;
            self.passed = true;
        }
    }

    pub fn up(&mut self, selected_id: usize) {
        if self.passed {
            self.next -= 1;
            self.passed = false;
        }
        if self.prev != 0 && selected_id == *self.paths.get(self.prev).unwrap() {
            self.prev -= 1;
            self.passed = true;
        }
    }

    pub fn dist_down(&self, selected_id: usize) -> u16 {
        if self.next == self.paths.len() {
            return 0;
        }
        (*self.paths.get(self.next).unwrap() - selected_id) as u16
    }

    pub fn dist_up(&self, selected_id: usize) -> u16 {
        (selected_id - *self.paths.get(self.prev).unwrap()) as u16
    }
}

pub struct Menu<'a> {
    pi: PathInfo,
    selected_id: usize,
    cursor_y: u16,
    term: term::Term<'a>,
    max_line_id: usize,
    lines: &'a Vec<Box<dyn Entry + 'a>>,
    colors: bool,
    scroll_offset: u16,
    big_jump: u16,
    small_jump: u16,
    popup_open: bool,
    first_y: u16,
    last_y: u16,
    window: Window,
}

struct Window {
    first_id: usize,
    last_id: usize,
    max_id: usize,
}

impl Window {
    pub fn set_first(&mut self, first: usize) {
        self.first_id = first;
    }
    pub fn set_last(&mut self, last: usize) {
        self.last_id = last;
    }
    pub fn up_shift_last(&mut self) {
        self.last_id -= 1;
    }
    pub fn down_shift_first(&mut self) {
        self.first_id += 1;
    }

    pub fn set(&mut self, first_id: usize, last_id: usize) {
        (self.first_id, self.last_id) = (first_id, last_id);
    }
}

impl<'a> Menu<'a> {
    fn new(
        out: StdoutLock<'a>,
        lines: &'a Vec<Box<dyn Entry + 'a>>,
        path_ids: Vec<usize>,
    ) -> io::Result<Menu<'a>> {
        let mut term = term::Term::new(out)?;
        term.claim()?;

        let (scroll_offset, big_jump, small_jump) = Menu::scroll_info(term.height);
        let max_line_id = lines.len() - 1;

        Ok(Menu {
            selected_id: 0,
            cursor_y: START_Y,
            first_y: 0,
            last_y: 0,
            term,
            max_line_id,
            window: Window {
                first_id: 0,
                last_id: 0,
                max_id: max_line_id,
            },
            lines,
            colors: config().colors,
            pi: PathInfo::new(path_ids),
            scroll_offset,
            big_jump,
            small_jump,
            popup_open: false,
        })
    }

    fn height(&self) -> u16 {
        self.term.height
    }

    fn max_cursor_y(&self) -> u16 {
        self.height() - 1
    }

    fn width(&self) -> u16 {
        self.term.width()
    }

    fn down_page(&mut self) -> io::Result<()> {
        let dist = if self.max_cursor_y() as usize > self.max_line_id {
            self.max_line_id - self.selected_id
        } else {
            (self.height() as usize).min(self.max_line_id - self.selected_id)
        };
        if dist == 0 {
            return Ok(());
        }
        for i in 1..=dist {
            self.pi.down(self.selected_id + i);
        }
        self.selected_id += dist;
        self.draw()
    }

    fn up_page(&mut self) -> io::Result<()> {
        let dist = if self.max_cursor_y() as usize > self.max_line_id {
            self.selected_id
        } else {
            (self.height() as usize).min(self.selected_id)
        };
        if dist == 0 {
            return Ok(());
        }
        for i in 1..=dist {
            self.pi.up(self.selected_id - i);
        }
        self.selected_id -= dist;
        self.draw()
    }

    fn scroll_info(num_rows: u16) -> (u16, u16, u16) {
        let scroll_offset = num_rows / 5;
        let big_jump = scroll_offset;
        let small_jump = 1;
        (scroll_offset, big_jump, small_jump)
    }

    pub fn enter(
        out: StdoutLock<'a>,
        lines: &'a Vec<Box<dyn Entry + 'a>>,
        path_ids: Vec<usize>,
    ) -> io::Result<()> {
        let mut menu: Menu = Menu::new(out, lines, path_ids)?;

        menu.draw()?;

        loop {
            let event = event::read();
            if let Ok(Event::Key(KeyEvent {
                code,
                modifiers,
                kind: crossterm::event::KeyEventKind::Press,
                ..
            })) = event
            {
                if !menu.popup_open {
                    match code {
                        KeyCode::Char('j') | KeyCode::Char('n') | KeyCode::Down => {
                            menu.down(menu.small_jump)?
                        }
                        KeyCode::Char('k') | KeyCode::Char('p') | KeyCode::Up => {
                            menu.up(menu.small_jump)?
                        }
                        KeyCode::Char('J') | KeyCode::Char('N') => menu.down(menu.big_jump)?,
                        KeyCode::Char('K') | KeyCode::Char('P') => menu.up(menu.big_jump)?,
                        KeyCode::Char('}') | KeyCode::Char(']') => menu.down_path()?,
                        KeyCode::Char('{') | KeyCode::Char('[') => menu.up_path()?,
                        KeyCode::Char('G') | KeyCode::Char('>') | KeyCode::End => menu.bottom()?,
                        KeyCode::Char('g') | KeyCode::Char('<') | KeyCode::Home => menu.top()?,
                        KeyCode::Char('f') | KeyCode::PageDown => menu.down_page()?,
                        KeyCode::Char('b') | KeyCode::PageUp => menu.up_page()?,
                        KeyCode::Char('h') => {
                            menu.popup(MENU_HELP.to_string() + "\npress q to quit this popup")?
                        }
                        KeyCode::Char('z')
                            if !modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            menu.center_cursor()?
                        }
                        KeyCode::Char('l') => menu.center_cursor()?,
                        KeyCode::Enter => {
                            let selected = &menu.lines.get(menu.selected_id).unwrap();
                            match selected.open_info() {
                                Ok(info) => {
                                    return menu.exit_and_open(
                                        info.path.as_os_str().to_os_string(),
                                        info.line,
                                    )
                                }
                                Err(mes) => menu.popup(mes.mes)?,
                            }
                        }
                        _ => {}
                    }
                }
                match code {
                    KeyCode::Char('q') => {
                        if menu.popup_open {
                            menu.popup_open = false;
                            menu.draw()?;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Char('z') => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            menu.suspend()?;
                            menu.resume()?;
                        }
                    }
                    KeyCode::Char('c') => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            break;
                        }
                    }
                    _ => {}
                }
            } else if let Ok(Event::Resize(new_width, new_height)) = event {
                if menu.height() != new_height || menu.width() != new_width {
                    menu.resize(new_height, new_width)?;
                }
            }
        }
        menu.give_up_term()
    }

    fn draw(&mut self) -> io::Result<()> {
        self.term.clear()?;

        let skip: usize;
        let count_above_cursor: usize;
        if self.selected_id > self.cursor_y as usize {
            skip = self.selected_id - self.cursor_y as usize;
            count_above_cursor = self.cursor_y as usize;
        } else {
            skip = 0;
            count_above_cursor = self.selected_id;
        };

        let take: usize = (count_above_cursor + (self.height() - self.cursor_y) as usize)
            .min(self.max_line_id + 1);
        self.window.set(skip, skip + take - 1);
        let start_cursor = START_Y + self.cursor_y - count_above_cursor as u16;

        for (i, line) in self.lines.iter().skip(skip).take(take).enumerate() {
            let cursor = start_cursor + i as u16;
            if i + skip == 0 {
                self.first_y = cursor;
            }
            if i + skip == self.window.max_id {
                self.last_y = cursor;
            }
            queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
        }
        self.draw_selected()?;
        self.popup_open = false;
        self.term.flush()
    }

    fn down(&mut self, try_dist: u16) -> io::Result<()> {
        self.destyle_selected()?;
        let dist: usize = (try_dist as usize).min(self.max_line_id - self.selected_id);
        let max_cursor_y = self.max_cursor_y();

        for _ in 0..dist {
            self.selected_id += 1;
            self.pi.down(self.selected_id);
            if self.cursor_y + self.scroll_offset < self.max_cursor_y() || self.bot_visible() {
                self.cursor_y += 1;
            } else {
                if self.top_visible() && self.first_y == 0 {
                    self.window.down_shift_first();
                }
                if self.first_y > 0 {
                    self.first_y -= 1;
                }
                let id = self.selected_id + self.scroll_offset as usize;
                self.window.set_last(id);
                queue!(
                    self.term,
                    terminal::ScrollUp(1),
                    cursor::MoveTo(START_X, max_cursor_y),
                    Print(self.lines.get(id).unwrap())
                )?;
            }
        }
        self.draw_selected()?;
        self.term.flush()
    }

    fn up(&mut self, try_dist: u16) -> io::Result<()> {
        self.destyle_selected()?;
        let dist: usize = (try_dist as usize).min(self.selected_id);
        for _ in 0..dist {
            self.selected_id -= 1;
            self.pi.up(self.selected_id);
            if self.cursor_y > self.scroll_offset || self.top_visible() {
                self.cursor_y -= 1;
            } else {
                if self.bot_visible() && self.last_y == self.max_cursor_y() {
                    self.window.up_shift_last();
                }
                let id = self.selected_id - self.cursor_y as usize;
                self.window.set_first(id);
                if self.last_y < self.max_cursor_y() {
                    self.last_y += 1;
                }
                queue!(
                    self.term,
                    terminal::ScrollDown(1),
                    cursor::MoveTo(START_X, START_Y),
                    Print(
                        self.lines
                            .get(self.selected_id - self.cursor_y as usize)
                            .unwrap()
                    )
                )?;
            }
        }
        self.draw_selected()?;
        self.term.flush()
    }

    fn center_cursor(&mut self) -> io::Result<()> {
        let mid = self.height() / 2;
        if self.cursor_y != mid {
            self.cursor_y = mid;
            self.draw()?;
        }
        Ok(())
    }

    fn resize(&mut self, new_height: u16, new_width: u16) -> io::Result<()> {
        self.term.set_dims((new_width, new_height));
        if self.selected_id > (self.height() / 2) as usize {
            self.center_cursor()
        } else {
            self.draw()
        }
    }

    fn suspend(&mut self) -> io::Result<()> {
        #[cfg(not(windows))]
        {
            self.give_up_term()?;
            signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP).unwrap();
        }
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        let orig_height = self.height();
        self.term.set_dims(terminal::size()?);
        (self.scroll_offset, self.big_jump, self.small_jump) = Menu::scroll_info(self.height());
        self.term.claim()?;
        if self.height() != orig_height {
            self.center_cursor()?;
        } else {
            self.draw()?;
        }
        Ok(())
    }

    fn bottom(&mut self) -> io::Result<()> {
        if self.selected_id == self.max_line_id {
            return Ok(());
        }
        self.pi.bottom();
        self.selected_id = self.max_line_id;
        if self.window.last_id >= self.window.max_id {
            self.cursor_y = self.last_y;
        } else {
            self.cursor_y = self.max_cursor_y();
        }
        self.draw()
    }

    fn top(&mut self) -> io::Result<()> {
        if self.selected_id == 0 {
            return Ok(());
        }
        self.pi.top();
        self.selected_id = 0;
        if self.window.first_id == 0 {
            self.cursor_y = self.first_y;
        } else {
            self.cursor_y = 0;
        }
        self.draw()
    }

    fn top_visible(&self) -> bool {
        self.window.first_id == 0
    }

    fn bot_visible(&self) -> bool {
        self.window.last_id == self.max_line_id
    }

    pub fn down_path(&mut self) -> io::Result<()> {
        if config().long_branch {
            return self.down(1);
        }
        let dist = self.pi.dist_down(self.selected_id);
        if dist != 0 {
            self.down(dist)?;
        }
        Ok(())
    }

    fn up_path(&mut self) -> io::Result<()> {
        if config().long_branch {
            return self.up(1);
        }
        let dist = self.pi.dist_up(self.selected_id);
        if dist != 0 {
            self.up(dist)?;
        }
        Ok(())
    }

    fn draw_selected(&mut self) -> io::Result<()> {
        if self.colors {
            queue!(self.term, SetBackgroundColor(formats::MENU_SELECTED))?;
        }
        queue!(
            self.term,
            cursor::MoveTo(0, self.cursor_y),
            Print(config().c.selected_indicator),
            cursor::MoveTo(START_X, self.cursor_y),
            Print(self.lines.get(self.selected_id).unwrap())
        )
    }

    fn destyle_selected(&mut self) -> io::Result<()> {
        queue!(
            self.term,
            cursor::MoveTo(0, self.cursor_y),
            Print(formats::SELECTED_INDICATOR_CLEAR),
            cursor::MoveTo(START_X, self.cursor_y),
            Print(self.lines.get(self.selected_id).unwrap())
        )
    }

    fn popup(&mut self, content: String) -> io::Result<()> {
        let lines: Vec<&str> = content.lines().collect();
        let content_width = lines.iter().map(|line| line.len()).max().unwrap() as u16;
        let height = lines.len() as u16 + 2;
        let x = self.width().saturating_sub(content_width) / 2;
        let y = self.height().saturating_sub(height) / 2;

        queue!(
            self.term,
            cursor::MoveTo(x, y),
            Print(format!(
                "{}{}{}",
                config().c.tl,
                formats::repeat(config().c.h, content_width as usize),
                config().c.tr,
            ))
        )?;

        for (i, line) in lines.iter().enumerate() {
            queue!(
                self.term,
                cursor::MoveTo(x, y + i as u16 + 1),
                Print(format!(
                    "{}{:w$}{}",
                    config().c.v,
                    line,
                    config().c.v,
                    w = content_width as usize
                ),),
            )?;
        }

        queue!(
            self.term,
            cursor::MoveTo(x, y + height - 1),
            Print(format!(
                "{}{}{}",
                config().c.bl,
                formats::repeat(config().c.h, content_width as usize),
                config().c.br,
            ))
        )?;
        self.popup_open = true;
        self.term.flush()?;

        Ok(())
    }

    fn give_up_term(&mut self) -> io::Result<()> {
        self.term.give()
    }

    #[cfg(target_os = "windows")]
    fn exit_and_open(&mut self, mut path: OsString, line_num: Option<usize>) -> io::Result<()> {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(path)
            .spawn()?;
        self.give_up_term()
    }

    #[cfg(unix)]
    fn exit_and_open(&mut self, mut path: OsString, line_num: Option<usize>) -> io::Result<()> {
        let opener = std::env::var("EDITOR")
            .ok()
            .filter(|val| !val.is_empty())
            .unwrap_or_else(|| {
                if cfg!(target_os = "macos") {
                    "open".to_string()
                } else {
                    "xdg-open".to_string()
                }
            });

        let mut command: Command = Command::new(&opener);
        match opener.as_str() {
            "vi" | "vim" | "nvim" | "nano" | "emacs" | "jove" | "kak" | "micro" => {
                if let Some(l) = line_num {
                    command.arg(format!("+{l}"));
                }
                command.arg(path);
            }
            "hx" => {
                if let Some(l) = line_num {
                    path.push(format!(":{l}"));
                    command.arg(path);
                } else {
                    command.arg(path);
                }
            }
            "code" => {
                if let Some(l) = line_num {
                    command.arg("--goto");
                    path.push(format!(":{l}"));
                    command.arg(path);
                } else {
                    command.arg(path);
                }
            }
            "jed" | "xjed" => {
                command.arg(path);
                if let Some(l) = line_num {
                    command.arg("-g");
                    command.arg(format!("{l}"));
                }
            }
            _ => {
                command.arg(path);
            }
        }
        self.give_up_term()?;

        use std::os::unix::process::CommandExt;
        let _ = command.exec();
        Ok(())
    }
}
