// SPDX-License-Identifier: MIT

use clap::{
    ArgAction, ArgGroup, Command, CommandFactory, Parser, ValueEnum, ValueHint,
    builder::PossibleValue,
    error::{ContextKind, ContextValue, Error, ErrorKind},
    value_parser,
};
use crossterm::event::KeyCode;
use std::{ffi::OsStr, path::PathBuf};

pub mod names {
    pub const TREEGREP_BIN: &str = "tgrep";
}

#[derive(Clone)]
pub enum OpenStrategy {
    Vi,
    Hx,
    Code,
    Jed,
    Default,
}

impl ValueEnum for OpenStrategy {
    fn value_variants<'a>() -> &'a [Self] {
        static VARIANTS: [OpenStrategy; 5] = [
            OpenStrategy::Vi,
            OpenStrategy::Hx,
            OpenStrategy::Code,
            OpenStrategy::Jed,
            OpenStrategy::Default,
        ];
        &VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let name = match self {
            OpenStrategy::Vi => "vi",
            OpenStrategy::Hx => "hx",
            OpenStrategy::Code => "code",
            OpenStrategy::Jed => "jed",
            OpenStrategy::Default => "default",
        };
        Some(PossibleValue::new(name))
    }
}

#[derive(Clone, Copy)]
pub enum Color {
    Black,
    White,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Grey,
    Rgb(u8, u8, u8),
    Ansi(u8),
}

pub const EXPR_HELP: &str = "a regex expression to search for";
pub const PATH_HELP: &str = "the path to search, if not provided, search the current directory";
const COLOR_HELP: &str =
    "black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)";

fn color_validation_error(cmd: &Command, kind: &str, value: &str) -> Error {
    let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
    err.insert(
        ContextKind::InvalidArg,
        ContextValue::String(kind.to_string()),
    );
    err.insert(
        ContextKind::InvalidValue,
        ContextValue::String(value.to_string()),
    );
    err
}

#[derive(Clone)]
pub struct ColorParser;

impl clap::builder::TypedValueParser for ColorParser {
    type Value = Color;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let s = value.to_string_lossy().trim().to_lowercase();

        let color = match s.as_str() {
            "black" => Color::Black,
            "white" => Color::White,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "grey" | "gray" => Color::Grey,
            _ if s.starts_with("rgb(") && s.ends_with(')') => {
                let inner = &s[4..s.len() - 1];
                let nums: Vec<_> = inner
                    .split('.')
                    .map(|x| x.trim().parse::<u8>())
                    .collect::<Result<_, _>>()
                    .map_err(|_| color_validation_error(cmd, "rgb", inner))?;
                if nums.len() == 3 {
                    Color::Rgb(nums[0], nums[1], nums[2])
                } else {
                    return Err(color_validation_error(cmd, "rgb", inner));
                }
            }
            _ if s.starts_with("ansi(") && s.ends_with(')') => {
                let inner = &s[5..s.len() - 1];
                let v = inner
                    .parse::<u8>()
                    .map_err(|_| color_validation_error(cmd, "ansi", inner))?;
                Color::Ansi(v)
            }
            _ => {
                let mut err = Error::new(ErrorKind::InvalidValue).with_cmd(cmd);
                err.insert(
                    ContextKind::InvalidArg,
                    ContextValue::String(arg.unwrap().to_string()),
                );
                err.insert(ContextKind::InvalidValue, ContextValue::String(s.clone()));
                return Err(err);
            }
        };

        Ok(color)
    }
}

#[derive(Clone)]
pub struct KeyCodeParser;

impl clap::builder::TypedValueParser for KeyCodeParser {
    type Value = KeyCode;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        let s = value.to_string_lossy();
        let lower = s.to_lowercase();

