// SPDX-License-Identifier: MIT

use crate::args::OpenStrategy;
use crate::errors::SUBMIT_ISSUE;
use crate::term::Term;
use crate::{config, formats, writer::Entry};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEventKind},
    queue,
    style::{Print, SetBackgroundColor},
    terminal,
};
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::Command;

const START_X: u16 = formats::SELECTED_INDICATOR_CLEAR.len() as u16;
const START_Y: u16 = 0;

struct PathInfo {
    paths: Vec<usize>,
    prev: usize,
    next: usize,
    passed: bool,
}

const MENU_HELP_POPUP: &str = "navigate with the following
\u{0020}- move up/down: k/j, p/n, up arrow/down arrow
\u{0020}- move up/down with a bigger jump: K/J, P/N
\u{0020}- move up/down paths: {/}, [/]
\u{0020}- move to the start/end: g/G, </>, home/end
\u{0020}- move up/down a page: b/f, pageup/pagedown
\u{0020}- center cursor: z/l
\u{0020}- open selection: enter
\u{0020}- scrolling and clicking
\u{0020}- quit: q, ctrl + c
press q to quit this popup";

impl OpenStrategy {
    fn from(editor: &str) -> Self {
        match editor {
            "vi" | "vim" | "nvim" | "nano" | "emacs" | "jove" | "kak" | "micro" => Self::Vi,
            "hx" => Self::Hx,
            "code" => Self::Code,
            "jed" | "xjed" => Self::Jed,
            _ => Self::Default,
        }
    }
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

pub struct PickerMenu<'a, 'b> {
    pi: PathInfo,
    selected_id: usize,
    cursor_y: u16,
    term: &'a mut Term<'b>,
    max_line_id: usize,
    lines: &'a Vec<Box<dyn Entry + 'a>>,
    colors: bool,
    scroll_offset: u16,
    big_jump: u16,
    small_jump: u16,
    popup_open: bool,
    window: Window,
}

struct Window {
    first: isize,
    last: isize,
}

impl Window {
    pub fn shift_up(&mut self) {
        self.first -= 1;
        self.last -= 1;
    }
    pub fn shift_down(&mut self) {
        self.first += 1;
        self.last += 1;
    }
    pub fn set(&mut self, first_id: isize, last_id: isize) {
        (self.first, self.last) = (first_id, last_id);
    }
}

impl<'a, 'b> PickerMenu<'a, 'b> {
    fn new(
        term: &'a mut Term<'b>,
        lines: &'a Vec<Box<dyn Entry + 'a>>,
        path_ids: Vec<usize>,
    ) -> io::Result<PickerMenu<'a, 'b>> {
        if !config().menu {
            term.claim()?;
        }

        let max_line_id = lines.len() - 1;

        let mut menu = PickerMenu {
            selected_id: 0,
            cursor_y: START_Y,
            term,
            max_line_id,
            window: Window { first: 0, last: 0 },
            lines,
            colors: config().colors,
            pi: PathInfo::new(path_ids),
            scroll_offset: 0,
            big_jump: 0,
            small_jump: 0,
            popup_open: false,
        };
        menu.update_offsets_and_jumps();
        Ok(menu)
    }

    fn inc_selected(&mut self, amount: usize) {
        for i in 1..=amount {
            self.pi.down(self.selected_id + i);
        }
        self.selected_id += amount;
    }

    fn dec_selected(&mut self, amount: usize) {
        for i in 1..=amount {
            self.pi.up(self.selected_id - i);
        }
        self.selected_id -= amount;
    }

    fn max_cursor_y(&self) -> u16 {
        self.term.height - 1
    }

    fn down_page(&mut self) -> io::Result<()> {
        let dist = (self.term.height as usize).min(self.max_line_id - self.selected_id);
        if dist != 0 {
            self.inc_jump(dist)
        } else {
            Ok(())
        }
    }

