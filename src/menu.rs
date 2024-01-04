// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::formats;
use crate::match_system::{Directory, File, Matches};
use crate::writer;
use crossterm::style::SetBackgroundColor;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};
use std::ffi::OsString;
use std::io::{self, StdoutLock, Write};
use std::process::Command;

const SCROLL_OFFSET: u16 = 5;
const START_X: u16 = 3;
const START_Y: u16 = 0;

pub struct Menu<'a, 'b> {
    selected_id: usize,
    cursor_y: u16,
    out: &'a mut StdoutLock<'b>,
    searched: Matches,
    lines: Vec<String>,
    num_rows: u16,
    colors: bool,
}

impl<'a, 'b> Menu<'a, 'b> {
    fn new(
        out: &'a mut StdoutLock<'b>,
        searched: Matches,
        config: &Config,
    ) -> io::Result<Menu<'a, 'b>> {
        let mut buffer: Vec<u8> = Vec::new();
        writer::write_results(&mut buffer, &searched, config)?;
        let lines: Vec<String> = buffer
            .split(|&byte| byte == formats::NEW_LINE as u8)
            .map(|v| String::from_utf8_lossy(v).into())
            .collect();

        Ok(Menu {
            selected_id: 0,
            cursor_y: 0,
            out,
            searched,
            lines,
            num_rows: Menu::num_rows(),
            colors: config.colors,
        })
    }

    fn num_rows() -> u16 {
        terminal::size().ok().map(|(_, height)| height).unwrap()
    }

