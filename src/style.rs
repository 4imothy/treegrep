// SPDX-License-Identifier: MIT

use crate::config;
use crossterm::style::{
    Attribute, Color, SetAttribute, SetForegroundColor, StyledContent, Stylize, style,
};
use std::fmt::{self, Display};

pub const RESET: SetAttribute = SetAttribute(Attribute::Reset);
const BOLD: SetAttribute = SetAttribute(Attribute::Bold);
pub const DIM: SetAttribute = SetAttribute(Attribute::Dim);

const RED_FG: SetForegroundColor = SetForegroundColor(Color::Red);
pub const SELECTED_INDICATOR_CLEAR: &str = "   ";
pub const LONG_BRANCH_FILE_SEPARATOR: &str = ", ";

pub const FILE_COLOR_DEFAULT: Color = Color::Cyan;
pub const DIR_COLOR_DEFAULT: Color = Color::Blue;
pub const LINE_NUMBER_COLOR_DEFAULT: Color = Color::Yellow;
pub const MATCHED_COLORS_DEFAULT: [Color; 3] = [Color::Green, Color::Magenta, Color::Red];
pub const SELECTED_BG_DEFAULT: Color = Color::DarkGrey;

pub struct Chars {
    pub bl: char,
    pub br: char,
    pub tl: char,
    pub tr: char,
    pub h: char,
    pub v: char,
    pub tee: char,
    pub selected_indicator: &'static str,
    pub ellipsis: char,
}

pub const NONE: Chars = Chars {
    bl: ' ',
    br: ' ',
    tl: ' ',
    tr: ' ',
    h: ' ',
    v: ' ',
    tee: ' ',
    selected_indicator: "   ",
    ellipsis: ' ',
};

pub const ASCII: Chars = Chars {
    bl: '+',
    br: '+',
    tl: '+',
    tr: '+',
    h: '-',
    v: '|',
    tee: '+',
    selected_indicator: "-> ",
    ellipsis: '|',
};

pub const SINGLE: Chars = Chars {
    bl: '└',
    br: '┘',
    tl: '┌',
    tr: '┐',
    h: '─',
    v: '│',
    tee: '├',
    selected_indicator: "─❱ ",
    ellipsis: '↴',
};

pub const ROUNDED: Chars = Chars {
    bl: '╰',
    br: '╯',
    tl: '╭',
    tr: '╮',
    h: '─',
    v: '│',
    tee: '├',
    selected_indicator: "─❱ ",
    ellipsis: '⤵',
};

pub const HEAVY: Chars = Chars {
    bl: '┗',
    br: '┛',
    tl: '┏',
    tr: '┓',
    h: '━',
    v: '┃',
    tee: '┣',
    selected_indicator: "━❱ ",
    ellipsis: '⤵',
};

pub const DOUBLE: Chars = Chars {
    bl: '╚',
    br: '╝',
    tl: '╔',
    tr: '╗',
    h: '═',
    v: '║',
    tee: '╠',
    selected_indicator: "═❱ ",
    ellipsis: '⤵',
};

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

pub fn error_prefix(bold: bool, colors: bool) -> String {
    let e = "error:";
    match (bold, colors) {
        (true, true) => format!("{BOLD}{RED_FG}{e}{RESET}"),
        (true, false) => format!("{BOLD}{e}{RESET}"),
        (false, true) => format!("{RED_FG}{e}{RESET}"),
        _ => e.into(),
    }
}

pub fn style_with<D>(orig: D, color: Color) -> StyledContent<D>
where
    D: Display,
{
    let mut styled = style(orig);
    if config().with_colors {
        styled = styled.with(color);
    }
    if config().with_bold {
        styled = styled.bold();
    }
    styled
}

pub fn match_substring(orig: &str, regexp_id: usize) -> StyledContent<&str> {
    style_with(
        orig,
        config().colors.matches[regexp_id % config().colors.matches.len()],
    )
}
