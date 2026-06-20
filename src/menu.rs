// SPDX-License-Identifier: MIT

use crate::{
    args::{self, OpenStrategy},
    config::{self, Config, base_config},
    errors::SUBMIT_ISSUE,
    match_system::Matches,
    matcher, style,
    term::Term,
    writer::{Entry, WithFilter, matches_to_display_lines},
};
use crossbeam_channel::{Receiver, Sender};

type SearchResult = (Matches, Arc<Config>);
use crossterm::{
    cursor, event,
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind},
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::{
    collections::HashSet,
    ffi::OsString,
    io::{self, Write},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const TOP_ROW: u16 = 0;
const DOUBLE_CLICK_MS: Duration = Duration::from_millis(500);

#[derive(PartialEq)]
enum Mode {
    Search,
    Navigate,
    Filter,
    Help,
}

enum Loop {
    Continue,
    Break,
    JumpCycle,
    OpenPath(OsString, Option<usize>),
}

#[derive(Clone, Copy)]
enum ViewAnchor {
    Middle,
    Top,
    Bottom,
}

impl ViewAnchor {
    fn next(&mut self) {
        *self = match *self {
            Self::Middle => Self::Top,
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Middle,
        };
    }
}

struct DoubleClick {
    down_row: u16,
    last_click: Option<(Instant, u16)>,
}

impl DoubleClick {
    fn new() -> Self {
        DoubleClick {
            down_row: 0,
            last_click: None,
        }
    }
    fn down(&mut self, row: u16) {
        self.down_row = row;
    }
    fn up(&mut self, row: u16) -> Option<bool> {
        if row != self.down_row {
            return None;
        }
        let is_double = self
            .last_click
            .map(|(t, r)| r == row && t.elapsed() < DOUBLE_CLICK_MS)
            .unwrap_or(false);
        self.last_click = if is_double {
            None
        } else {
            Some((Instant::now(), row))
        };
        Some(is_double)
    }
}

struct Window {
    first: isize,
    last: isize,
}

impl Window {
    fn new() -> Self {
        Window { first: 0, last: 0 }
    }
    fn shift_up(&mut self) {
        self.first -= 1;
        self.last -= 1;
    }
    fn shift_down(&mut self) {
        self.first += 1;
        self.last += 1;
    }
    fn set(&mut self, first: isize, last: isize) {
        self.first = first;
        self.last = last;
    }
}

fn fold_end<'e>(orig: usize, lines: &[Box<dyn Entry + 'e>]) -> usize {
    let depth = lines[orig].depth();
    let mut end = orig + 1;
    while end < lines.len() && lines[end].depth() > depth {
        end += 1;
    }
    end
}

fn str_move_back(s: &str, cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }
    let mut i = cursor - 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn str_move_forward(s: &str, cursor: usize) -> usize {
    if cursor < s.len() {
        cursor + s[cursor..].chars().next().unwrap().len_utf8()
    } else {
        cursor
    }
}

fn str_backspace(s: &mut String, cursor: &mut usize) {
    let new = str_move_back(s, *cursor);
    if new < *cursor {
        s.remove(new);
        *cursor = new;
    }
}

fn str_kill_word_backward(s: &mut String, cursor: &mut usize) {
    while *cursor > 0 && s.as_bytes().get(*cursor - 1) == Some(&b' ') {
        str_backspace(s, cursor);
    }
    while *cursor > 0 && s.as_bytes().get(*cursor - 1) != Some(&b' ') {
        str_backspace(s, cursor);
    }
}

fn str_kill_word_forward(s: &mut String, cursor: usize) {
    while cursor < s.len() && s.as_bytes()[cursor] != b' ' {
        s.remove(cursor);
    }
}

fn str_move_word_forward(s: &str, cursor: usize) -> usize {
    let mut i = cursor;
    while i < s.len() && s.as_bytes()[i] != b' ' {
        i = str_move_forward(s, i);
    }
    while i < s.len() && s.as_bytes()[i] == b' ' {
        i = str_move_forward(s, i);
    }
    i
}

fn str_move_word_back(s: &str, cursor: usize) -> usize {
    let mut i = cursor;
    while i > 0 && s.as_bytes()[i - 1] == b' ' {
        i = str_move_back(s, i);
    }
    while i > 0 && s.as_bytes()[i - 1] != b' ' {
        i = str_move_back(s, i);
    }
    i
}

fn str_transpose_chars(s: &mut String, cursor: &mut usize) -> bool {
    let at = if *cursor == s.len() {
        str_move_back(s, *cursor)
    } else {
        *cursor
    };
    let prev = str_move_back(s, at);
    if prev < at && at < s.len() {
        let next = str_move_forward(s, at);
        let a = s[prev..at].to_string();
        let b = s[at..next].to_string();
        s.replace_range(prev..next, &format!("{}{}", b, a));
        *cursor = prev + b.len() + a.len();
        true
    } else {
        false
    }
}

fn str_transpose_words(s: &mut String, cursor: usize) -> Option<usize> {
    let mut w2_end = cursor;
    while w2_end < s.len() && s.as_bytes()[w2_end] == b' ' {
        w2_end = str_move_forward(s, w2_end);
    }
    while w2_end < s.len() && s.as_bytes()[w2_end] != b' ' {
        w2_end = str_move_forward(s, w2_end);
    }
    let mut w2_start = w2_end;
    while w2_start > 0 && s.as_bytes()[w2_start - 1] != b' ' {
        w2_start = str_move_back(s, w2_start);
    }
    if w2_start == w2_end {
        return None;
    }
    let mut w1_end = w2_start;
    while w1_end > 0 && s.as_bytes()[w1_end - 1] == b' ' {
        w1_end = str_move_back(s, w1_end);
    }
    let mut w1_start = w1_end;
    while w1_start > 0 && s.as_bytes()[w1_start - 1] != b' ' {
        w1_start = str_move_back(s, w1_start);
    }
    if w1_start == w1_end {
        return None;
    }
    let w1 = s[w1_start..w1_end].to_string();
    let w2 = s[w2_start..w2_end].to_string();
    s.replace_range(w2_start..w2_end, &w1);
    s.replace_range(w1_start..w1_end, &w2);
    let gap = w2_start - w1_end;
    Some(w1_start + w2.len() + gap + w1.len())
}

