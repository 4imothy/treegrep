// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::formats;
use crate::match_system::{Directory, File, Matches};
use crate::writer;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Print, SetBackgroundColor},
    terminal::{self, ClearType},
};
use std::ffi::OsString;
use std::io::{self, StdoutLock, Write};
use std::process::Command;

const START_X: u16 = formats::SELECTED_INDICATOR.len() as u16;
const START_Y: u16 = 0;

struct PathStore {
    paths: Vec<usize>,
    prev: usize,
    next: usize,
    past: bool,
}

impl PathStore {
    pub fn new(paths: Vec<usize>) -> PathStore {
        PathStore {
            paths,
            prev: 0,
            next: 1,
            past: false,
        }
    }

    pub fn top(&mut self) {
        self.prev = 0;
        self.next = 1;
        self.past = false;
    }

    pub fn bottom(&mut self, files: bool) {
        let shift = if files { 1 } else { 0 };
        self.prev = self.paths.len() - 1 - shift;
        self.next = self.paths.len() - shift;
        self.past = false;
    }

    pub fn shift_down(&mut self, selected_id: usize) {
        if self.past {
            self.prev += 1;
            self.past = false;
        }
        if self.next != self.paths.len() && selected_id == *self.paths.get(self.next).unwrap() {
            self.next += 1;
            self.past = true;
        }
    }