        let named = match lower.as_str() {
            "up" => Some(KeyCode::Up),
            "down" => Some(KeyCode::Down),
            "left" => Some(KeyCode::Left),
            "right" => Some(KeyCode::Right),
            "home" => Some(KeyCode::Home),
            "end" => Some(KeyCode::End),
            "pageup" => Some(KeyCode::PageUp),
            "pagedown" => Some(KeyCode::PageDown),
            "tab" => Some(KeyCode::Tab),
            "enter" => Some(KeyCode::Enter),
            "backspace" => Some(KeyCode::Backspace),
            "delete" => Some(KeyCode::Delete),
            "esc" => Some(KeyCode::Esc),
            "insert" => Some(KeyCode::Insert),
            _ => None,
        };
        if let Some(code) = named {
            return Ok(code);
        }

        if lower.len() > 1
            && lower.starts_with('f')
            && let Ok(n) = lower[1..].parse::<u8>()
            && n >= 1
        {
            return Ok(KeyCode::F(n));
        }

        let mut chars = s.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            return Ok(KeyCode::Char(c));
        }

        let mut err = Error::new(ErrorKind::InvalidValue).with_cmd(cmd);
        err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(arg.map(|a| a.to_string()).unwrap_or_default()),
        );
        err.insert(
            ContextKind::InvalidValue,
            ContextValue::String(s.into_owned()),
        );
        Err(err)
    }
}

pub fn key_display(code: KeyCode) -> String {
    match code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Home => "home".to_string(),
        KeyCode::End => "end".to_string(),
        KeyCode::PageUp => "pageup".to_string(),
        KeyCode::PageDown => "pagedown".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Insert => "insert".to_string(),
        KeyCode::F(n) => format!("f{n}"),
        _ => format!("{code:?}"),
    }
}

pub const LONG_BRANCHES_EACH: &str = "branch_each";
pub const NO_BOLD: &str = "no_bold";
pub const EXPRESSION_POSITIONAL: &str = "positional_regexp";
pub const EXPRESSION: &str = "regexp";
pub const PATH_POSITIONAL: &str = "positional_path";
pub const PATH: &str = "path";
pub const NO_COLORS: &str = "no_color";
pub const COUNT: &str = "count";
pub const HIDDEN: &str = "hidden";
pub const LINE_NUMBER: &str = "line_number";
pub const SELECT: &str = "select";
pub const MENU: &str = "menu";
pub const FILES: &str = "files";
pub const MAX_DEPTH: &str = "max_depth";
pub const CHAR_VERTICAL: &str = "char_vertical";
pub const CHAR_HORIZONTAL: &str = "char_horizontal";
pub const CHAR_TOP_LEFT: &str = "char_top_left";
pub const CHAR_TOP_RIGHT: &str = "char_top_right";
pub const CHAR_BOTTOM_LEFT: &str = "char_bottom_left";
pub const CHAR_BOTTOM_RIGHT: &str = "char_bottom_right";
pub const CHAR_TEE: &str = "char_tee";
pub const SELECTED_INDICATOR: &str = "selected_indicator";
pub const ELLIPSES: &str = "ellipsis";
pub const SEARCH_PROMPT: &str = "search_prompt";
pub const SEARCH_PROMPT_INACTIVE: &str = "search_prompt_inactive";
pub const FILTER_PROMPT: &str = "filter_prompt";
pub const KEY_DOWN: &str = "key_down";
pub const KEY_UP: &str = "key_up";
pub const KEY_BIG_DOWN: &str = "key_big_down";
pub const KEY_BIG_UP: &str = "key_big_up";
pub const KEY_DOWN_PATH: &str = "key_down_path";
pub const KEY_UP_PATH: &str = "key_up_path";
pub const KEY_DOWN_SAME_DEPTH: &str = "key_down_same_depth";
pub const KEY_UP_SAME_DEPTH: &str = "key_up_same_depth";
pub const KEY_TOP: &str = "key_top";
pub const KEY_BOTTOM: &str = "key_bottom";
pub const KEY_PAGE_DOWN: &str = "key_page_down";
pub const KEY_PAGE_UP: &str = "key_page_up";
pub const KEY_CYCLE_VIEW: &str = "key_cycle_view";
pub const KEY_HELP: &str = "key_help";
pub const KEY_QUIT: &str = "key_quit";
pub const KEY_OPEN: &str = "key_open";
pub const KEY_FOLD: &str = "key_fold";
pub const KEY_FILTER: &str = "key_filter";
pub const KEY_SEARCH: &str = "key_search";
pub const KEY_SUBMIT_SEARCH: &str = "key_submit_search";
pub const OVERVIEW: &str = "overview";
pub const AUTO_OPEN: &str = "auto_open";
pub const BRANCH_LEN: &str = "branch_len";
pub const LINKS: &str = "links";
pub const TRIM_LEFT: &str = "trim";
pub const NO_IGNORE: &str = "no_ignore";
pub const MAX_LENGTH: &str = "max_length";
pub const GLOB: &str = "glob";
pub const COMPLETIONS: &str = "completions";
pub const SELECTION_FILE: &str = "selection_file";
pub const REPEAT: &str = "repeat";
pub const REPEAT_FILE: &str = "repeat_file";
pub const BEFORE_CONTEXT: &str = "before_context";
pub const AFTER_CONTEXT: &str = "after_context";
pub const CONTEXT: &str = "context";