fn bar_display(prompt: &str, text: &str, cursor_byte: usize, width: usize) -> (String, u16) {
    let prompt_chars = prompt.chars().count();
    let cursor_chars = text[..cursor_byte].chars().count();
    let cursor_col = prompt_chars + cursor_chars;
    let scroll_chars = if cursor_col >= width {
        cursor_col + 1 - width
    } else {
        0
    };
    let visible_chars = width.saturating_sub(prompt_chars);
    let text_visible: String = text
        .chars()
        .skip(scroll_chars)
        .take(visible_chars)
        .collect();
    (
        format!("{}{}", prompt, text_visible),
        (cursor_col - scroll_chars) as u16,
    )
}

fn edit_bar(
    s: &mut String,
    cursor: &mut usize,
    code: KeyCode,
    ctrl: bool,
    alt: bool,
) -> Option<bool> {
    if alt {
        match code {
            KeyCode::Char('f') => {
                *cursor = str_move_word_forward(s, *cursor);
                Some(false)
            }
            KeyCode::Char('b') => {
                *cursor = str_move_word_back(s, *cursor);
                Some(false)
            }
            KeyCode::Char('d') => {
                str_kill_word_forward(s, *cursor);
                Some(true)
            }
            KeyCode::Backspace => {
                str_kill_word_backward(s, cursor);
                Some(true)
            }
            KeyCode::Char('t') => str_transpose_words(s, *cursor).map(|new| {
                *cursor = new;
                true
            }),
            _ => None,
        }
    } else if ctrl {
        match code {
            KeyCode::Char('a') => {
                *cursor = 0;
                Some(false)
            }
            KeyCode::Char('e') => {
                *cursor = s.len();
                Some(false)
            }
            KeyCode::Char('f') => {
                *cursor = str_move_forward(s, *cursor);
                Some(false)
            }
            KeyCode::Char('b') => {
                *cursor = str_move_back(s, *cursor);
                Some(false)
            }
            KeyCode::Char('u') if !s.is_empty() => {
                s.clear();
                *cursor = 0;
                Some(true)
            }
            KeyCode::Char('k') if *cursor < s.len() => {
                s.truncate(*cursor);
                Some(true)
            }
            KeyCode::Char('w') if !s.is_empty() => {
                str_kill_word_backward(s, cursor);
                Some(true)
            }
            KeyCode::Char('d') if *cursor < s.len() => {
                s.remove(*cursor);
                Some(true)
            }
            KeyCode::Char('t') => str_transpose_chars(s, cursor).then_some(true),
            _ => None,
        }
    } else {
        match code {
            KeyCode::Char(c) => {
                s.insert(*cursor, c);
                *cursor += c.len_utf8();
                Some(true)
            }
            KeyCode::Backspace if *cursor > 0 => {
                str_backspace(s, cursor);
                Some(true)
            }
            KeyCode::Delete if *cursor < s.len() => {
                s.remove(*cursor);
                Some(true)
            }
            KeyCode::Left => {
                *cursor = str_move_back(s, *cursor);
                Some(false)
            }
            KeyCode::Right => {
                *cursor = str_move_forward(s, *cursor);
                Some(false)
            }
            KeyCode::Home => {
                *cursor = 0;
                Some(false)
            }
            KeyCode::End => {
                *cursor = s.len();
                Some(false)
            }
            _ => None,
        }
    }
}

fn draw_popup(term: &mut Term, content: &str) -> io::Result<()> {
    let cfg = config::base_config();
    let lines: Vec<&str> = content.lines().collect();
    let cw = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
    let height = lines.len() as u16 + 2;
    let x = term.width().saturating_sub(cw) / 2;
    let y = term.height.saturating_sub(height) / 2;
    queue!(
        term,
        cursor::MoveTo(x, y),
        Print(format!(
            "{}{}{}",
            cfg.chars.tl,
            style::repeat(cfg.chars.h, cw as usize),
            cfg.chars.tr
        ))
    )?;
    for (i, line) in lines.iter().enumerate() {
        queue!(
            term,
            cursor::MoveTo(x, y + i as u16 + 1),
            Print(format!(
                "{}{:w$}{}",
                cfg.chars.v,
                line,
                cfg.chars.v,
                w = cw as usize
            ))
        )?;
    }
    queue!(
        term,
        cursor::MoveTo(x, y + height - 1),
        Print(format!(
            "{}{}{}",
            cfg.chars.bl,
            style::repeat(cfg.chars.h, cw as usize),
            cfg.chars.br
        ))
    )?;
    term.flush()
}

fn format_keys(keys: &[KeyCode]) -> String {
    keys.iter()
        .map(|&k| args::key_display(k))
        .collect::<Vec<_>>()
        .join(",")
}

