// SPDX-License-Identifier: MIT

use crate::config;
use crossterm::style::{
    style, Attribute, Color, SetAttribute, SetForegroundColor, StyledContent, Stylize,
};
use std::fmt::{self, Display};

pub const RESET: SetAttribute = SetAttribute(Attribute::Reset);
const BOLD: SetAttribute = SetAttribute(Attribute::Bold);

const RED_FG: SetForegroundColor = SetForegroundColor(Color::Red);
const MATCHED_COLORS: [Color; 3] = [Color::Green, Color::Magenta, Color::Red];

pub const MENU_SELECTED: Color = Color::DarkGrey;
pub const SELECTED_INDICATOR_CLEAR: &str = "   ";
pub const LONG_BRANCH_FILE_SEPARATOR: &str = ", ";

pub struct Chars {
    pub bl: char,
    pub br: char,
    pub tl: char,
    pub tr: char,
    pub h: char,
    pub v: char,
    pub tee: char,
    pub selected_indicator: &'static str,
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
};

pub const NEW_LINE: char = '\n';
pub const CRLF: &str = "\r\n";

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
    let e_str = "error:";
    match (bold, colors) {
        (true, true) => format!("{}{}{}{}", BOLD, RED_FG, e_str, RESET),
        (true, false) => format!("{}{}{}", BOLD, e_str, RESET),
        (false, true) => format!("{}{}{}", RED_FG, e_str, RESET),
        _ => e_str.to_string(),
    }
}

pub fn style_str(orig: &str, color: Color, attr: Attribute) -> StyledContent<&str> {
    let mut styled = style(orig);
    if config().colors {
        styled = styled.with(color);
    }
    if config().bold {
        styled = styled.attribute(attr);
    }
    styled
}

pub fn dir_name(name: &str) -> StyledContent<&str> {
    style_str(name, Color::Blue, Attribute::Bold)
}

pub fn file_name(name: &str) -> StyledContent<&str> {
    style_str(name, Color::Cyan, Attribute::Bold)
}

pub fn match_substring(orig: &str, pattern_id: usize) -> StyledContent<&str> {
    style_str(orig, match_color(pattern_id), Attribute::Bold)
}

pub fn line_number(num: usize) -> StyledContent<usize> {
    let mut styled_num = style(num);
    if config().colors {
        styled_num = styled_num.with(Color::Yellow);
    }
    if config().bold {
        styled_num = styled_num.attribute(Attribute::Bold);
    }
    styled_num
}

pub fn match_color(i: usize) -> Color {
    MATCHED_COLORS[i % MATCHED_COLORS.len()]
}