    pub fn draw(out: &'a mut StdoutLock<'b>, matches: Matches, config: &Config) -> io::Result<()> {
        let mut menu: Menu = Menu::new(out, matches, config)?;

        menu.enter()?;
        menu.write_menu()?;

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
                    KeyCode::Char(c) => match c {
                        'j' | 'n' => menu.move_down()?,
                        'k' | 'p' => menu.move_up()?,
                        'q' => break 'outer,
                        'c' => {
                            if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                break 'outer;
                            }
                        }
                        'z' => {
                            if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                menu.suspend()?;
                                menu.resume()?;
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Up => menu.move_up()?,
                    KeyCode::Down => menu.move_down()?,
                    KeyCode::Enter => {
                        let selected =
                            Selected::get_selected_info(menu.selected_id, &menu.searched, config);
                        let line_num: Option<usize> = selected.line;
                        let path = config
                            .path
                            .join(selected.path)
                            .to_string_lossy()
                            .to_string();

                        return menu.exit_and_open(path, line_num);
                    }
                    _ => {}
                }
            } else if let Ok(Event::Resize(_, rows)) = event {
                if menu.num_rows != rows {
                    menu.num_rows = rows;
                    menu.redraw()?;
                }
            }
        }

        menu.leave()
    }

    fn write_menu(&mut self) -> io::Result<()> {
        let mut cursor_y: u16 = START_Y;
        for line in self.lines.iter().take(self.num_rows as usize) {
            queue!(self.out, cursor::MoveTo(START_X, cursor_y), Print(line))?;
            cursor_y += 1;
        }
        self.style_at_cursor(self.cursor_y)?;
        self.out.flush()
    }

    // TODO make this work with keeping the selected id
    fn redraw(&mut self) -> io::Result<()> {
        execute!(self.out, terminal::Clear(ClearType::All))?;
        self.write_menu()?;
        self.selected_id = 0;
        self.cursor_y = START_Y;
        self.style_at_cursor(self.cursor_y)?;
        Ok(())
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
        self.num_rows = Menu::num_rows();
        self.enter()?;
        self.redraw()?;
        Ok(())
    }

    fn move_down(&mut self) -> io::Result<()> {
        if self.selected_id == self.lines.len() - 2 {
            return Ok(());
        }
        self.destyle_at_cursor(self.cursor_y)?;
        self.selected_id += 1;
        self.style_at_cursor(self.cursor_y + 1)?;
        if self.cursor_y + SCROLL_OFFSET != self.num_rows {
            self.cursor_y += 1;
        } else {
            execute!(self.out, terminal::ScrollUp(1))?;
            if (self.selected_id + SCROLL_OFFSET as usize) < self.lines.len() {
                execute!(self.out, cursor::MoveTo(START_X, self.num_rows))?;
                execute!(
                    self.out,
                    Print(
                        self.lines
                            .get(self.selected_id - 1 + SCROLL_OFFSET as usize)
                            .unwrap()
                    )
                )?;
            }
        }
        Ok(())
    }

    fn move_up(&mut self) -> io::Result<()> {
        if self.selected_id == 0 {
            return Ok(());
        }
        self.destyle_at_cursor(self.cursor_y)?;
        self.selected_id -= 1;
        self.style_at_cursor(self.cursor_y - 1)?;
        if self.selected_id < SCROLL_OFFSET as usize || self.cursor_y != SCROLL_OFFSET {
            self.cursor_y -= 1;
        } else {
            execute!(self.out, terminal::ScrollDown(1))?;
            if self.selected_id + 1 > SCROLL_OFFSET as usize {
                execute!(self.out, cursor::MoveTo(START_X, START_Y))?;
                execute!(
                    self.out,
                    Print(
                        self.lines
                            .get(self.selected_id - SCROLL_OFFSET as usize)
                            .unwrap()
                    )
                )?;
            }
        }

        Ok(())
    }

    fn style_at_cursor(&mut self, cursor_y: u16) -> io::Result<()> {
        if self.colors {
            queue!(self.out, SetBackgroundColor(formats::MENU_SELECTED))?;
        }
        queue!(
            self.out,
            cursor::MoveTo(0, cursor_y),
            Print(formats::SELECTED_INDICATOR),
        )?;

        queue!(
            self.out,
            cursor::MoveTo(START_X, cursor_y),
            Print(self.lines.get(self.selected_id).unwrap())
        )?;
        self.out.flush()
    }

    fn destyle_at_cursor(&mut self, cursor_y: u16) -> io::Result<()> {
        execute!(
            self.out,
            cursor::MoveTo(0, cursor_y),
            Print(formats::SELECTED_INDICATOR_CLEAR),
            cursor::MoveTo(START_X, cursor_y),
            Print(self.lines.get(self.selected_id).unwrap())
        )
    }

    fn enter(&mut self) -> io::Result<()> {
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
    fn exit_and_open(&mut self, path: String, _line_num: Option<usize>) -> io::Result<()> {
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg(path)
            .spawn()?;
        self.leave()
    }

    #[cfg(not(windows))]
    fn exit_and_open(&mut self, path: String, line_num: Option<usize>) -> io::Result<()> {
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
                    command.arg(format!("{path}:{l}"));
                } else {
                    command.arg(path);
                }
            }
            "code" => {
                if let Some(l) = line_num {
                    command.arg("--goto");
                    command.arg(format!("{path}:{l}"));
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

struct Selected {
    path: OsString,
    line: Option<usize>,
}

impl Selected {
    pub fn new(path: OsString, line: Option<usize>) -> Selected {
        Selected { path, line }
    }

    fn get_selected_info(selected: usize, searched: &Matches, config: &Config) -> Selected {
        let mut current: usize = 0;
        match searched {
            Matches::Dir(dirs) => {
                return Selected::search_dir(
                    dirs.get(0).unwrap(),
                    selected,
                    &mut current,
                    dirs,
                    config,
                )
                .unwrap();
            }
            Matches::File(file) => {
                return Selected::search_file(file, selected, &mut current, config).unwrap();
            }
        }
    }

    fn search_dir(
        dir: &Directory,
        selected: usize,
        current: &mut usize,
        dirs: &Vec<Directory>,
        config: &Config,
    ) -> Option<Selected> {
        let children = &dir.children;
        let files = &dir.files;
        let mut sel: Option<Selected>;
        if *current == selected {
            return Some(Selected::new(dir.path.as_os_str().to_owned(), None));
        }
        *current += 1;
        for child in children {
            sel = Selected::search_dir(dirs.get(*child).unwrap(), selected, current, dirs, config);
            if sel.is_some() {
                return sel;
            }
        }
        for file in files {
            sel = Selected::search_file(file, selected, current, config);
            if sel.is_some() {
                return sel;
            }
        }
        return None;
    }

    fn search_file(
        file: &File,
        selected: usize,
        current: &mut usize,
        config: &Config,
    ) -> Option<Selected> {
        if *current == selected {
            return Some(Selected::new(file.path.clone().into_os_string(), None));
        }
        *current += 1;
        if !config.just_files {
            for line in file.lines.iter() {
                if *current == selected {
                    return Some(Selected::new(
                        file.path.clone().into_os_string(),
                        line.line_num,
                    ));
                }
                *current += 1;
            }
        }
        None
    }
}
