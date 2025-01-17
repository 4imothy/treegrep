// SPDX-License-Identifier: MIT

use crate::config;
use crate::match_system::Match;
use crossterm::style::{
    style, Attribute, Color, ResetColor, SetAttribute, SetForegroundColor, StyledContent, Stylize,
};
use std::fmt::{self, Display};

const RESET: SetAttribute = SetAttribute(Attribute::Reset);
const RESET_COLOR: ResetColor = ResetColor;
const NO_BOLD: SetAttribute = SetAttribute(Attribute::NormalIntensity);
const BOLD: SetAttribute = SetAttribute(Attribute::Bold);

const RED_FG: SetForegroundColor = SetForegroundColor(Color::Red);
const GREEN_FG: SetForegroundColor = SetForegroundColor(Color::Green);
const MAGENTA_FG: SetForegroundColor = SetForegroundColor(Color::Magenta);
const MATCHED_COLORS: [SetForegroundColor; 3] = [GREEN_FG, MAGENTA_FG, RED_FG];

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

pub fn resets() -> String {
    let mut result = String::new();

    if config().colors {
        result.push_str(&RESET_COLOR.to_string());
    }

    if config().bold {
        result.push_str(&NO_BOLD.to_string());
    }

    result
}

fn style_str(orig: &str, color: Color, attr: Attribute) -> StyledContent<&str> {
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

pub fn line_number(num: usize) -> StyledContent<String> {
    let mut styled_num = style(format!("{}:", num));
    if config().colors {
        styled_num = styled_num.with(Color::Yellow);
    }
    if config().bold {
        styled_num = styled_num.attribute(Attribute::Bold);
    }
    styled_num
}

pub fn bold() -> Vec<u8> {
    BOLD.to_string().into_bytes()
}

pub fn color(i: usize) -> Vec<u8> {
    MATCHED_COLORS[i % MATCHED_COLORS.len()]
        .to_string()
        .into_bytes()
}

pub fn style_line(mut contents: &[u8], matches: Vec<Match>) -> Vec<u8> {
    let cut;
    if config().trim {
        (contents, cut) = contents.trim_left();
    } else {
        cut = 0;
    }
    if config().max_length < contents.len() {
        contents = &contents[0..config().max_length];
    }
    let mut styled_line = contents.to_vec();
    if config().colors {
        let mut shift = 0;
        for mut m in matches {
            if m.start == m.end || m.start >= config().max_length {
                continue;
            }
            m.start -= cut;
            m.end -= cut;
            let styler = color(m.pattern_id);
            let mut start = m.start + shift;
            shift += styler.len();
            styled_line.splice(start..start, styler.into_iter());
            start = m.start + shift;
            if config().bold {
                let bold = bold();
                shift += bold.len();
                styled_line.splice(start..start, bold.into_iter());
            }
            let end = m.end + shift;
            let reset = resets();
            shift += reset.len();
            styled_line.splice(end..end, reset.bytes().into_iter());
        }
    }

    styled_line
}

trait SliceExt {
    fn trim_left(&self) -> (&Self, usize);
}

impl SliceExt for [u8] {
    fn trim_left(&self) -> (&[u8], usize) {
        fn is_space(b: u8) -> bool {
            match b {
                b'\t' | b'\n' | b'\x0B' | b'\x0C' | b'\r' | b' ' => true,
                _ => false,
            }
        }

        let start = self
            .iter()
            .take_while(|&&b| -> bool { is_space(b) })
            .count();

        (&self[start..], start)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_left() {
        let bytes: &[u8] = b"    \t  Hello, World!";

        let (trimmed, count) = bytes.trim_left();

        assert_eq!(trimmed, b"Hello, World!");
        assert_eq!(count, 7);
    }
}