    fn up_page(&mut self) -> io::Result<()> {
        let dist = (self.term.height as usize).min(self.selected_id);
        if dist != 0 {
            self.dec_jump(dist)
        } else {
            Ok(())
        }
    }

    fn update_offsets_and_jumps(&mut self) {
        self.scroll_offset = self.term.height / 5;
        self.big_jump = self.scroll_offset;
        self.small_jump = 1;
    }

    pub fn enter(
        term: &mut Term,
        lines: &'a Vec<Box<dyn Entry + 'a>>,
        path_ids: Vec<usize>,
    ) -> io::Result<()> {
        let mut menu: PickerMenu = PickerMenu::new(term, lines, path_ids)?;

        menu.draw()?;
        let mut down_row: u16 = 0;

        loop {
            let event = event::read()?;
            match event {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
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
                            KeyCode::Char('G') | KeyCode::Char('>') | KeyCode::End => {
                                menu.bottom()?
                            }
                            KeyCode::Char('g') | KeyCode::Char('<') | KeyCode::Home => {
                                menu.top()?
                            }
                            KeyCode::Char('f') | KeyCode::PageDown => menu.down_page()?,
                            KeyCode::Char('b') | KeyCode::PageUp => menu.up_page()?,
                            KeyCode::Char('h') => menu.popup(MENU_HELP_POPUP.to_string())?,
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
                                        if let Some(f) = &config().selection_file {
                                            let mut buf = Vec::new();
                                            buf.extend_from_slice(
                                                info.path.as_os_str().as_encoded_bytes(),
                                            );
                                            buf.push(formats::NEW_LINE as u8);
                                            if let Some(l) = info.line {
                                                buf.extend_from_slice(l.to_string().as_bytes());
                                            }
                                            std::fs::write(f, buf)?;
                                            break;
                                        } else {
                                            menu.term.give()?;
                                            return menu.exit_and_open(
                                                info.path.as_os_str().to_os_string(),
                                                info.line,
                                            );
                                        }
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
                }
                Event::Mouse(mouse_event) => {
                    if !menu.popup_open {
                        match mouse_event.kind {
                            MouseEventKind::ScrollUp => {
                                menu.up_scroll()?;
                            }
                            MouseEventKind::ScrollDown => {
                                menu.down_scroll()?;
                            }
                            MouseEventKind::Down(button) => {
                                if button.is_left() {
                                    down_row = mouse_event.row;
                                }
                            }
                            MouseEventKind::Up(button) => {
                                if button.is_left() && mouse_event.row == down_row {
                                    menu.click_on(down_row)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize(new_width, new_height) => {
                    if menu.term.height != new_height || menu.term.width() != new_width {
                        menu.resize(new_height, new_width)?;
                    }
                }
                _ => {}
            }
        }
        menu.term.give()
    }

    fn click_on(&mut self, row: u16) -> io::Result<()> {
        let cursor_y = self.cursor_y as isize;
        let selected_id = self.selected_id as isize;
        let lines_len = self.lines.len() as isize;

        let start_results = cursor_y - selected_id;
        let end_results = cursor_y + lines_len - selected_id;
        if (row as isize) < start_results || (row as isize) >= end_results {
            return Ok(());
        }
        if self.cursor_y > row {
            self.dec_selected((self.cursor_y - row) as usize);
        } else {
            self.inc_selected((row - self.cursor_y) as usize);
        }
        self.cursor_y = row;
        self.draw()
    }

    fn draw(&mut self) -> io::Result<()> {
        self.term.clear()?;

        let first = self.selected_id as isize - self.cursor_y as isize;

        let count_above_cursor = (self.cursor_y as usize).min(self.selected_id);

        let take = (count_above_cursor + (self.term.height - self.cursor_y) as usize)
            .min(self.max_line_id + 1);
        let skip = first.max(0) as usize;
        self.window.set(first, (skip + take - 1) as isize);

        let start_cursor = START_Y + self.cursor_y - count_above_cursor as u16;

        for (i, line) in self.lines.iter().skip(skip).take(take).enumerate() {
            let cursor = start_cursor + i as u16;
            queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
        }
        self.style_selected()?;
        self.popup_open = false;
        self.term.flush()
    }

    fn scroll_and_fill_line<C: crossterm::Command>(
        &mut self,
        down: bool,
        scroll: &C,
        y: u16,
    ) -> io::Result<()> {
        let line_id = if down {
            self.window.shift_up();
            self.window.first
        } else {
            self.window.shift_down();
            self.window.last
        };
        if let Some(line) = (line_id >= 0).then(|| self.lines.get(line_id as usize).unwrap()) {
            queue!(self.term, scroll, cursor::MoveTo(START_X, y), Print(line))?;
        }
        Ok(())
    }

    fn down_scroll(&mut self) -> io::Result<()> {
        if self.cursor_y < self.scroll_offset || self.bot_visible() {
            self.down(self.small_jump)
        } else {
            let max_cursor_y = self.max_cursor_y();
            self.destyle_selected()?;
            self.inc_selected(1);
            self.scroll_and_fill_line(false, &terminal::ScrollUp(1), max_cursor_y)?;
            self.style_selected()?;
            self.term.flush()
        }
    }

    fn up_scroll(&mut self) -> io::Result<()> {
        if self.cursor_y + self.scroll_offset > self.max_cursor_y() || self.top_visible() {
            self.up(self.small_jump)
        } else {
            self.destyle_selected()?;
            self.dec_selected(1);
            self.scroll_and_fill_line(true, &terminal::ScrollDown(1), START_Y)?;
            self.style_selected()?;
            self.term.flush()
        }
    }

    fn down(&mut self, try_dist: u16) -> io::Result<()> {
        self.destyle_selected()?;
        let dist: usize = (try_dist as usize).min(self.max_line_id - self.selected_id);
        let max_cursor_y = self.max_cursor_y();
        self.inc_selected(dist);
        for _ in 0..dist {
            if self.cursor_y + self.scroll_offset < max_cursor_y || self.bot_visible() {
                self.cursor_y += 1;
            } else {
                self.scroll_and_fill_line(false, &terminal::ScrollUp(1), max_cursor_y)?;
            }
        }
        self.style_selected()?;
        self.term.flush()
    }

    fn up(&mut self, try_dist: u16) -> io::Result<()> {
        self.destyle_selected()?;
        let dist: usize = (try_dist as usize).min(self.selected_id);
        self.dec_selected(dist);
        for _ in 0..dist {
            if self.cursor_y > self.scroll_offset || self.top_visible() {
                self.cursor_y -= 1;
            } else {
                self.scroll_and_fill_line(true, &terminal::ScrollDown(1), START_Y)?;
            }
        }
        self.style_selected()?;
        self.term.flush()
    }

    fn center_cursor(&mut self) -> io::Result<()> {
        let mid = self.term.height / 2;
        if self.cursor_y != mid {
            self.cursor_y = mid;
            self.draw()?;
        }
        Ok(())
    }

    fn resize(&mut self, new_height: u16, new_width: u16) -> io::Result<()> {
        self.term.set_dims(new_height, new_width);
        self.update_offsets_and_jumps();
        if self.cursor_y as usize > (self.term.height / 2) as usize {
            self.center_cursor()
        } else {
            self.draw()
        }
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
            let orig_height = self.term.height;
            let (width, height) = terminal::size()?;
            self.term.set_dims(height, width);
            self.update_offsets_and_jumps();
            self.term.claim()?;
            if self.term.height != orig_height {
                self.center_cursor()?;
            } else {
                self.draw()?;
            }
        }
        Ok(())
    }

    fn bottom(&mut self) -> io::Result<()> {
        if self.selected_id == self.max_line_id {
            Ok(())
        } else if self.bot_visible() {
            self.down((self.max_line_id - self.selected_id) as u16)
        } else {
            self.inc_selected(self.max_line_id - self.selected_id);
            self.cursor_y = self.max_cursor_y();
            self.draw()
        }
    }

    fn top(&mut self) -> io::Result<()> {
        if self.selected_id == 0 {
            Ok(())
        } else if self.top_visible() {
            self.up(self.selected_id as u16)
        } else {
            self.dec_selected(self.selected_id);
            self.cursor_y = 0;
            self.draw()
        }
    }

    fn top_visible(&self) -> bool {
        self.window.first <= 0
    }

    fn bot_visible(&self) -> bool {
        self.window.last >= self.max_line_id as isize
    }

    pub fn down_path(&mut self) -> io::Result<()> {
        let dist = self.pi.dist_down(self.selected_id);
        if dist == 0 {
            Ok(())
        } else {
            self.inc_jump(dist as usize)
        }
    }

    fn up_path(&mut self) -> io::Result<()> {
        let dist = self.pi.dist_up(self.selected_id);
        if dist == 0 {
            Ok(())
        } else {
            self.dec_jump(dist as usize)
        }
    }

    fn dec_jump(&mut self, dist: usize) -> io::Result<()> {
        if (self.selected_id - dist) < self.window.first.max(0) as usize {
            self.dec_selected(dist);
            self.draw()
        } else {
            self.destyle_selected()?;
            self.dec_selected(dist);
            self.cursor_y -= dist as u16;
            self.style_selected()?;
            self.term.flush()
        }
    }

    fn inc_jump(&mut self, dist: usize) -> io::Result<()> {
        if self.selected_id + dist > self.window.last.max(0) as usize {
            self.inc_selected(dist);
            self.draw()
        } else {
            self.destyle_selected()?;
            self.inc_selected(dist);
            self.cursor_y += dist as u16;
            self.style_selected()?;
            self.term.flush()
        }
    }

    fn style_selected(&mut self) -> io::Result<()> {
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
        let x = self.term.width().saturating_sub(content_width) / 2;
        let y = self.term.height.saturating_sub(height) / 2;

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
                )),
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

    fn exit_and_open(&mut self, mut path: OsString, line_num: Option<usize>) -> io::Result<()> {
        let env_editor = std::env::var("EDITOR").ok().filter(|s| !s.is_empty());
        let mut cmd = match config().editor.as_ref().or(env_editor.as_ref()) {
            Some(editor) => {
                let mut cmd = Command::new(editor);
                match config()
                    .open_like
                    .as_ref()
                    .unwrap_or(&OpenStrategy::from(editor))
                {
                    OpenStrategy::Vi => {
                        if let Some(line) = line_num {
                            cmd.arg(format!("+{line}"));
                        }
                    }
                    OpenStrategy::Hx => {
                        if let Some(line) = line_num {
                            path.push(format!(":{line}"));
                        }
                    }
                    OpenStrategy::Code => {
                        if let Some(line) = line_num {
                            cmd.arg("--goto");
                            path.push(format!(":{line}"));
                        }
                    }
                    OpenStrategy::Jed => {
                        if let Some(line) = line_num {
                            cmd.arg("-g").arg(format!("{line}"));
                        }
                    }
                    OpenStrategy::Default => {}
                }
                cmd.arg(&path);
                cmd
            }
            None => {
                let mut cmd = match () {
                    _ if cfg!(target_os = "macos") => Command::new("open"),
                    _ if cfg!(target_os = "windows") => Command::new("cmd"),
                    _ if cfg!(unix) => Command::new("xdg-open"),
                    _ => panic!("unable to find opener {SUBMIT_ISSUE}"),
                };
                if cfg!(windows) {
                    cmd.args(["/C", "start"]);
                }
                cmd.arg(&path);
                cmd
            }
        };

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            Err(cmd.exec())
        }
        #[cfg(not(unix))]
        {
            cmd.spawn()?;
            Ok(())
        }
    }
}