pub const DEFAULT_OPTS_ENV_NAME: &str = "TREEGREP_DEFAULT_OPTS";

pub const REPEAT_STORED: &[&str] = &[
    EXPRESSION,
    PATH,
    GLOB,
    HIDDEN,
    LINE_NUMBER,
    FILES,
    COUNT,
    LINKS,
    TRIM_LEFT,
    NO_IGNORE,
    MAX_DEPTH,
    MAX_LENGTH,
    BEFORE_CONTEXT,
    AFTER_CONTEXT,
    OVERVIEW,
    LONG_BRANCHES_EACH,
];

pub const HELP_TEMPLATE: &str = concat!(
    "{name} {version}\n\nby {author}\n\nhome page: ",
    env!("CARGO_PKG_HOMEPAGE"),
    "\n\n{about}\n\n{usage}\n\n{all-args}{after-help}"
);

#[derive(Parser, Clone)]
#[command(
    name = "tgrep",
    bin_name = "tgrep",
    no_binary_name = true,
    author,
    version,
    about,
    disable_help_flag = true,
    disable_version_flag = true,
    next_help_heading = "options",
    group(ArgGroup::new("mode").required(true).multiple(true).args([EXPRESSION_POSITIONAL, EXPRESSION, FILES, COMPLETIONS, MENU, REPEAT])),
    group(ArgGroup::new("expressions").args([EXPRESSION_POSITIONAL, EXPRESSION]).multiple(true)),
    group(ArgGroup::new("paths").args([PATH_POSITIONAL, PATH])),
)]
pub struct Args {
    #[arg(
        value_hint = ValueHint::Other,
        help_heading = "arguments",
        display_order = 1,
        index = 1,
        help = EXPR_HELP,
    )]
    pub positional_regexp: Option<String>,

    #[arg(
        value_hint = ValueHint::AnyPath,
        value_parser = value_parser!(PathBuf),
        help_heading = "arguments",
        display_order = 2,
        index = 2,
        help = PATH_HELP,
    )]
    pub positional_path: Option<PathBuf>,

    #[arg(
        long,
        short = 'e',
        value_name = "",
        value_hint = ValueHint::Other,
        action = ArgAction::Append,
        help = EXPR_HELP,
    )]
    pub regexp: Vec<String>,

    #[arg(long, short = 'p', value_name = "", help = PATH_HELP)]
    pub path: Option<PathBuf>,

    #[arg(
        long,
        short = 's',
        help = "results are shown in a selection interface for opening"
    )]
    pub select: bool,

    #[arg(long, short = 'm', help = "open a search and selection interface")]
    pub menu: bool,

    #[arg(
        long,
        short = 'f',
        help = "if an expression is given, hide matched content, otherwise, show the files that would be searched"
    )]
    pub files: bool,

    #[arg(long, short = '.', help = "search hidden files")]
    pub hidden: bool,

    #[arg(long, short = 'n', help = "show the line numbers of matches")]
    pub line_number: bool,

    #[arg(
        long,
        short = 'c',
        help = "display number of files matched in directory and number of lines matched in a file"
    )]
    pub count: bool,

    #[arg(long, short = 'g', value_name = "", action = ArgAction::Append, help = "rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic")]
    pub glob: Vec<String>,

    #[arg(long, short = 'l', help = "search linked paths")]
    pub links: bool,

    #[arg(long, short = 'o', help = "conclude results with an overview")]
    pub overview: bool,

    #[arg(long, short = 'd', value_name = "", help = "the max depth to search")]
    pub max_depth: Option<usize>,

    #[arg(
        long,
        short = 'C',
        value_name = "",
        requires = "expressions",
        help = "number of lines to show before and after each match"
    )]
    pub context: Option<usize>,

    #[arg(
        long,
        short = 'B',
        value_name = "",
        requires = "expressions",
        help = "number of lines to show before each match"
    )]
    pub before_context: Option<usize>,

    #[arg(
        long,
        short = 'A',
        value_name = "",
        requires = "expressions",
        help = "number of lines to show after each match"
    )]
    pub after_context: Option<usize>,

    #[arg(long, help = "trigger search on every keystroke in the menu")]
    pub live: bool,

    #[arg(
        long,
        value_name = "",
        requires = "expressions",
        help = "set the max length for a matched line"
    )]
    pub max_length: Option<usize>,

    #[arg(long, help = "disable ignore files")]
    pub no_ignore: bool,

    #[arg(
        long,
        requires = "expressions",
        help = "trim whitespace at the beginning of lines"
    )]
    pub trim: bool,

    #[arg(long, value_name = "", help = "set the number of threads to use")]
    pub threads: Option<usize>,

    #[arg(long, value_name = "", help = "command used to open selections")]
    pub editor: Option<String>,

    #[arg(
        long,
        help = "if there is only one match, open it in the configured editor"
    )]
    pub auto_open: bool,

    #[arg(
        long,
        value_name = "",
        help = "command line syntax for opening a file at a line"
    )]
    pub open_like: Option<OpenStrategy>,

    #[arg(long, value_name = "", help = "generate completions for given shell")]
    pub completions: Option<clap_complete::Shell>,

    #[arg(long, help = "repeats the last saved search")]
    pub repeat: bool,

    #[arg(
        long,
        default_value_t = 3,
        hide_short_help = true,
        value_name = "",
        help = "number of characters to show before a match"
    )]
    pub branch_len: usize,

    #[arg(
        long,
        default_value_t = 1,
        requires = FILES,
        value_name = "",
        help = "number of files to print on each branch"
    )]
    pub branch_each: usize,

    #[arg(long, help = "disable colors")]
    pub no_color: bool,

    #[arg(long, help = "disable bold")]
    pub no_bold: bool,

    #[arg(long, help = "disable mouse events")]
    pub no_mouse: bool,

    #[arg(long, help = "disable the terminal alternate screen")]
    pub no_alternate_screen: bool,

    #[arg(long, value_parser = ColorParser, value_name = "", hide_short_help = true, help = COLOR_HELP)]
    pub file_color: Option<Color>,

    #[arg(long, value_parser = ColorParser, value_name = "", hide_short_help = true, help = COLOR_HELP)]
    pub dir_color: Option<Color>,

    #[arg(long, value_parser = ColorParser, value_name = "", hide_short_help = true, help = COLOR_HELP)]
    pub text_color: Option<Color>,

    #[arg(long, value_parser = ColorParser, value_name = "", hide_short_help = true, help = COLOR_HELP)]
    pub branch_color: Option<Color>,

    #[arg(long, value_parser = ColorParser, value_name = "", hide_short_help = true, help = COLOR_HELP)]
    pub line_number_color: Option<Color>,

    #[arg(long, value_parser = ColorParser, value_name = "", value_delimiter = ',', hide_short_help = true, help = COLOR_HELP)]
    pub match_colors: Vec<Color>,

    #[arg(long, value_parser = ColorParser, value_name = "", hide_short_help = true, help = COLOR_HELP)]
    pub selected_indicator_color: Option<Color>,

    #[arg(long, value_parser = ColorParser, value_name = "", hide_short_help = true, help = COLOR_HELP)]
    pub selected_bg_color: Option<Color>,

    #[arg(long, value_parser = ColorParser, value_name = "", hide_short_help = true, help = COLOR_HELP)]
    pub filter_highlight_color: Option<Color>,

    #[arg(
        long,
        default_value_t = '│',
        hide_short_help = true,
        value_name = "",
        help = "vertical branch character"
    )]
    pub char_vertical: char,

    #[arg(
        long,
        default_value_t = '─',
        hide_short_help = true,
        value_name = "",
        help = "horizontal branch character"
    )]
    pub char_horizontal: char,

    #[arg(
        long,
        default_value_t = '╭',
        hide_short_help = true,
        value_name = "",
        help = "top-left corner character"
    )]
    pub char_top_left: char,

    #[arg(
        long,
        default_value_t = '╮',
        hide_short_help = true,
        value_name = "",
        help = "top-right corner character"
    )]
    pub char_top_right: char,

    #[arg(
        long,
        default_value_t = '╰',
        hide_short_help = true,
        value_name = "",
        help = "bottom-left corner character"
    )]
    pub char_bottom_left: char,

    #[arg(
        long,
        default_value_t = '╯',
        hide_short_help = true,
        value_name = "",
        help = "bottom-right corner character"
    )]
    pub char_bottom_right: char,

    #[arg(
        long,
        default_value_t = '├',
        hide_short_help = true,
        value_name = "",
        help = "tee branch character"
    )]
    pub char_tee: char,

    #[arg(
        long,
        default_value = "⤵",
        hide_short_help = true,
        value_name = "",
        help = "folded indicator"
    )]
    pub ellipsis: String,

    #[arg(
        long,
        default_value = "➜ ",
        hide_short_help = true,
        value_name = "",
        help = "search mode prompt"
    )]
    pub search_prompt: String,

    #[arg(
        long,
        default_value = "- ",
        hide_short_help = true,
        value_name = "",
        help = "search prompt when not searching"
    )]
    pub search_prompt_inactive: String,

    #[arg(
        long,
        default_value = "/",
        hide_short_help = true,
        value_name = "",
        help = "filter mode prompt"
    )]
    pub filter_prompt: String,

    #[arg(
        long,
        default_value = "─❱ ",
        hide_short_help = true,
        value_name = "",
        help = "selected indicator characters"
    )]
    pub selected_indicator: String,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["down", "j", "n"], hide_short_help = true, value_name = "", help = "move down")]
    pub key_down: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["up", "k", "p"], hide_short_help = true, value_name = "", help = "move up")]
    pub key_up: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["J", "N"], hide_short_help = true, value_name = "", help = "big jump down")]
    pub key_big_down: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["K", "P"], hide_short_help = true, value_name = "", help = "big jump up")]
    pub key_big_up: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["}", "]"], hide_short_help = true, value_name = "", help = "move down to the next path")]
    pub key_down_path: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["{", "["], hide_short_help = true, value_name = "", help = "move up to the previous path")]
    pub key_up_path: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = [")", "d"], hide_short_help = true, value_name = "", help = "move down to the next path at same depth")]
    pub key_down_same_depth: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["(", "u"], hide_short_help = true, value_name = "", help = "move up to the previous path at same depth")]
    pub key_up_same_depth: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["home", "g", "<"], hide_short_help = true, value_name = "", help = "move to the top")]
    pub key_top: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["end", "G", ">"], hide_short_help = true, value_name = "", help = "move to the bottom")]
    pub key_bottom: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["pagedown", "f"], hide_short_help = true, value_name = "", help = "page down")]
    pub key_page_down: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["pageup", "b"], hide_short_help = true, value_name = "", help = "page up")]
    pub key_page_up: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["z", "l"], hide_short_help = true, value_name = "", help = "cycle cursor position (top/center/bottom)")]
    pub key_cycle_view: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["h"], hide_short_help = true, value_name = "", help = "show help")]
    pub key_help: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["q"], hide_short_help = true, value_name = "", help = "quit")]
    pub key_quit: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["enter"], hide_short_help = true, value_name = "", help = "open selection")]
    pub key_open: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["tab"], hide_short_help = true, value_name = "", help = "fold/unfold path")]
    pub key_fold: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["/", "s"], hide_short_help = true, value_name = "", help = "filter within results")]
    pub key_filter: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = [":"], hide_short_help = true, value_name = "", help = "enter search mode")]
    pub key_search: Vec<KeyCode>,

    #[arg(long, value_parser = KeyCodeParser, default_values = ["enter"], hide_short_help = true, value_name = "", help = "submit search query")]
    pub key_submit_search: Vec<KeyCode>,

    #[arg(long, value_parser = value_parser!(PathBuf), value_name = "", hide_short_help = true, value_hint = ValueHint::AnyPath, help = "file to write selection to")]
    pub selection_file: Option<PathBuf>,

    #[arg(long, value_parser = value_parser!(PathBuf), value_name = "", hide_short_help = true, value_hint = ValueHint::AnyPath, help = "file to save and replay regexp searches")]
    pub repeat_file: Option<PathBuf>,

    #[arg(long, short = 'h', action = ArgAction::Help)]
    pub help: Option<bool>,

    #[arg(long, short = 'V', action = ArgAction::Version)]
    pub version: Option<bool>,
}

