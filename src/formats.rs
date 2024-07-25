// SPDX-License-Identifier: CC-BY-4.0

use crate::config;
use crossterm::style::{
    Attribute, Color, ResetColor, SetAttribute, SetForegroundColor, StyledContent, Stylize,
};
use std::fmt::{self, Display};

pub const RESET: SetAttribute = SetAttribute(Attribute::Reset);
pub const RESET_COLOR: ResetColor = ResetColor;
pub const NO_BOLD: SetAttribute = SetAttribute(Attribute::NormalIntensity);
pub const BOLD: SetAttribute = SetAttribute(Attribute::Bold);

const RED_FG: SetForegroundColor = SetForegroundColor(Color::Red);
const GREEN_FG: SetForegroundColor = SetForegroundColor(Color::Green);
const MAGENTA_FG: SetForegroundColor = SetForegroundColor(Color::Magenta);
const MATCHED_COLORS: [SetForegroundColor; 3] = [GREEN_FG, MAGENTA_FG, RED_FG];

pub const MENU_SELECTED: Color = Color::DarkGrey;
pub const SELECTED_INDICATOR: &str = "-> ";
pub const SELECTED_INDICATOR_CLEAR: &str = "   ";
pub const LONG_BRANCH_FILE_SEPARATOR: &str = ", ";

pub const PREFIX_LEN_DEFAULT: usize = 3;

pub struct Chars {
    pub bl: char,
    pub br: char,
    pub tl: char,
    pub tr: char,
    pub h: char,
    pub v: char,
    pub tee: char,
}

pub const NONE: Chars = Chars {
    bl: ' ',
    br: ' ',
    tl: ' ',
    tr: ' ',
    h: ' ',
    v: ' ',
    tee: ' ',
};

pub const SINGLE: Chars = Chars {
    bl: '└',
    br: '┘',
    tl: '┌',
    tr: '┐',
    h: '─',
    v: '│',
    tee: '├',
};

pub const ROUNDED: Chars = Chars {
    bl: '╰',
    br: '╯',
    tl: '╭',
    tr: '╮',
    h: '─',
    v: '│',
    tee: '├',
};

pub const HEAVY: Chars = Chars {
    bl: '┗',
    br: '┛',
    tl: '┏',
    tr: '┓',
    h: '━',
    v: '┃',
    tee: '┣',
};

pub const DOUBLE: Chars = Chars {
    bl: '╚',
    br: '╝',
    tl: '╔',
    tr: '╗',
    h: '═',
    v: '║',
    tee: '╠',
};

pub const NEW_LINE: char = '\n';

pub struct DisplayRepeater<T>(T, usize);
impl<T: Display> Display for DisplayRepeater<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.1 {
            self.0.fmt(f)?;
        }
        Ok(())
    }
}
pub fn repeat<T>(item: T, times: usize) -> DisplayRepeater<T> {
    DisplayRepeater(item, times)
}

pub fn error_prefix() -> String {
    let e_str = "error:";
    match (config().colors, config().bold) {
        (true, true) => format!("{}{}{}{}", BOLD, RED_FG, e_str, RESET),
        (true, false) => format!("{}{}{}", RED_FG, e_str, RESET),
        _ => e_str.to_string(),
    }
}

pub fn reset_bold_and_fg() -> Vec<u8> {
    (RESET_COLOR.to_string() + &NO_BOLD.to_string()).into_bytes()
}

pub fn bold() -> Vec<u8> {
    BOLD.to_string().into_bytes()
}

pub fn dir_name(name: &str) -> StyledContent<&str> {
    let mut styled_name = name.with(Color::Blue);
    if config().bold {
        styled_name = styled_name.attribute(Attribute::Bold);
    }
    styled_name
}

pub fn file_name(name: &str) -> StyledContent<&str> {
    let mut styled_name = name.with(Color::Cyan);
    if config().bold {
        styled_name = styled_name.attribute(Attribute::Bold);
    }
    styled_name
}

pub fn line_number(num: usize) -> StyledContent<String> {
    format!("{}:", num)
        .with(Color::Yellow)
        .attribute(Attribute::Bold)
}

pub fn get_color(i: usize) -> SetForegroundColor {
    MATCHED_COLORS[i % MATCHED_COLORS.len()]
}