fn help_table(rows: &[(String, &str)], indent: usize, ncols: usize) -> String {
    let key_w = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let desc_w = rows.iter().map(|(_, d)| d.len()).max().unwrap_or(0);
    let pad = " ".repeat(indent);
    rows.chunks(ncols)
        .map(|chunk| {
            let mut line = pad.clone();
            for (j, (k, d)) in chunk.iter().enumerate() {
                if j > 0 {
                    line.push_str("   ");
                }
                if j + 1 < chunk.len() {
                    line.push_str(&format!("{k:<key_w$}  {d:<desc_w$}"));
                } else {
                    line.push_str(&format!("{k:<key_w$}  {d}"));
                }
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn help_popup() -> String {
    let cfg = config::base_config();
    let k = &cfg.keys;
    let sp = &cfg.chars.search_prompt;
    let np = &cfg.chars.search_prompt_inactive;
    let fp = &cfg.chars.filter_prompt;

    let bar_rows: Vec<(String, &str)> = vec![
        ("ctrl a/e".into(), "begin/end"),
        ("ctrl f/b".into(), "char forward/back"),
        ("ctrl u".into(), "clear"),
        ("ctrl k".into(), "kill to end"),
        ("ctrl d".into(), "delete char"),
        ("ctrl t".into(), "transpose chars"),
        ("alt f/b".into(), "word forward/back"),
        ("alt d".into(), "kill word forward"),
        ("alt bksp".into(), "kill word back"),
        ("alt t".into(), "transpose words"),
    ];

    let nav_rows: Vec<(String, &str)> = vec![
        (
            format!("{}/backtab/opt+q", format_keys(&k.search)),
            "search",
        ),
        (format_keys(&k.filter), "filter"),
        (format_keys(&k.up), "up"),
        (format_keys(&k.down), "down"),
        (format_keys(&k.big_up), "big jump up"),
        (format_keys(&k.big_down), "big jump down"),
        (format_keys(&k.up_path), "prev path"),
        (format_keys(&k.down_path), "next path"),
        (format_keys(&k.up_same_depth), "prev same depth"),
        (format_keys(&k.down_same_depth), "next same depth"),
        (format_keys(&k.top), "top"),
        (format_keys(&k.bottom), "bottom"),
        (format_keys(&k.page_up), "page up"),
        (format_keys(&k.page_down), "page down"),
        (format_keys(&k.cycle_view), "cycle cursor position"),
        (format_keys(&k.fold), "fold/unfold"),
        ("scroll/click".into(), "scroll/click"),
        (format_keys(&k.open), "open"),
        (format_keys(&k.help), "help"),
        (format!("esc/{}", format_keys(&k.quit)), "quit"),
    ];

    format!(
        "text bars\n\
         {}\n\
         modes\n\
         \x20- search ({sp})  {}/backtab → navigate, esc → navigate\n\
         \x20- filter ({fp})  enter/backtab → navigate, esc → clear and navigate\n\
         \x20- navigate ({np})\n\
         {}\n\
         press {} to close",
        help_table(&bar_rows, 2, 2),
        format_keys(&k.submit_search),
        help_table(&nav_rows, 4, 2),
        format_keys(&k.quit),
    )
}

struct CurrentResults {
    lines: Vec<Box<dyn Entry>>,
}

impl CurrentResults {
    fn new(matches: Matches, config: Arc<Config>) -> io::Result<Self> {
        let lines =
            matches_to_display_lines(&matches, config).map_err(|e| io::Error::other(e.mes))?;
        Ok(CurrentResults { lines })
    }
}

pub struct Menu<'a, 'b> {
    in_menu: bool,
    needs_search: bool,
    search: String,
    filter: String,
    search_cursor: usize,
    term: &'a mut Term<'b>,
    current: Option<Box<CurrentResults>>,
    mode: Mode,
    error_msg: Option<String>,
    search_gen: u64,
    search_started: Option<Instant>,
    abort: Arc<AtomicBool>,
    result_rx: Receiver<(u64, Result<Option<SearchResult>, String>)>,
    result_tx: Sender<(u64, Result<Option<SearchResult>, String>)>,
    selected_id: usize,
    cursor_y: u16,
    max: usize,
    jump: ViewAnchor,
    folded: HashSet<usize>,
    visible: Vec<usize>,
    big_jump: u16,
    filter_cursor: usize,
    bars_below: u16,
    term_height: u16,
    window: Window,
    pending_save: bool,
    debounce_deadline: Option<Instant>,
}

impl<'a, 'b> Menu<'a, 'b> {
    fn new(term: &'a mut Term<'b>) -> Self {
        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        let term_height = term.height;
        let in_menu = base_config().menu;
        Menu {
            in_menu,
            needs_search: false,
            search: String::new(),
            search_cursor: 0,
            term,
            current: None,
            mode: if in_menu {
                Mode::Search
            } else {
                Mode::Navigate
            },
            error_msg: None,
            search_gen: 0,
            search_started: None,
            abort: Arc::new(AtomicBool::new(false)),
            result_rx,
            result_tx,
            selected_id: 0,
            cursor_y: in_menu as u16,
            max: 0,
            jump: ViewAnchor::Middle,
            folded: HashSet::new(),
            visible: Vec::new(),
            big_jump: (term_height / 5).max(1),
            filter: String::new(),
            filter_cursor: 0,
            bars_below: 0,
            term_height,
            window: Window::new(),
            pending_save: false,
            debounce_deadline: None,
        }
    }

    fn set_dims(&mut self, h: u16) {
        self.term_height = h;
        self.big_jump = (h / 5).max(1);
    }

    fn start_y(&self) -> u16 {
        self.in_menu as u16
    }

    fn content_height(&self) -> u16 {
        self.term_height
            .saturating_sub(self.start_y())
            .saturating_sub(self.bars_below)
    }

    fn max_cursor_y(&self) -> u16 {
        self.start_y() + self.content_height().saturating_sub(1)
    }

    fn start_x() -> u16 {
        config::base_config().chars.selected_indicator_clear.len() as u16
    }

    fn lines(&self) -> &[Box<dyn Entry + 'static>] {
        self.current.as_ref().map_or(&[], |c| &c.lines)
    }

    fn open_result(
        &mut self,
        path: OsString,
        line: Option<usize>,
    ) -> io::Result<Option<(OsString, Option<usize>)>> {
        self.term.give()?;
        Ok(Some((path, line)))
    }

    fn update_visible(&mut self) {
        let lower = self.filter.to_lowercase();
        let lines = self
            .current
            .as_ref()
            .map(|c| c.lines.as_slice())
            .unwrap_or(&[]);
        let include = if lower.is_empty() {
            None
        } else {
            let n = lines.len();
            let mut m = vec![false; n];
            for (i, b) in m.iter_mut().enumerate() {
                if lines[i].filter_text().to_lowercase().contains(&lower) {
                    *b = true;
                }
            }
            for i in (0..n).rev() {
                if lines[i].is_path() && !m[i] {
                    let end = fold_end(i, lines);
                    if (i + 1..end).any(|j| m[j]) {
                        m[i] = true;
                    }
                }
            }
            Some(m)
        };
        let mut visible = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            if include.as_ref().is_none_or(|inc| inc[i]) {
                visible.push(i);
            }
            i = if self.folded.contains(&i) {
                fold_end(i, lines)
            } else {
                i + 1
            };
        }
        self.visible = visible;
        self.max = self.visible.len().saturating_sub(1);
        if !self.visible.is_empty() {
            self.selected_id = self.selected_id.min(self.max);
            let max_cy = self.max_cursor_y();
            if self.cursor_y > max_cy {
                self.cursor_y = max_cy;
            }
        }
    }

    fn toggle_fold(&mut self) -> bool {
        if self.visible.is_empty() {
            return false;
        }
        let orig = self.visible[self.selected_id];
        if !self.lines().get(orig).is_some_and(|l| l.is_path()) {
            return false;
        }
        if !self.folded.remove(&orig) {
            self.folded.insert(orig);
        }
        self.update_visible();
        true
    }

    fn print_line(&mut self, orig: usize) -> io::Result<()> {
        let lines = self
            .current
            .as_ref()
            .map(|c| c.lines.as_slice())
            .unwrap_or(&[]);
        let entry = &*lines[orig];
        let filter = self.filter.as_str();
        queue!(self.term, Print(WithFilter { entry, filter }))?;
        if self.folded.contains(&orig) && fold_end(orig, lines) > orig + 1 {
            let cfg = config::base_config();
            queue!(self.term, Print(&cfg.chars.ellipsis))?;
        }
        Ok(())
    }

    fn style_selected(&mut self) -> io::Result<()> {
        if self.visible.is_empty() {
            return Ok(());
        }
        let cfg = config::base_config();
        queue!(self.term, cursor::MoveTo(0, self.cursor_y))?;
        if cfg.with_colors {
            queue!(self.term, SetBackgroundColor(cfg.colors.selected_bg))?;
        }
        if cfg.with_colors
            && let Some(c) = cfg.colors.selected_indicator
        {
            queue!(
                self.term,
                Print(style::style_with(
                    cfg.chars.selected_indicator.as_str(),
                    c,
                    &cfg,
                )),
                SetBackgroundColor(cfg.colors.selected_bg)
            )?;
        } else {
            queue!(self.term, Print(cfg.chars.selected_indicator.as_str()))?;
        }
        queue!(self.term, cursor::MoveTo(Self::start_x(), self.cursor_y))?;
        let orig = self.visible[self.selected_id];
        self.print_line(orig)
    }

    fn top_visible(&self) -> bool {
        self.window.first <= 0
    }

    fn bot_visible(&self) -> bool {
        self.window.last >= self.max as isize
    }

    fn destyle_selected(&mut self) -> io::Result<()> {
        let cfg = config::base_config();
        queue!(
            self.term,
            ResetColor,
            cursor::MoveTo(0, self.cursor_y),
            Print(cfg.chars.selected_indicator_clear.as_str()),
            cursor::MoveTo(Self::start_x(), self.cursor_y),
        )?;
        let orig = self.visible[self.selected_id];
        self.print_line(orig)
    }

    fn scroll_fill(&mut self, scrolling_down: bool) -> io::Result<()> {
        if scrolling_down {
            for i in 0..self.bars_below {
                queue!(
                    self.term,
                    cursor::MoveTo(0, self.term_height - 1 - i),
                    terminal::Clear(ClearType::CurrentLine),
                )?;
            }
            self.window.shift_down();
            queue!(self.term, terminal::ScrollUp(1))?;
        } else {
            if self.start_y() > 0 {
                queue!(
                    self.term,
                    cursor::MoveTo(0, 0),
                    terminal::Clear(ClearType::CurrentLine),
                )?;
            }
            self.window.shift_up();
            queue!(self.term, terminal::ScrollDown(1))?;
        }
        let fill_y = if scrolling_down {
            self.max_cursor_y()
        } else {
            self.start_y()
        };
        let line_id = if scrolling_down {
            self.window.last
        } else {
            self.window.first
        };
        let orig = (line_id >= 0)
            .then(|| self.visible.get(line_id as usize).copied())
            .flatten();
        if let Some(orig) = orig {
            queue!(self.term, cursor::MoveTo(Self::start_x(), fill_y))?;
            self.print_line(orig)?;
        }
        Ok(())
    }

    fn draw_content(&mut self) -> io::Result<()> {
        if self.visible.is_empty() {
            return Ok(());
        }
        let ch = self.content_height();
        let rel = self.cursor_y - self.start_y();
        let first = self.selected_id as isize - rel as isize;
        let above = (rel as usize).min(self.selected_id);
        let take = (above + (ch - rel) as usize).min(self.max + 1);
        let skip = first.max(0) as usize;
        let actual_take = take.min(self.visible.len().saturating_sub(skip));
        let start_row = self.start_y() + rel - above as u16;
        self.window
            .set(first, (skip + actual_take).saturating_sub(1) as isize);
        for y in self.start_y()..start_row {
            queue!(
                self.term,
                cursor::MoveTo(0, y),
                terminal::Clear(ClearType::CurrentLine)
            )?;
        }
        for i in 0..actual_take {
            let orig = self.visible[skip + i];
            let row = start_row + i as u16;
            queue!(
                self.term,
                cursor::MoveTo(0, row),
                terminal::Clear(ClearType::CurrentLine),
                cursor::MoveTo(Self::start_x(), row)
            )?;
            self.print_line(orig)?;
        }
        for y in (start_row + actual_take as u16)..(self.start_y() + ch) {
            queue!(
                self.term,
                cursor::MoveTo(0, y),
                terminal::Clear(ClearType::CurrentLine)
            )?;
        }
        self.style_selected()
    }

    fn down_clamp(&mut self, try_dist: u16) -> bool {
        if self.visible.is_empty() {
            return false;
        }
        let dist = (try_dist as usize).min(self.max - self.selected_id);
        if dist == 0 {
            return false;
        }
        self.selected_id += dist;
        if self.selected_id as isize <= self.window.last {
            self.cursor_y = (self.cursor_y + dist as u16).min(self.max_cursor_y());
        }
        true
    }

    fn up_clamp(&mut self, try_dist: u16) -> bool {
        if self.visible.is_empty() {
            return false;
        }
        let dist = (try_dist as usize).min(self.selected_id);
        if dist == 0 {
            return false;
        }
        self.selected_id -= dist;
        if (self.selected_id as isize) >= self.window.first.max(0) {
            self.cursor_y = self
                .cursor_y
                .saturating_sub(dist as u16)
                .max(self.start_y());
        }
        true
    }

    fn top(&mut self) -> bool {
        if self.visible.is_empty() || self.selected_id == 0 {
            return false;
        }
        self.cursor_y = if self.top_visible() {
            (self.cursor_y as isize - self.selected_id as isize) as u16
        } else {
            self.start_y()
        };
        self.selected_id = 0;
        true
    }

    fn bottom(&mut self) -> bool {
        if self.visible.is_empty() || self.selected_id == self.max {
            return false;
        }
        self.cursor_y = if self.bot_visible() {
            (self.cursor_y as isize + (self.max - self.selected_id) as isize) as u16
        } else {
            self.max_cursor_y()
        };
        self.selected_id = self.max;
        true
    }

    fn down_page(&mut self) -> bool {
        let dist = (self.content_height() as usize).min(self.max.saturating_sub(self.selected_id));
        if dist == 0 {
            return false;
        }
        self.down_clamp(dist as u16)
    }

    fn up_page(&mut self) -> bool {
        let dist = (self.content_height() as usize).min(self.selected_id);
        if dist == 0 {
            return false;
        }
        self.up_clamp(dist as u16)
    }

    fn jump_cursor(&mut self, loc: ViewAnchor) -> bool {
        let y = match loc {
            ViewAnchor::Middle => self.start_y() + self.content_height() / 2,
            ViewAnchor::Top => self.start_y(),
            ViewAnchor::Bottom => self.max_cursor_y(),
        };
        if self.cursor_y != y {
            self.cursor_y = y;
            true
        } else {
            false
        }
    }

    fn down_path(&mut self) -> bool {
        if self.visible.is_empty() {
            return false;
        }
        let lines = self.lines();
        let d = self.visible[self.selected_id + 1..]
            .iter()
            .position(|&o| lines[o].is_path());
        match d {
            Some(d) => self.down_clamp(d as u16 + 1),
            None => false,
        }
    }

    fn up_path(&mut self) -> bool {
        if self.visible.is_empty() {
            return false;
        }
        let lines = self.lines();
        let i = self.visible[..self.selected_id]
            .iter()
            .rposition(|&o| lines[o].is_path());
        match i {
            Some(i) => self.up_clamp((self.selected_id - i) as u16),
            None => false,
        }
    }

    fn down_path_same_depth(&mut self) -> bool {
        if self.visible.is_empty() {
            return false;
        }
        let lines = self.lines();
        let orig = self.visible[self.selected_id];
        if !lines[orig].is_path() {
            return self.down_path();
        }
        let depth = lines[orig].depth();
        let d = self.visible[self.selected_id + 1..]
            .iter()
            .position(|&o| lines[o].is_path() && lines[o].depth() == depth);
        match d {
            Some(d) => self.down_clamp(d as u16 + 1),
            None => false,
        }
    }

    fn up_path_same_depth(&mut self) -> bool {
        if self.visible.is_empty() {
            return false;
        }
        let lines = self.lines();
        let orig = self.visible[self.selected_id];
        if !lines[orig].is_path() {
            return self.up_path();
        }
        let depth = lines[orig].depth();
        let i = self.visible[..self.selected_id]
            .iter()
            .rposition(|&o| lines[o].is_path() && lines[o].depth() == depth);
        match i {
            Some(i) => self.up_clamp((self.selected_id - i) as u16),
            None => false,
        }
    }

    fn click_on(&mut self, row: u16) -> bool {
        if row < self.start_y() || self.visible.is_empty() {
            return false;
        }
        let first = self.selected_id as isize - (self.cursor_y - self.start_y()) as isize;
        let row_idx = first + (row - self.start_y()) as isize;
        if row_idx < 0 || row_idx as usize >= self.visible.len() {
            return false;
        }
        let new_id = row_idx as usize;
        if new_id == self.selected_id {
            return false;
        }
        self.selected_id = new_id;
        self.cursor_y = row;
        true
    }

    fn needs_bars(&self) -> u16 {
        if self.mode == Mode::Filter || !self.filter.is_empty() {
            1
        } else {
            0
        }
    }

    fn trigger_search(&mut self) {
        self.needs_search = false;
        self.search_gen += 1;
        let search_id = self.search_gen;
        self.abort.store(true, Ordering::Relaxed);
        self.abort = Arc::new(AtomicBool::new(false));
        if self.search.is_empty() {
            self.error_msg = None;
            self.apply_results(None);
            return;
        }
        match config::parse_menu_query(&self.search) {
            Ok(new_config) => {
                if new_config.search.regexps.is_empty() && !new_config.search.files {
                    self.error_msg = None;
                    self.apply_results(None);
                    return;
                }
                self.error_msg = None;
                let new_config = Arc::new(new_config);
                let thread_config = Arc::clone(&new_config);
                self.search_started = Some(Instant::now());
                let abort = Arc::clone(&self.abort);
                let tx = self.result_tx.clone();
                std::thread::spawn(move || {
                    let result = matcher::search(abort, thread_config.clone())
                        .map(|opt| opt.map(|m| (m, thread_config)))
                        .map_err(|e| e.mes);
                    let _ = tx.send((search_id, result));
                });
            }
            Err(msg) => {
                self.error_msg = Some(msg.mes);
                self.apply_results(None);
            }
        }
    }

    fn apply_results(&mut self, result: Option<SearchResult>) {
        self.search_started = None;
        self.current = None;
        if let Some((m, config)) = result
            && let Ok(r) = CurrentResults::new(m, config)
        {
            self.current = Some(Box::new(r));
        }
        if self.pending_save {
            self.pending_save = false;
            if self.current.is_some() {
                self.save_query_for_repeat();
            }
        }
        self.selected_id = 0;
        self.cursor_y = self.start_y();
        self.folded.clear();
        self.update_visible();
    }

    fn save_query_for_repeat(&self) {
        let cfg = config::base_config();
        let Some(file) = &cfg.repeat_file else {
            return;
        };
        if self.search.is_empty() {
            let _ = config::save_search_params(file, &cfg);
        } else if let Ok(merged) = config::parse_menu_query(&self.search) {
            let _ = config::save_search_params(file, &merged);
        }
    }

    fn queue_query_bar(&mut self) -> io::Result<()> {
        let cfg = config::base_config();
        let width = self.term.width() as usize;
        let prompt = match self.mode {
            Mode::Search => cfg.chars.search_prompt.as_str(),
            _ => cfg.chars.search_prompt_inactive.as_str(),
        };
        let (top, _) = bar_display(prompt, &self.search, self.search_cursor, width);
        queue!(
            self.term,
            ResetColor,
            cursor::MoveTo(0, TOP_ROW),
            terminal::Clear(ClearType::CurrentLine),
            Print(top),
        )
    }

    fn queue_filter_bar(&mut self) -> io::Result<()> {
        if self.mode != Mode::Filter && self.filter.is_empty() {
            return Ok(());
        }
        let cfg = config::base_config();
        let y = self.term.height - 1;
        let width = self.term.width() as usize;
        let (display, _) = bar_display(
            &cfg.chars.filter_prompt,
            &self.filter,
            self.filter_cursor,
            width,
        );
        queue!(
            self.term,
            ResetColor,
            cursor::MoveTo(0, y),
            terminal::Clear(ClearType::CurrentLine),
            Print(display),
        )
    }

    fn position_cursor(&mut self) -> io::Result<()> {
        let width = self.term.width() as usize;
        match self.mode {
            Mode::Search => {
                let (_, cx) = bar_display(
                    &config::base_config().chars.search_prompt,
                    &self.search,
                    self.search_cursor,
                    width,
                );
                queue!(self.term, cursor::Show, cursor::MoveTo(cx, TOP_ROW))
            }
            Mode::Filter => {
                let (_, cx) = bar_display(
                    &config::base_config().chars.filter_prompt,
                    &self.filter,
                    self.filter_cursor,
                    width,
                );
                let y = self.term.height - 1;
                queue!(self.term, cursor::Show, cursor::MoveTo(cx, y))
            }
            _ => queue!(self.term, cursor::Hide),
        }
    }

    fn sync_start_y(&mut self) -> bool {
        if self.mode == Mode::Search && !self.in_menu {
            self.in_menu = true;
            self.cursor_y = (self.cursor_y + 1).min(self.max_cursor_y());
            true
        } else {
            false
        }
    }

    fn enter_search(&mut self) -> io::Result<()> {
        self.mode = Mode::Search;
        if self.sync_start_y() {
            self.draw_results()?;
        }
        self.draw_query()?;
        self.draw_filter()
    }

    fn enter_filter(&mut self) -> io::Result<()> {
        self.mode = Mode::Filter;
        self.filter_cursor = self.filter.len();
        self.bars_below = self.needs_bars();
        self.draw_query()?;
        self.draw_filter()
    }

    fn draw_query(&mut self) -> io::Result<()> {
        if self.in_menu {
            self.queue_query_bar()?;
        }
        self.position_cursor()?;
        self.term.flush()
    }

    fn draw_filter(&mut self) -> io::Result<()> {
        self.queue_filter_bar()?;
        self.position_cursor()?;
        self.term.flush()
    }

    fn draw_results(&mut self) -> io::Result<()> {
        self.sync_start_y();
        self.bars_below = self.needs_bars();
        self.set_dims(self.term.height);
        let start_y = self.start_y();
        queue!(
            self.term,
            cursor::MoveTo(0, start_y),
            terminal::Clear(ClearType::FromCursorDown)
        )?;
        if self.bars_below > 0 {
            self.queue_filter_bar()?;
        }

        let cfg = config::base_config();
        if self.visible.is_empty() {
            if let Some(err) = self.error_msg.clone() {
                if cfg.with_colors {
                    queue!(self.term, SetForegroundColor(Color::Red))?;
                }
                let start_y = self.start_y();
                for (i, line) in err.lines().enumerate().take(self.content_height() as usize) {
                    queue!(
                        self.term,
                        cursor::MoveTo(0, start_y + i as u16),
                        Print(line)
                    )?;
                }
                if cfg.with_colors {
                    queue!(self.term, ResetColor)?;
                }
            } else {
                let y = self.start_y() + self.content_height() / 2;
                let submit_hint;
                let msg = if self.in_menu && self.search.is_empty() {
                    "type to search"
                } else if self.search_started.is_some() {
                    "searching..."
                } else if self.needs_search {
                    submit_hint =
                        format!("press {} to search", format_keys(&cfg.keys.submit_search));
                    &submit_hint
                } else {
                    "no results"
                };
                queue!(self.term, cursor::MoveTo(0, y), Print(msg))?;
            }
        } else {
            self.draw_content()?;
        }
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            let (width, height) = terminal::size()?;
            self.term.set_dims(height, width);
            self.set_dims(height);
            self.term.claim()?;
            self.draw_results()?;
            self.draw_query()?;
            self.draw_filter()?;
        }
        Ok(())
    }

    fn step_down(&mut self, scroll: bool) -> io::Result<()> {
        if self.visible.is_empty() || self.selected_id >= self.max {
            return Ok(());
        }
        self.destyle_selected()?;
        self.selected_id += 1;
        let do_scroll = if scroll {
            self.cursor_y >= self.big_jump && !self.bot_visible()
        } else {
            self.cursor_y + self.big_jump >= self.max_cursor_y() && !self.bot_visible()
        };
        if do_scroll {
            self.scroll_fill(true)?;
        } else {
            self.cursor_y = (self.cursor_y + 1).min(self.max_cursor_y());
        }
        self.style_selected()?;
        if self.in_menu {
            self.draw_query()?;
        }
        if self.bars_below > 0 {
            self.draw_filter()?;
        } else if !self.in_menu {
            self.term.flush()?;
        }
        Ok(())
    }

    fn step_up(&mut self, scroll: bool) -> io::Result<()> {
        if self.visible.is_empty() || self.selected_id == 0 {
            return Ok(());
        }
        self.destyle_selected()?;
        self.selected_id -= 1;
        let do_scroll = if scroll {
            self.cursor_y + self.big_jump <= self.max_cursor_y() && !self.top_visible()
        } else {
            self.cursor_y <= self.start_y() + self.big_jump && !self.top_visible()
        };
        if do_scroll {
            self.scroll_fill(false)?;
        } else {
            self.cursor_y = self.cursor_y.saturating_sub(1).max(self.start_y());
        }
        self.style_selected()?;
        if self.in_menu {
            self.draw_query()?;
        }
        if self.bars_below > 0 {
            self.draw_filter()?;
        } else if !self.in_menu {
            self.term.flush()?;
        }
        Ok(())
    }

    fn handle_open(&mut self) -> io::Result<Loop> {
        if self.visible.is_empty() {
            return Ok(Loop::Continue);
        }
        let orig = self.visible[self.selected_id];
        let info = match self.current.as_ref().map(|c| c.lines[orig].open_info()) {
            Some(Ok(info)) => info,
            Some(Err(mes)) => {
                draw_popup(self.term, &mes.mes)?;
                self.mode = Mode::Help;
                return Ok(Loop::Continue);
            }
            None => return Ok(Loop::Continue),
        };
        let cfg = config::base_config();
        if let Some(f) = &cfg.selection_file {
            let mut buf = info.path.as_os_str().as_encoded_bytes().to_vec();
            buf.push(b'\n');
            if let Some(l) = info.line {
                buf.extend_from_slice(l.to_string().as_bytes());
            }
            std::fs::write(f, buf)?;
            Ok(Loop::Break)
        } else {
            Ok(Loop::OpenPath(
                info.path.as_os_str().to_os_string(),
                info.line,
            ))
        }
    }

    fn handle_navigate(&mut self, code: KeyCode, modifiers: KeyModifiers) -> io::Result<Loop> {
        let cfg = config::base_config();
        let keys = &cfg.keys;

        if modifiers.contains(KeyModifiers::ALT) {
            if code == KeyCode::Char('q') {
                self.enter_search()?;
            }
            return Ok(Loop::Continue);
        }
        if self.in_menu && keys.submit_search.contains(&code) {
            return self.handle_open();
        }
        if code == KeyCode::Down || keys.down.contains(&code) {
            self.step_down(false)?;
            return Ok(Loop::Continue);
        }
        if code == KeyCode::Up || keys.up.contains(&code) {
            self.step_up(false)?;
            return Ok(Loop::Continue);
        }
        if code == KeyCode::Esc {
            return Ok(Loop::Break);
        }
        if code == KeyCode::BackTab {
            self.enter_search()?;
            return Ok(Loop::Continue);
        }
        if keys.cycle_view.contains(&code) {
            let loc = self.jump;
            if self.jump_cursor(loc) {
                self.draw_results()?;
                self.draw_query()?;
                self.draw_filter()?;
            }
            self.jump.next();
            return Ok(Loop::JumpCycle);
        }
        if keys.open.contains(&code) {
            return self.handle_open();
        }
        if keys.help.contains(&code) {
            draw_popup(self.term, &help_popup())?;
            self.mode = Mode::Help;
            return Ok(Loop::Continue);
        }
        if keys.filter.contains(&code) {
            self.enter_filter()?;
            return Ok(Loop::Continue);
        }
        if keys.search.contains(&code) {
            self.enter_search()?;
            return Ok(Loop::Continue);
        }
        if keys.quit.contains(&code) {
            return Ok(Loop::Break);
        }

        let old_cursor_y = self.cursor_y;
        let big_jump = self.big_jump;
        let (changed, is_fold) = match code {
            KeyCode::PageDown => (self.down_page(), false),
            KeyCode::PageUp => (self.up_page(), false),
            KeyCode::Left => (self.up_path(), false),
            KeyCode::Right => (self.down_path(), false),
            KeyCode::Tab => (self.toggle_fold(), true),
            _ => {
                if keys.big_down.contains(&code) {
                    (self.down_clamp(big_jump), false)
                } else if keys.big_up.contains(&code) {
                    (self.up_clamp(big_jump), false)
                } else if keys.down_path.contains(&code) {
                    (self.down_path(), false)
                } else if keys.up_path.contains(&code) {
                    (self.up_path(), false)
                } else if keys.down_same_depth.contains(&code) {
                    (self.down_path_same_depth(), false)
                } else if keys.up_same_depth.contains(&code) {
                    (self.up_path_same_depth(), false)
                } else if keys.page_down.contains(&code) {
                    (self.down_page(), false)
                } else if keys.page_up.contains(&code) {
                    (self.up_page(), false)
                } else if keys.top.contains(&code) {
                    (self.top(), false)
                } else if keys.bottom.contains(&code) {
                    (self.bottom(), false)
                } else if keys.fold.contains(&code) {
                    (self.toggle_fold(), true)
                } else {
                    (false, false)
                }
            }
        };

        if changed {
            if is_fold {
                self.draw_results()?;
                self.draw_query()?;
                self.draw_filter()?;
            } else {
                if !self.visible.is_empty() {
                    let cfg = config::base_config();
                    queue!(
                        self.term,
                        cursor::MoveTo(0, old_cursor_y),
                        Print(cfg.chars.selected_indicator_clear.as_str())
                    )?;
                }
                self.draw_content()?;
                self.term.flush()?;
            }
        }

        Ok(Loop::Continue)
    }

    fn run(&mut self) -> io::Result<Option<(OsString, Option<usize>)>> {
        let mut dc = DoubleClick::new();
        loop {
            if self.debounce_deadline.is_some_and(|d| Instant::now() >= d) {
                self.debounce_deadline = None;
                self.trigger_search();
                self.draw_results()?;
                self.draw_query()?;
                self.draw_filter()?;
            }
            let mut got_result = false;
            while let Ok((id, result)) = self.result_rx.try_recv() {
                if id == self.search_gen {
                    match result {
                        Ok(r) => {
                            self.error_msg = None;
                            self.apply_results(r);
                        }
                        Err(e) => {
                            self.error_msg = Some(e);
                            self.apply_results(None);
                        }
                    }
                    got_result = true;
                }
            }
            if got_result {
                self.draw_results()?;
                self.draw_query()?;
                self.draw_filter()?;
                if self.mode == Mode::Help {
                    draw_popup(self.term, &help_popup())?;
                }
            }

            if !event::poll(Duration::from_millis(20))? {
                if self
                    .search_started
                    .is_some_and(|t| t.elapsed() > Duration::from_millis(150))
                    && !self.visible.is_empty()
                {
                    self.current = None;
                    self.visible.clear();
                    self.draw_results()?;
                    self.draw_query()?;
                    self.draw_filter()?;
                }
                continue;
            }
            let mut is_jump = false;
            match event::read()? {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    let ctrl = modifiers.contains(KeyModifiers::CONTROL);
                    let alt = modifiers.contains(KeyModifiers::ALT);
                    match (ctrl, code) {
                        (true, KeyCode::Char('c')) => break,
                        (true, KeyCode::Char('z')) => {
                            self.term.suspend()?;
                            self.resume()?;
                        }
                        _ => {
                            let lc = if self.mode == Mode::Help {
                                let cfg = config::base_config();
                                if cfg.keys.quit.contains(&code) || code == KeyCode::Esc {
                                    self.mode = Mode::Navigate;
                                    self.draw_results()?;
                                    self.draw_query()?;
                                    self.draw_filter()?;
                                }
                                Loop::Continue
                            } else if self.mode == Mode::Filter {
                                let changed = edit_bar(
                                    &mut self.filter,
                                    &mut self.filter_cursor,
                                    code,
                                    ctrl,
                                    alt,
                                );
                                match changed {
                                    Some(true) => {
                                        self.update_visible();
                                        if !event::poll(Duration::ZERO)? {
                                            self.draw_results()?;
                                            self.draw_query()?;
                                            self.draw_filter()?;
                                        }
                                    }
                                    Some(false) => self.draw_filter()?,
                                    None => match code {
                                        KeyCode::Esc => {
                                            self.mode = Mode::Navigate;
                                            self.filter.clear();
                                            self.filter_cursor = 0;
                                            self.bars_below = self.needs_bars();
                                            self.update_visible();
                                            self.draw_results()?;
                                            self.position_cursor()?;
                                            self.term.flush()?;
                                        }
                                        KeyCode::Enter => {
                                            self.mode = Mode::Navigate;
                                            self.position_cursor()?;
                                            self.term.flush()?;
                                        }
                                        KeyCode::BackTab => {
                                            self.enter_search()?;
                                        }
                                        _ => {}
                                    },
                                }
                                Loop::Continue
                            } else if self.mode == Mode::Navigate {
                                let r = self.handle_navigate(code, modifiers)?;
                                is_jump = matches!(r, Loop::JumpCycle);
                                if is_jump { Loop::Continue } else { r }
                            } else {
                                let cfg = config::base_config();
                                if code == KeyCode::BackTab
                                    || cfg.keys.submit_search.contains(&code)
                                {
                                    let had_pending_debounce =
                                        self.debounce_deadline.take().is_some();
                                    if cfg.live {
                                        if had_pending_debounce && !self.search.is_empty() {
                                            self.trigger_search();
                                            self.pending_save = true;
                                        } else if self.current.is_some() && !self.search.is_empty()
                                        {
                                            self.save_query_for_repeat();
                                        }
                                    } else {
                                        if !self.search.is_empty() {
                                            self.trigger_search();
                                            self.pending_save = true;
                                        }
                                    }
                                    self.mode = Mode::Navigate;
                                    self.draw_results()?;
                                    self.draw_query()?;
                                    self.draw_filter()?;
                                } else if code == KeyCode::Esc {
                                    self.debounce_deadline = None;
                                    self.mode = Mode::Navigate;
                                    self.draw_query()?;
                                } else {
                                    let changed = edit_bar(
                                        &mut self.search,
                                        &mut self.search_cursor,
                                        code,
                                        ctrl,
                                        alt,
                                    );
                                    match changed {
                                        Some(true) => {
                                            if cfg.live {
                                                let delay = cfg.live_delay.unwrap_or(0);
                                                if delay > 0 {
                                                    self.debounce_deadline = Some(
                                                        Instant::now()
                                                            + Duration::from_millis(delay),
                                                    );
                                                    self.draw_query()?;
                                                    self.draw_filter()?;
                                                } else {
                                                    self.trigger_search();
                                                    if !event::poll(Duration::ZERO)? {
                                                        self.draw_results()?;
                                                        self.draw_query()?;
                                                        self.draw_filter()?;
                                                    }
                                                }
                                            } else {
                                                self.needs_search = !self.search.is_empty();
                                                self.draw_query()?;
                                            }
                                        }
                                        Some(false) => self.draw_query()?,
                                        None => {}
                                    }
                                }
                                Loop::Continue
                            };
                            match lc {
                                Loop::Continue | Loop::JumpCycle => {}
                                Loop::Break => break,
                                Loop::OpenPath(path, line) => return self.open_result(path, line),
                            }
                        }
                    }
                }
                Event::Mouse(ev) => match ev.kind {
                    MouseEventKind::ScrollUp => self.step_up(true)?,
                    MouseEventKind::ScrollDown => self.step_down(true)?,
                    MouseEventKind::Down(btn) if btn.is_left() => dc.down(ev.row),
                    MouseEventKind::Up(btn) if btn.is_left() => {
                        if let Some(is_double) = dc.up(ev.row) {
                            if self.click_on(ev.row) {
                                self.draw_results()?;
                                self.draw_query()?;
                                self.draw_filter()?;
                            }
                            if is_double {
                                match self.handle_open()? {
                                    Loop::Continue | Loop::JumpCycle => {}
                                    Loop::Break => break,
                                    Loop::OpenPath(path, line) => {
                                        return self.open_result(path, line);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Event::Resize(w, h) if (self.term.height != h || self.term.width() != w) => {
                    self.term.set_dims(h, w);
                    self.set_dims(h);
                    if self.in_menu {
                        let max_cy = self.max_cursor_y();
                        if self.cursor_y > max_cy {
                            self.cursor_y = max_cy;
                        }
                    }
                    self.draw_results()?;
                    self.draw_query()?;
                    self.draw_filter()?;
                }
                _ => {}
            }
            if !is_jump {
                self.jump = ViewAnchor::Middle;
            }
        }
        self.term.give()?;
        Ok(None)
    }

    pub fn launch(term: &mut Term, matches: Option<Matches>) -> io::Result<()> {
        let mut menu = Menu::new(term);
        if let Some(m) = matches {
            menu.apply_results(Some((m, base_config())));
            menu.draw_results()?;
        }
        menu.draw_query()?;
        menu.draw_filter()?;
        if let Some((path, line)) = menu.run()? {
            return open_path(path, line);
        }
        Ok(())
    }
}

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

pub fn open_path(mut path: OsString, line_num: Option<usize>) -> io::Result<()> {
    let cfg = config::base_config();
    let env_editor = std::env::var("EDITOR").ok().filter(|s| !s.is_empty());
    let mut cmd = match cfg.editor.as_ref().or(env_editor.as_ref()) {
        Some(editor) => {
            let mut cmd = Command::new(editor);
            match cfg
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