    pub fn shift_up(&mut self, selected_id: usize) {
        if self.past {
            self.next -= 1;
            self.past = false;
        }
        if self.prev != 0 && selected_id == *self.paths.get(self.prev).unwrap() {
            self.prev -= 1;
            self.past = true;
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

pub struct Menu<'a, 'b> {
    ps: PathStore,
    selected_id: usize,
    cursor_y: u16,
    out: &'a mut StdoutLock<'b>,
    searched: &'a Matches,
    lines: Vec<String>,
    num_rows: u16,
    colors: bool,
    scroll_offset: u16,
    big_jump: u16,
    small_jump: u16,
}

impl<'a, 'b> Menu<'a, 'b> {
    fn new(
        out: &'a mut StdoutLock<'b>,
        searched: &'a Matches,
        config: &Config,
    ) -> io::Result<Menu<'a, 'b>> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut path_ids: Vec<usize> = Vec::new();
        writer::write_results(&mut buffer, &searched, config, Some(&mut path_ids))?;
        let lines: Vec<String> = buffer
            .split(|&byte| byte == formats::NEW_LINE as u8)
            .map(|v| String::from_utf8_lossy(v).into())
            .collect();

        let num_rows = Menu::num_rows();
        let (scroll_offset, big_jump, small_jump) = Menu::scroll_info(num_rows);

        Ok(Menu {
            selected_id: 0,
            cursor_y: 0,
            out,
            searched,
            lines,
            colors: config.colors,
            ps: PathStore::new(path_ids),
            num_rows,
            scroll_offset,
            big_jump,
            small_jump,
        })
    }

    fn num_rows() -> u16 {
        terminal::size().ok().map(|(_, num_rows)| num_rows).unwrap()
    }

    fn page_jump(&self) -> u16 {
        self.num_rows - 1
    }

    fn max_line_id(&self) -> usize {
        self.lines.len() - 2
    }

    fn set_rows(&mut self, num_rows: u16) {
        self.num_rows = num_rows;
        (self.scroll_offset, self.big_jump, self.small_jump) = Menu::scroll_info(self.num_rows);
    }

    fn scroll_info(num_rows: u16) -> (u16, u16, u16) {
        let scroll_offset = num_rows / 5;
        let big_jump = scroll_offset;
        let small_jump = 1;
        (scroll_offset, big_jump, small_jump)
    }

    pub fn enter(out: &'a mut StdoutLock<'b>, matches: Matches, config: &Config) -> io::Result<()> {
        let mut menu: Menu = Menu::new(out, &matches, config)?;

        menu.setup_term()?;
        menu.draw(false)?;

        'outer: loop {
            let event = event::read();
            if let Ok(Event::Key(KeyEvent {
                code,
                modifiers,
                kind: crossterm::event::KeyEventKind::Press,
                ..
            })) = event
            {
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
                    KeyCode::Char('z') => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            menu.suspend()?;
                            menu.resume()?;
                        }
                    }
                    KeyCode::Char('G') | KeyCode::Char('>') | KeyCode::End => {
                        menu.bottom(config.just_files)?
                    }
                    KeyCode::Char('g') | KeyCode::Char('<') | KeyCode::Home => menu.top()?,
                    KeyCode::Char('f') | KeyCode::PageDown => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            menu.down(menu.page_jump())?;
                        }
                    }
                    KeyCode::Char('b') | KeyCode::PageUp => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            menu.up(menu.page_jump())?;
                        }
                    }
                    KeyCode::Enter => {
                        let match_info = MatchInfo::find(menu.selected_id, &menu.searched, config);
                        let path = config.path.join(match_info.path);

                        return menu
                            .exit_and_open(path.as_os_str().to_os_string(), match_info.line_num);
                    }
                    KeyCode::Char('q') => break 'outer,
                    KeyCode::Char('c') => {
                        if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            break 'outer;
                        }
                    }
                    _ => {}
                }
            } else if let Ok(Event::Resize(_, rows)) = event {
                if menu.num_rows != rows {
                    menu.set_rows(rows);
                    menu.draw(true)?;
                }
            }
        }

        menu.leave()
    }

    fn draw(&mut self, resize: bool) -> io::Result<()> {
        queue!(self.out, terminal::Clear(ClearType::All))?;
        if resize && self.selected_id > (self.num_rows / 2) as usize {
            self.cursor_y = self.num_rows / 2;
        }
        let skip: usize = if self.selected_id > self.cursor_y as usize {
            self.selected_id - self.cursor_y as usize
        } else {
            0
        };
        for (i, line) in self
            .lines
            .iter()
            .skip(skip as usize)
            .take(self.num_rows as usize)
            .enumerate()
        {
            queue!(
                self.out,
                cursor::MoveTo(START_X, START_Y + i as u16),
                Print(line)
            )?;
        }
        self.style_at_cursor()?;
        self.out.flush()
    }

    fn suspend(&mut self) -> io::Result<()> {
        #[cfg(not(windows))]
        {
            self.leave()?;
            signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP).unwrap();
        }
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        self.set_rows(Menu::num_rows());
        self.setup_term()?;
        self.draw(false)?;
        Ok(())
    }

    fn bottom(&mut self, files: bool) -> io::Result<()> {
        self.ps.bottom(files);
        self.selected_id = self.max_line_id();
        self.cursor_y = if self.max_line_id() < self.num_rows as usize {
            self.max_line_id() as u16
        } else {
            self.num_rows - self.scroll_offset
        };
        self.draw(false)
    }

    fn top(&mut self) -> io::Result<()> {
        self.ps.top();
        self.selected_id = 0;
        self.cursor_y = 0;
        self.draw(false)
    }

    fn down(&mut self, dist: u16) -> io::Result<()> {
        self.destyle_at_cursor()?;
        for _ in 0..dist {
            if self.selected_id == self.max_line_id() {
                break;
            }
            self.selected_id += 1;
            self.ps.shift_down(self.selected_id);
            if self.cursor_y + self.scroll_offset != self.num_rows {
                self.cursor_y += 1;
            } else {
                queue!(self.out, terminal::ScrollUp(1))?;
                if (self.selected_id + self.scroll_offset as usize) < self.lines.len() {
                    queue!(
                        self.out,
                        cursor::MoveTo(START_X, self.num_rows),
                        Print(
                            self.lines
                                .get(self.selected_id - 1 + self.scroll_offset as usize)
                                .unwrap()
                        )
                    )?;
                }
            }
        }
        self.style_at_cursor()?;
        self.out.flush()
    }

    fn up(&mut self, dist: u16) -> io::Result<()> {
        self.destyle_at_cursor()?;
        for _ in 0..dist {
            if self.selected_id == 0 {
                break;
            }
            self.selected_id -= 1;
            self.ps.shift_up(self.selected_id);
            if self.selected_id < self.scroll_offset as usize || self.cursor_y != self.scroll_offset
            {
                self.cursor_y -= 1;
            } else {
                queue!(self.out, terminal::ScrollDown(1))?;
                if self.selected_id + 1 > self.scroll_offset as usize {
                    queue!(
                        self.out,
                        cursor::MoveTo(START_X, START_Y),
                        Print(
                            self.lines
                                .get(self.selected_id - self.scroll_offset as usize)
                                .unwrap()
                        )
                    )?;
                }
            }
        }
        self.style_at_cursor()?;
        self.out.flush()
    }

    pub fn down_path(&mut self) -> io::Result<()> {
        let dist = self.ps.dist_down(self.selected_id);
        if dist != 0 {
            self.down(dist)?;
        }
        Ok(())
    }

    fn up_path(&mut self) -> io::Result<()> {
        let dist = self.ps.dist_up(self.selected_id);
        if dist != 0 {
            self.up(dist)?;
        }
        Ok(())
    }

    fn style_at_cursor(&mut self) -> io::Result<()> {
        if self.colors {
            queue!(self.out, SetBackgroundColor(formats::MENU_SELECTED))?;
        }
        queue!(
            self.out,
            cursor::MoveTo(0, self.cursor_y),
            Print(formats::SELECTED_INDICATOR),
            cursor::MoveTo(START_X, self.cursor_y),
            Print(self.lines.get(self.selected_id).unwrap())
        )
    }

    fn destyle_at_cursor(&mut self) -> io::Result<()> {
        queue!(
            self.out,
            cursor::MoveTo(0, self.cursor_y),
            Print(formats::SELECTED_INDICATOR_CLEAR),
            cursor::MoveTo(START_X, self.cursor_y),
            Print(self.lines.get(self.selected_id).unwrap())
        )
    }

    fn setup_term(&mut self) -> io::Result<()> {
        execute!(
            self.out,
            cursor::Hide,
            terminal::EnterAlternateScreen,
            terminal::DisableLineWrap,
        )?;
        terminal::enable_raw_mode()
    }

    fn leave(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        self.out.flush()?;
        execute!(
            io::stderr(),
            style::ResetColor,
            cursor::SetCursorStyle::DefaultUserShape,
            terminal::LeaveAlternateScreen,
            terminal::EnableLineWrap,
            cursor::Show,
        )
    }

    #[cfg(windows)]
    fn exit_and_open(&mut self, path: OsString, _line_num: Option<usize>) -> io::Result<()> {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(path)
            .spawn()?;
        self.leave()
    }

    #[cfg(not(windows))]
    fn exit_and_open(&mut self, mut path: OsString, line_num: Option<usize>) -> io::Result<()> {
        let opener = match std::env::var("EDITOR") {
            Ok(val) if !val.is_empty() => val,
            _ => match std::env::consts::OS {
                "macos" => "open".to_string(),
                _ => "xdg-open".to_string(),
            },
        };

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
        use std::os::unix::process::CommandExt;
        self.leave()?;

        command.exec();
        Ok(())
    }
}

