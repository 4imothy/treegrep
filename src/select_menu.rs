// SPDX-License-Identifier: MIT

use crate::{args::OpenStrategy, config, errors::SUBMIT_ISSUE, style, term::Term, writer::Entry};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEventKind},
    queue,
    style::{Print, SetBackgroundColor},
    terminal,
};
use std::{
    ffi::OsString,
    io::{self, Write},
    process::Command,
};

const START_X: u16 = style::SELECTED_INDICATOR_CLEAR.len() as u16;
const START_Y: u16 = 0;

const MENU_HELP_POPUP: &str = "navigate with the following
\u{0020}- move up/down: k/j, p/n, up arrow/down arrow
\u{0020}- move up/down with a bigger jump: K/J, P/N
\u{0020}- move up/down paths: {/}, [/]
\u{0020}- move up/down paths of the same depth: (/), u/d
\u{0020}- move to the start/end: g/G, </>, home/end
\u{0020}- move up/down a page: b/f, pageup/pagedown
\u{0020}- center cursor: z/l
\u{0020}- fold/unfold path: tab
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

pub struct SelectMenu<'a, 'b> {
    jump: JumpLocation,
    selected_id: usize,
    cursor_y: u16,
    term: &'a mut Term<'b>,
    max: usize,
    lines: &'a Vec<Box<dyn Entry + 'a>>,
    colors: bool,
    scroll_offset: u16,
    big_jump: u16,
    small_jump: u16,
    popup_open: bool,
    window: Window,
    folded: Vec<usize>,
    visible: Vec<usize>,
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

#[derive(Clone, Copy)]
enum JumpLocation {
    Top,
    Middle,
    Bottom,
}

impl JumpLocation {
    fn default() -> Self {
        Self::Middle
    }
    fn reset(&mut self) {
        *self = Self::default();
    }
    fn next(&mut self) {
        *self = match *self {
            Self::Middle => Self::Top,
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Middle,
        };
    }
}

impl<'a, 'b> SelectMenu<'a, 'b> {
    pub fn enter(
        term: &'a mut Term<'b>,
        lines: &'a Vec<Box<dyn Entry + 'a>>,
    ) -> io::Result<SelectMenu<'a, 'b>> {
        let mut menu = SelectMenu {
            selected_id: 0,
            jump: JumpLocation::default(),
            cursor_y: START_Y,
            term,
            max: lines.len() - 1,
            window: Window { first: 0, last: 0 },
            lines,
            colors: config().with_colors,
            scroll_offset: 0,
            big_jump: 0,
            small_jump: 0,
            popup_open: false,
            folded: Vec::new(),
            visible: (0..lines.len()).collect(),
        };
        menu.update_offsets_and_jumps();
        Ok(menu)
    }

    fn down_select(&mut self, amount: usize) {
        self.selected_id += amount;
    }

    fn up_select(&mut self, amount: usize) {
        self.selected_id -= amount;
    }

    fn max_cursor_y(&self) -> u16 {
        self.term.height - 1
    }

    fn down_page(&mut self) -> io::Result<()> {
        let dist = (self.term.height as usize).min(self.max - self.selected_id);
        if dist != 0 {
            self.jump_down(dist)
        } else {
            Ok(())
        }
    }

    fn up_page(&mut self) -> io::Result<()> {
        let dist = (self.term.height as usize).min(self.selected_id);
        if dist != 0 {
            self.jump_up(dist)
        } else {
            Ok(())
        }
    }

    fn update_offsets_and_jumps(&mut self) {
        self.scroll_offset = self.term.height / 5;
        self.big_jump = self.scroll_offset;
        self.small_jump = 1;
    }

    fn print_line(&mut self, orig: usize) -> io::Result<()> {
        queue!(self.term, Print(&self.lines[orig]))?;
        if self.folded.contains(&orig) {
            queue!(self.term, Print(config().chars.ellipsis))?;
        }
        Ok(())
    }

