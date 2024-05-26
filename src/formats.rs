// SPDX-License-Identifier: CC-BY-4.0

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

pub const TREE_SPACER_LEN: usize = 3;
pub const STRAIGHT_BL: char = '└';
pub const STRAIGHT_BR: char = '┘';
pub const STRAIGHT_TL: char = '┌';
pub const STRAIGHT_TR: char = '┐';
pub const HORIZONTAL: char = '─';
pub const VERTICAL: char = '│';
pub const TEE: char = '├';

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

pub fn error_prefix(colors: bool) -> String {
    let e_str = "error:";
    if colors {
        format!("{}{}{}{}", BOLD, RED_FG, e_str, RESET)
    } else {
        e_str.to_string()
    }
}

pub fn reset_bold_and_fg() -> Vec<u8> {
    (RESET_COLOR.to_string() + &NO_BOLD.to_string()).into_bytes()
}

pub fn bold() -> Vec<u8> {
    BOLD.to_string().into_bytes()
}

pub fn dir_name(name: &str) -> StyledContent<&str> {
    name.with(Color::Blue).attribute(Attribute::Bold)
}

pub fn file_name(name: &str) -> StyledContent<&str> {
    name.with(Color::Cyan).attribute(Attribute::Bold)
}

pub fn line_number(num: usize) -> StyledContent<String> {
    format!("{}:", num)
        .with(Color::Yellow)
        .attribute(Attribute::Bold)
}

pub fn get_color(i: usize) -> SetForegroundColor {
    MATCHED_COLORS[i % MATCHED_COLORS.len()]
}