struct MatchInfo {
    path: OsString,
    line_num: Option<usize>,
}

impl MatchInfo {
    pub fn new(path: OsString, line_num: Option<usize>) -> Self {
        MatchInfo { path, line_num }
    }

    pub fn find(selected: usize, searched: &Matches, config: &Config) -> MatchInfo {
        let mut current: usize = 0;
        match searched {
            Matches::Dir(dirs) => {
                return Self::search_dir(
                    dirs.get(0).unwrap(),
                    selected,
                    &mut current,
                    dirs,
                    config,
                )
                .unwrap();
            }
            Matches::File(file) => {
                return Self::search_file(file, selected, &mut current, config).unwrap();
            }
        }
    }

    fn search_dir(
        dir: &Directory,
        selected: usize,
        current: &mut usize,
        dirs: &Vec<Directory>,
        config: &Config,
    ) -> Option<Self> {
        let children = &dir.children;
        let files = &dir.files;
        if *current == selected {
            return Some(Self::new(dir.path.as_os_str().to_owned(), None));
        }
        *current += 1;
        for child in children {
            if let Some(sel) =
                Self::search_dir(dirs.get(*child).unwrap(), selected, current, dirs, config)
            {
                return Some(sel);
            }
        }
        for file in files {
            if let Some(sel) = Self::search_file(file, selected, current, config) {
                return Some(sel);
            }
        }
        return None;
    }

    fn search_file(
        file: &File,
        selected: usize,
        current: &mut usize,
        config: &Config,
    ) -> Option<Self> {
        if *current == selected {
            return Some(Self::new(file.path.clone().into_os_string(), None));
        }
        *current += 1;
        if !config.just_files {
            for line in file.lines.iter() {
                if *current == selected {
                    return Some(Self::new(file.path.clone().into_os_string(), line.line_num));
                }
                *current += 1;
            }
        }
        None
    }
}