    fn fold_end(&self, orig: usize) -> usize {
        let depth = self.lines[orig].depth();
        let mut end = orig + 1;
        while end < self.lines.len() && self.lines[end].depth() > depth {
            end += 1;
        }
        end
    }

    fn toggle_fold(&mut self) -> io::Result<()> {
        let orig = self.visible[self.selected_id];
        if !self.lines[orig].is_path() {
            return Ok(());
        }
        let fold_end = self.fold_end(orig);
        let after = self.selected_id + 1;
        if let Some(pos) = self.folded.iter().position(|&v| v == orig) {
            self.folded.swap_remove(pos);
            let mut to_insert = Vec::new();
            let mut i = orig + 1;
            while i < fold_end {
                to_insert.push(i);
                if self.folded.contains(&i) {
                    i = self.fold_end(i);
                } else {
                    i += 1;
                }
            }
            self.visible.splice(after..after, to_insert);
        } else {
            self.folded.push(orig);
            let end = self.visible[after..]
                .iter()
                .position(|&o| o >= fold_end)
                .map_or(self.visible.len(), |p| after + p);
            self.visible.drain(after..end);
        }
        self.max = self.visible.len() - 1;
        self.draw()
    }

    pub fn launch(term: &mut Term, lines: &'a Vec<Box<dyn Entry + 'a>>) -> io::Result<()> {
        let mut menu: SelectMenu = SelectMenu::enter(term, lines)?;

        menu.draw()?;
        let mut down_row: u16 = 0;
        let mut cursor_jump;

        loop {
            cursor_jump = false;
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
                            KeyCode::Char(')') | KeyCode::Char('d') => {
                                menu.down_path_same_depth()?
                            }
                            KeyCode::Char('(') | KeyCode::Char('u') => menu.up_path_same_depth()?,
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
                                menu.jump_cursor(menu.jump)?;
                                menu.jump.next();
                                cursor_jump = true;
                            }
                            KeyCode::Char('l') => {
                                menu.jump_cursor(menu.jump)?;
                                menu.jump.next();
                                cursor_jump = true;
                            }
                            KeyCode::Tab => menu.toggle_fold()?,
                            KeyCode::Enter => {
                                let selected = &menu.lines[menu.visible[menu.selected_id]];
                                match selected.open_info() {
                                    Ok(info) => {
                                        if let Some(f) = &config().selection_file {
                                            let mut buf = Vec::new();
                                            buf.extend_from_slice(
                                                info.path.as_os_str().as_encoded_bytes(),
                                            );
                                            buf.push(b'\n');
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
                                menu.term.suspend()?;
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
            if !cursor_jump {
                menu.jump.reset();
            }
        }
        menu.term.give()
    }

    fn click_on(&mut self, row: u16) -> io::Result<()> {
        let cursor_y = self.cursor_y as isize;
        let selected_id = self.selected_id as isize;
        let visible_len = self.visible.len() as isize;

        let start_results = cursor_y - selected_id;
        let end_results = cursor_y + visible_len - selected_id;
        if (row as isize) < start_results || (row as isize) >= end_results {
            return Ok(());
        }
        self.destyle_selected()?;
        if self.cursor_y > row {
            self.up_select((self.cursor_y - row) as usize);
        } else {
            self.down_select((row - self.cursor_y) as usize);
        }
        self.cursor_y = row;
        self.style_selected()?;
        self.term.flush()
    }

    fn draw(&mut self) -> io::Result<()> {
        self.term.clear()?;

        let first = self.selected_id as isize - self.cursor_y as isize;

        let count_above_cursor = (self.cursor_y as usize).min(self.selected_id);

        let take =
            (count_above_cursor + (self.term.height - self.cursor_y) as usize).min(self.max + 1);
        let skip = first.max(0) as usize;
        self.window.set(first, (skip + take - 1) as isize);

        let start_cursor = START_Y + self.cursor_y - count_above_cursor as u16;

        for i in 0..take.min(self.visible.len().saturating_sub(skip)) {
            let orig = self.visible[skip + i];
            queue!(self.term, cursor::MoveTo(START_X, start_cursor + i as u16))?;
            self.print_line(orig)?;
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
        let orig = (line_id >= 0)
            .then(|| self.visible.get(line_id as usize).copied())
            .flatten();
        if let Some(orig) = orig {
            queue!(self.term, scroll, cursor::MoveTo(START_X, y))?;
            self.print_line(orig)?;
        }
        Ok(())
    }

    fn down_scroll(&mut self) -> io::Result<()> {
        if self.cursor_y < self.scroll_offset || self.bot_visible() {
            self.down(self.small_jump)
        } else {
            self.destyle_selected()?;
            self.down_select(1);
            self.scroll_and_fill_line(false, &terminal::ScrollUp(1), self.max_cursor_y())?;
            self.style_selected()?;
            self.term.flush()
        }
    }

    fn up_scroll(&mut self) -> io::Result<()> {
        if self.cursor_y + self.scroll_offset > self.max_cursor_y() || self.top_visible() {
            self.up(self.small_jump)
        } else {
            self.destyle_selected()?;
            self.up_select(1);
            self.scroll_and_fill_line(true, &terminal::ScrollDown(1), START_Y)?;
            self.style_selected()?;
            self.term.flush()
        }
    }

    fn down(&mut self, try_dist: u16) -> io::Result<()> {
        let dist: usize = (try_dist as usize).min(self.max - self.selected_id);
        if dist != 0 {
            self.destyle_selected()?;
            let max_cursor_y = self.max_cursor_y();
            self.down_select(dist);
            for _ in 0..dist {
                if self.cursor_y + self.scroll_offset < max_cursor_y || self.bot_visible() {
                    self.cursor_y += 1;
                } else {
                    self.scroll_and_fill_line(false, &terminal::ScrollUp(1), max_cursor_y)?;
                }
            }
            self.style_selected()?;
            self.term.flush()
        } else {
            Ok(())
        }
    }

    fn up(&mut self, try_dist: u16) -> io::Result<()> {
        let dist: usize = (try_dist as usize).min(self.selected_id);
        if dist != 0 {
            self.destyle_selected()?;
            self.up_select(dist);
            for _ in 0..dist {
                if self.cursor_y > self.scroll_offset || self.top_visible() {
                    self.cursor_y -= 1;
                } else {
                    self.scroll_and_fill_line(true, &terminal::ScrollDown(1), START_Y)?;
                }
            }
            self.style_selected()?;
            self.term.flush()
        } else {
            Ok(())
        }
    }

    fn jump_cursor(&mut self, loc: JumpLocation) -> io::Result<()> {
        let y = match loc {
            JumpLocation::Middle => self.term.height / 2,
            JumpLocation::Top => 0,
            JumpLocation::Bottom => self.max_cursor_y(),
        };
        if self.cursor_y != y {
            self.cursor_y = y;
            self.draw()
        } else {
            Ok(())
        }
    }

    fn resize(&mut self, new_height: u16, new_width: u16) -> io::Result<()> {
        self.term.set_dims(new_height, new_width);
        self.update_offsets_and_jumps();
        if self.cursor_y as usize > (self.term.height / 2) as usize {
            self.jump_cursor(JumpLocation::Middle)
        } else {
            self.draw()
        }
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
                self.jump_cursor(JumpLocation::Middle)?;
            } else {
                self.draw()?;
            }
        }
        Ok(())
    }

    fn bottom(&mut self) -> io::Result<()> {
        if self.selected_id == self.max {
            Ok(())
        } else if self.bot_visible() {
            self.down((self.max - self.selected_id) as u16)
        } else {
            self.down_select(self.max - self.selected_id);
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
            self.up_select(self.selected_id);
            self.cursor_y = 0;
            self.draw()
        }
    }

    fn top_visible(&self) -> bool {
        self.window.first <= 0
    }

    fn bot_visible(&self) -> bool {
        self.window.last >= self.max as isize
    }

    pub fn down_path(&mut self) -> io::Result<()> {
        let after = &self.visible[self.selected_id + 1..];
        match after.iter().position(|&o| self.lines[o].is_path()) {
            Some(d) => self.jump_down(d + 1),
            None => Ok(()),
        }
    }

    fn up_path(&mut self) -> io::Result<()> {
        let before = &self.visible[..self.selected_id];
        match before.iter().rposition(|&o| self.lines[o].is_path()) {
            Some(i) => self.jump_up(self.selected_id - i),
            None => Ok(()),
        }
    }

    fn down_path_same_depth(&mut self) -> io::Result<()> {
        let orig = self.visible[self.selected_id];
        if !self.lines[orig].is_path() {
            return self.down_path();
        }
        let depth = self.lines[orig].depth();
        let after = &self.visible[self.selected_id + 1..];
        match after
            .iter()
            .position(|&o| self.lines[o].is_path() && self.lines[o].depth() == depth)
        {
            Some(d) => self.jump_down(d + 1),
            None => Ok(()),
        }
    }

    fn up_path_same_depth(&mut self) -> io::Result<()> {
        let orig = self.visible[self.selected_id];
        if !self.lines[orig].is_path() {
            return self.up_path();
        }
        let depth = self.lines[orig].depth();
        let before = &self.visible[..self.selected_id];
        match before
            .iter()
            .rposition(|&o| self.lines[o].is_path() && self.lines[o].depth() == depth)
        {
            Some(i) => self.jump_up(self.selected_id - i),
            None => Ok(()),
        }
    }

    fn jump_up(&mut self, dist: usize) -> io::Result<()> {
        if (self.selected_id - dist) < self.window.first.max(0) as usize {
            self.up_select(dist);
            self.draw()
        } else {
            self.destyle_selected()?;
            self.up_select(dist);
            self.cursor_y -= dist as u16;
            self.style_selected()?;
            self.term.flush()
        }
    }

    fn jump_down(&mut self, dist: usize) -> io::Result<()> {
        if self.selected_id + dist > self.window.last.max(0) as usize {
            self.down_select(dist);
            self.draw()
        } else {
            self.destyle_selected()?;
            self.down_select(dist);
            self.cursor_y += dist as u16;
            self.style_selected()?;
            self.term.flush()
        }
    }

    fn style_selected(&mut self) -> io::Result<()> {
        queue!(self.term, cursor::MoveTo(0, self.cursor_y),)?;
        if self.colors {
            queue!(self.term, SetBackgroundColor(config().colors.selected_bg))?;
        }
        if self.colors
            && let Some(c) = config().colors.selected_indicator
        {
            queue!(
                self.term,
                Print(style::style_with(config().chars.selected_indicator, c)),
                SetBackgroundColor(config().colors.selected_bg)
            )?;
        } else {
            queue!(self.term, Print(config().chars.selected_indicator),)?;
        }
        queue!(self.term, cursor::MoveTo(START_X, self.cursor_y))?;
        self.print_line(self.visible[self.selected_id])
    }

    fn destyle_selected(&mut self) -> io::Result<()> {
        queue!(
            self.term,
            cursor::MoveTo(0, self.cursor_y),
            Print(style::SELECTED_INDICATOR_CLEAR),
            cursor::MoveTo(START_X, self.cursor_y),
        )?;
        self.print_line(self.visible[self.selected_id])
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
                config().chars.tl,
                style::repeat(config().chars.h, content_width as usize),
                config().chars.tr,
            ))
        )?;

        for (i, line) in lines.iter().enumerate() {
            queue!(
                self.term,
                cursor::MoveTo(x, y + i as u16 + 1),
                Print(format!(
                    "{}{:w$}{}",
                    config().chars.v,
                    line,
                    config().chars.v,
                    w = content_width as usize
                )),
            )?;
        }

        queue!(
            self.term,
            cursor::MoveTo(x, y + height - 1),
            Print(format!(
                "{}{}{}",
                config().chars.bl,
                style::repeat(config().chars.h, content_width as usize),
                config().chars.br,
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