pub fn generate_command() -> Command {
    let mut cmd = Args::command()
        .args_override_self(true)
        .help_template(HELP_TEMPLATE)
        .after_help(format!(
            "arguments are prefixed with the contents of the {DEFAULT_OPTS_ENV_NAME} environment variable"
        ));
    for id in REPEAT_STORED {
        cmd = cmd.mut_arg(id, |a| {
            let help = a
                .get_help()
                .map(|h| format!("{h} (saved for repeat)"))
                .unwrap_or_else(|| "(saved for repeat)".to_string());
            a.help(help)
        });
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::builder::TypedValueParser;
    use std::ffi::OsStr;

    fn parse_key(s: &str) -> Result<KeyCode, clap::Error> {
        KeyCodeParser.parse_ref(&generate_command(), None, OsStr::new(s))
    }

    #[test]
    fn test_named_keys() {
        assert_eq!(parse_key("up").unwrap(), KeyCode::Up);
        assert_eq!(parse_key("down").unwrap(), KeyCode::Down);
        assert_eq!(parse_key("left").unwrap(), KeyCode::Left);
        assert_eq!(parse_key("right").unwrap(), KeyCode::Right);
        assert_eq!(parse_key("tab").unwrap(), KeyCode::Tab);
        assert_eq!(parse_key("enter").unwrap(), KeyCode::Enter);
        assert_eq!(parse_key("pageup").unwrap(), KeyCode::PageUp);
        assert_eq!(parse_key("pagedown").unwrap(), KeyCode::PageDown);
        assert_eq!(parse_key("esc").unwrap(), KeyCode::Esc);
        assert_eq!(parse_key("home").unwrap(), KeyCode::Home);
        assert_eq!(parse_key("end").unwrap(), KeyCode::End);
        assert_eq!(parse_key("backspace").unwrap(), KeyCode::Backspace);
        assert_eq!(parse_key("delete").unwrap(), KeyCode::Delete);
        assert_eq!(parse_key("insert").unwrap(), KeyCode::Insert);
    }

    #[test]
    fn test_char_keys() {
        assert_eq!(parse_key("j").unwrap(), KeyCode::Char('j'));
        assert_eq!(parse_key("J").unwrap(), KeyCode::Char('J'));
        assert_eq!(parse_key("1").unwrap(), KeyCode::Char('1'));
    }

    #[test]
    fn test_function_keys() {
        assert_eq!(parse_key("f1").unwrap(), KeyCode::F(1));
        assert_eq!(parse_key("f12").unwrap(), KeyCode::F(12));
        assert_eq!(parse_key("f255").unwrap(), KeyCode::F(255));
    }

    #[test]
    fn test_invalid_keys() {
        assert!(parse_key("toolong").is_err());
        assert!(parse_key("").is_err());
    }
}
