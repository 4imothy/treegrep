// SPDX-License-Identifier: MIT

use clap::{
    Arg, ArgAction, ArgGroup, Command, Error, ValueEnum, ValueHint,
    builder::PossibleValue,
    error::{ContextKind, ContextValue, ErrorKind},
    value_parser,
};
use crossterm::event::KeyCode;
use std::{ffi::OsStr, path::PathBuf};

pub const DEFAULT_PREFIX_LEN: &str = "3";
pub const DEFAULT_LONG_BRANCH_EACH: &str = "5";

pub mod names {
    pub const TREEGREP_BIN: &str = "tgrep";
}

pub struct ArgInfo {
    pub id: &'static str,
    pub h: &'static str,
    pub s: Option<char>,
}

impl ArgInfo {
    const fn new(id: &'static str, h: &'static str, s: Option<char>) -> Self {
        Self { id, h, s }
    }
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

#[derive(Clone)]
struct ColorParser;

impl clap::builder::TypedValueParser for ColorParser {
    type Value = Color;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
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
                    .map_err(|_| {
                        let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                        err.insert(
                            ContextKind::InvalidArg,
                            ContextValue::String("rgb".to_string()),
                        );
                        err.insert(
                            ContextKind::InvalidValue,
                            ContextValue::String(inner.to_string()),
                        );
                        err
                    })?;
                if nums.len() == 3 {
                    Color::Rgb(nums[0], nums[1], nums[2])
                } else {
                    let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                    err.insert(
                        ContextKind::InvalidArg,
                        ContextValue::String("rgb".to_string()),
                    );
                    err.insert(
                        ContextKind::InvalidValue,
                        ContextValue::String(inner.to_string()),
                    );
                    return Err(err);
                }
            }
            _ if s.starts_with("ansi(") && s.ends_with(')') => {
                let inner = &s[5..s.len() - 1];
                let v = inner.parse::<u8>().map_err(|_| {
                    let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                    err.insert(
                        ContextKind::InvalidArg,
                        ContextValue::String("ansi".to_string()),
                    );
                    err.insert(
                        ContextKind::InvalidValue,
                        ContextValue::String(inner.to_string()),
                    );
                    err
                })?;
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
struct KeyCodeParser;

impl clap::builder::TypedValueParser for KeyCodeParser {
    type Value = KeyCode;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
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

pub const EXPRESSION_GROUP_ID: &str = "expressions";
pub const TARGET_GROUP_ID: &str = "targets";

pub const LONG_BRANCHES_EACH: ArgInfo = ArgInfo::new(
    "long-branch-each",
    "number of files to print on each branch",
    None,
);
pub const NO_BOLD: ArgInfo = ArgInfo::new("no-bold", "don't bold anything", None);
pub const PATH_POSITIONAL: ArgInfo = ArgInfo::new("positional target", PATH_HELP, None);
pub const PATH: ArgInfo = ArgInfo::new("path", PATH_HELP, Some('p'));
pub const EXPRESSION_POSITIONAL: ArgInfo = ArgInfo::new("positional regexp", EXPR_HELP, None);
pub const EXPRESSION: ArgInfo = ArgInfo::new("regexp", EXPR_HELP, Some('e'));
pub const NO_COLORS: ArgInfo = ArgInfo::new("no-color", "don't use colors", None);
pub const COUNT: ArgInfo = ArgInfo::new(
    "count",
    "display number of files matched in directory and number of lines matched in a file",
    Some('c'),
);
pub const HIDDEN: ArgInfo = ArgInfo::new("hidden", "search hidden files", Some('.'));
pub const LINE_NUMBER: ArgInfo =
    ArgInfo::new("line-number", "show line number of match", Some('n'));
pub const SELECT: ArgInfo = ArgInfo::new(
    "select",
    "results are shown in a selection interface for opening",
    Some('s'),
);
pub const MENU: ArgInfo = ArgInfo::new(
    "menu",
    "provide arguments and select results through an interface",
    None,
);
pub const HELP: ArgInfo = ArgInfo::new("help", "print help", Some('h'));
pub const VERSION: ArgInfo = ArgInfo::new("version", "print version", Some('V'));
pub const FILES: ArgInfo = ArgInfo::new(
    "files",
    "if an expression is given, hide matched content, \
     otherwise, show the files that would be searched",
    Some('f'),
);
pub const MAX_DEPTH: ArgInfo = ArgInfo::new("max-depth", "the max depth to search", Some('d'));
pub const CHAR_VERTICAL: ArgInfo = ArgInfo::new("char-vertical", "vertical branch character", None);
pub const CHAR_HORIZONTAL: ArgInfo =
    ArgInfo::new("char-horizontal", "horizontal branch character", None);
pub const CHAR_TOP_LEFT: ArgInfo = ArgInfo::new("char-top-left", "top-left corner character", None);
pub const CHAR_TOP_RIGHT: ArgInfo =
    ArgInfo::new("char-top-right", "top-right corner character", None);
pub const CHAR_BOTTOM_LEFT: ArgInfo =
    ArgInfo::new("char-bottom-left", "bottom-left corner character", None);
pub const CHAR_BOTTOM_RIGHT: ArgInfo =
    ArgInfo::new("char-bottom-right", "bottom-right corner character", None);
pub const CHAR_TEE: ArgInfo = ArgInfo::new("char-tee", "tee branch character", None);
pub const CHAR_ELLIPSIS: ArgInfo =
    ArgInfo::new("char-ellipsis", "folding indicator character", None);
pub const KEY_DOWN: ArgInfo = ArgInfo::new("key-down", "move down", None);
pub const KEY_UP: ArgInfo = ArgInfo::new("key-up", "move up", None);
pub const KEY_BIG_DOWN: ArgInfo = ArgInfo::new("key-big-down", "big jump down", None);
pub const KEY_BIG_UP: ArgInfo = ArgInfo::new("key-big-up", "big jump up", None);
pub const KEY_DOWN_PATH: ArgInfo = ArgInfo::new("key-down-path", "next path", None);
pub const KEY_UP_PATH: ArgInfo = ArgInfo::new("key-up-path", "previous path", None);
pub const KEY_DOWN_SAME_DEPTH: ArgInfo =
    ArgInfo::new("key-down-same-depth", "next path at same depth", None);
pub const KEY_UP_SAME_DEPTH: ArgInfo =
    ArgInfo::new("key-up-same-depth", "previous path at same depth", None);
pub const KEY_TOP: ArgInfo = ArgInfo::new("key-top", "go to top", None);
pub const KEY_BOTTOM: ArgInfo = ArgInfo::new("key-bottom", "go to bottom", None);
pub const KEY_PAGE_DOWN: ArgInfo = ArgInfo::new("key-page-down", "page down", None);
pub const KEY_PAGE_UP: ArgInfo = ArgInfo::new("key-page-up", "page up", None);
pub const KEY_CENTER: ArgInfo = ArgInfo::new("key-center", "center cursor", None);
pub const KEY_HELP: ArgInfo = ArgInfo::new("key-help", "show help", None);
pub const KEY_QUIT: ArgInfo = ArgInfo::new("key-quit", "quit", None);
pub const KEY_OPEN: ArgInfo = ArgInfo::new("key-open", "open selection", None);
pub const KEY_FOLD: ArgInfo = ArgInfo::new("key-fold", "fold/unfold path", None);
pub const KEY_SEARCH: ArgInfo = ArgInfo::new("key-search", "search within results", None);
pub const EDITOR: ArgInfo = ArgInfo::new("editor", "command used to open selections", None);
pub const OPEN_LIKE: ArgInfo = ArgInfo::new(
    "open-like",
    "command line syntax for opening a file at a line",
    None,
);
pub const PREFIX_LEN: ArgInfo = ArgInfo::new(
    "prefix-len",
    "number of characters to show before a match",
    None,
);
pub const LINKS: ArgInfo = ArgInfo::new("links", "search linked paths", Some('l'));
pub const TRIM_LEFT: ArgInfo =
    ArgInfo::new("trim", "trim whitespace at the beginning of lines", None);
pub const THREADS: ArgInfo = ArgInfo::new(
    "threads",
    "set the appropriate number of threads to use",
    None,
);
pub const NO_IGNORE: ArgInfo = ArgInfo::new("no-ignore", "don't use ignore files", None);
pub const MAX_LENGTH: ArgInfo =
    ArgInfo::new("max-length", "set the max length for a matched line", None);
pub const GLOB: ArgInfo = ArgInfo::new(
    "glob",
    "rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic",
    Some('g'),
);
pub const COMPLETIONS: ArgInfo =
    ArgInfo::new("completions", "generate completions for given shell", None);
pub const SELECTION_FILE: ArgInfo = ArgInfo::new(
    "selection-file",
    "file to write selection to (first line: file path, second line: line number if applicable)",
    None,
);
pub const REPEAT: ArgInfo = ArgInfo::new("repeat", "repeats the last saved search", None);
pub const REPEAT_FILE: ArgInfo =
    ArgInfo::new("repeat-file", "file where arguments are saved", None);
pub const OVERVIEW: ArgInfo =
    ArgInfo::new("overview", "conclude results with an overview", Some('o'));
pub const LONG_BRANCHES: ArgInfo = ArgInfo::new(
    "long-branch",
    "multiple files from the same directory are shown on the same branch",
    None,
);
pub const FILE_COLOR: ArgInfo = ArgInfo::new("file-color", COLOR_HELP, None);
pub const DIR_COLOR: ArgInfo = ArgInfo::new("dir-color", COLOR_HELP, None);
pub const TEXT_COLOR: ArgInfo = ArgInfo::new("text-color", COLOR_HELP, None);
pub const BRANCH_COLOR: ArgInfo = ArgInfo::new("branch-color", COLOR_HELP, None);
pub const LINE_NUMBER_COLOR: ArgInfo = ArgInfo::new("line-number-color", COLOR_HELP, None);
pub const SELECTED_INDICATOR_COLOR: ArgInfo =
    ArgInfo::new("selected-indicator-color", COLOR_HELP, None);
pub const SELECTED_BG_COLOR: ArgInfo = ArgInfo::new("selected-bg-color", COLOR_HELP, None);
pub const MATCH_COLORS: ArgInfo = ArgInfo::new("match-colors", COLOR_HELP, None);
pub const SEARCH_HIGHLIGHT_COLOR: ArgInfo =
    ArgInfo::new("search-highlight-color", COLOR_HELP, None);
pub const SELECTED_INDICATOR: ArgInfo =
    ArgInfo::new("selected-indicator", "selected indicator characters", None);
pub const DEFAULT_SELECTED_INDICATOR: &str = "─❱ ";
pub const BEFORE_CONTEXT: ArgInfo = ArgInfo::new(
    "before-context",
    "number of lines to show before each match",
    Some('B'),
);
pub const AFTER_CONTEXT: ArgInfo = ArgInfo::new(
    "after-context",
    "number of lines to show after each match",
    Some('A'),
);
pub const CONTEXT: ArgInfo = ArgInfo::new(
    "context",
    "number of lines to show before and after each match",
    Some('C'),
);

const HELP_TEMPLATE: &str = concat!(
    "{name} {version}

by {author}

home page: ",
    env!("CARGO_PKG_HOMEPAGE"),
    "

{about}

{usage}

{all-args}{after-help}"
);

pub const DEFAULT_OPTS_ENV_NAME: &str = "TREEGREP_DEFAULT_OPTS";
const DONT_NEED_REGEXP: &[&str] = &[FILES.id, COMPLETIONS.id, MENU.id, REPEAT.id];

pub fn generate_command() -> Command {
    let mut command = Command::new(env!("CARGO_PKG_NAME"))
        .no_binary_name(true)
        .bin_name(names::TREEGREP_BIN)
        .help_template(HELP_TEMPLATE.to_owned())
        .args_override_self(true)
        .after_help(format!(
            "arguments are prefixed with the contents of \
             the {DEFAULT_OPTS_ENV_NAME} environment variable"
        ))
        .author(env!("CARGO_PKG_AUTHORS"))
        .disable_help_flag(true)
        .disable_version_flag(true)
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .next_help_heading("options");

    command = add_expressions(command);
    command = add_paths(command);

    for opt in get_args() {
        command = command.arg(opt);
    }

    command
}

fn bool_arg(info: ArgInfo) -> Arg {
    let mut arg = Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .action(ArgAction::SetTrue);

    if let Some(s) = info.s {
        arg = arg.short(s);
    }

    arg
}

fn color_arg(info: ArgInfo) -> Arg {
    Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .value_name("")
        .value_parser(ColorParser)
        .action(ArgAction::Set)
        .hide_short_help(true)
}

fn usize_arg(info: &ArgInfo, default_value: Option<&'static str>) -> Arg {
    let mut arg = Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .value_name("")
        .action(ArgAction::Set);
    if let Some(s) = info.s {
        arg = arg.short(s);
    }
    if let Some(dv) = default_value {
        arg = arg.default_value(dv);
    }
    arg
}

fn hidden_usize_arg(info: &ArgInfo, default_value: Option<&'static str>) -> Arg {
    usize_arg(info, default_value).hide_short_help(true)
}

fn char_arg(info: ArgInfo) -> Arg {
    Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .value_name("")
        .value_parser(value_parser!(char))
        .action(ArgAction::Set)
        .hide_short_help(true)
}

fn string_arg(info: ArgInfo, default_value: &'static str) -> Arg {
    Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .value_name("")
        .default_value(default_value)
        .action(ArgAction::Set)
        .hide_short_help(true)
}

fn key_arg(info: ArgInfo, defaults: &'static [&'static str]) -> Arg {
    Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .value_name("")
        .value_parser(KeyCodeParser)
        .action(ArgAction::Append)
        .default_values(defaults)
        .hide_short_help(true)
}

fn get_args() -> Vec<Arg> {
    let long_branches = Arg::new(LONG_BRANCHES.id)
        .long(LONG_BRANCHES.id)
        .help(LONG_BRANCHES.h)
        .requires(FILES.id)
        .action(ArgAction::SetTrue);

    let glob = Arg::new(GLOB.id)
        .long(GLOB.id)
        .help(GLOB.h)
        .short(GLOB.s)
        .value_name("")
        .action(ArgAction::Append);

    let selection_file = Arg::new(SELECTION_FILE.id)
        .long(SELECTION_FILE.id)
        .help(SELECTION_FILE.h)
        .value_parser(value_parser!(PathBuf))
        .value_name("")
        .value_hint(ValueHint::AnyPath);

    let repeat_file = Arg::new(REPEAT_FILE.id)
        .long(REPEAT_FILE.id)
        .help(REPEAT_FILE.h)
        .value_parser(value_parser!(PathBuf))
        .value_name("")
        .value_hint(ValueHint::AnyPath);

    let editor = Arg::new(EDITOR.id)
        .long(EDITOR.id)
        .help(EDITOR.h)
        .value_name("")
        .action(ArgAction::Set);

    let open_like = Arg::new(OPEN_LIKE.id)
        .long(OPEN_LIKE.id)
        .help(OPEN_LIKE.h)
        .value_parser(value_parser!(OpenStrategy))
        .value_name("")
        .action(ArgAction::Set);

    let completions = Arg::new(COMPLETIONS.id)
        .long(COMPLETIONS.id)
        .help(COMPLETIONS.h)
        .value_parser(clap::value_parser!(clap_complete::Shell))
        .value_name("")
        .action(ArgAction::Set);
    let help = Arg::new(HELP.id)
        .long(HELP.id)
        .short(HELP.s)
        .help(HELP.h)
        .action(ArgAction::Help);
    let version = Arg::new(VERSION.id)
        .long(VERSION.id)
        .short(VERSION.s)
        .help(VERSION.h)
        .action(ArgAction::Version);

    vec![
        bool_arg(SELECT),
        bool_arg(FILES),
        bool_arg(HIDDEN),
        bool_arg(LINE_NUMBER),
        bool_arg(COUNT),
        glob,
        bool_arg(LINKS),
        bool_arg(OVERVIEW),
        usize_arg(&MAX_DEPTH, None),
        usize_arg(&CONTEXT, None).requires(EXPRESSION_GROUP_ID),
        usize_arg(&BEFORE_CONTEXT, None).requires(EXPRESSION_GROUP_ID),
        usize_arg(&AFTER_CONTEXT, None).requires(EXPRESSION_GROUP_ID),
        usize_arg(&MAX_LENGTH, None).requires(EXPRESSION_GROUP_ID),
        bool_arg(MENU),
        bool_arg(NO_IGNORE),
        bool_arg(TRIM_LEFT).requires(EXPRESSION_GROUP_ID),
        usize_arg(&THREADS, None),
        long_branches,
        editor,
        open_like,
        completions,
        selection_file,
        repeat_file,
        bool_arg(REPEAT),
        bool_arg(NO_COLORS),
        bool_arg(NO_BOLD),
        color_arg(FILE_COLOR),
        color_arg(DIR_COLOR),
        color_arg(TEXT_COLOR),
        color_arg(LINE_NUMBER_COLOR),
        color_arg(BRANCH_COLOR),
        color_arg(MATCH_COLORS).value_delimiter(','),
        color_arg(SELECTED_INDICATOR_COLOR),
        color_arg(SELECTED_BG_COLOR),
        color_arg(SEARCH_HIGHLIGHT_COLOR),
        hidden_usize_arg(&PREFIX_LEN, Some(DEFAULT_PREFIX_LEN)),
        hidden_usize_arg(&LONG_BRANCHES_EACH, Some(DEFAULT_LONG_BRANCH_EACH))
            .requires(LONG_BRANCHES.id),
        char_arg(CHAR_VERTICAL),
        char_arg(CHAR_HORIZONTAL),
        char_arg(CHAR_TOP_LEFT),
        char_arg(CHAR_TOP_RIGHT),
        char_arg(CHAR_BOTTOM_LEFT),
        char_arg(CHAR_BOTTOM_RIGHT),
        char_arg(CHAR_TEE),
        char_arg(CHAR_ELLIPSIS),
        string_arg(SELECTED_INDICATOR, DEFAULT_SELECTED_INDICATOR),
        key_arg(KEY_DOWN, &["down", "j", "n"]),
        key_arg(KEY_UP, &["up", "k", "p"]),
        key_arg(KEY_BIG_DOWN, &["J", "N"]),
        key_arg(KEY_BIG_UP, &["K", "P"]),
        key_arg(KEY_DOWN_PATH, &["}", "]"]),
        key_arg(KEY_UP_PATH, &["{", "["]),
        key_arg(KEY_DOWN_SAME_DEPTH, &[")", "d"]),
        key_arg(KEY_UP_SAME_DEPTH, &["(", "u"]),
        key_arg(KEY_TOP, &["home", "g", "<"]),
        key_arg(KEY_BOTTOM, &["end", "G", ">"]),
        key_arg(KEY_PAGE_DOWN, &["pagedown", "f"]),
        key_arg(KEY_PAGE_UP, &["pageup", "b"]),
        key_arg(KEY_CENTER, &["z", "l"]),
        key_arg(KEY_HELP, &["h"]),
        key_arg(KEY_QUIT, &["q"]),
        key_arg(KEY_OPEN, &["enter"]),
        key_arg(KEY_FOLD, &["tab"]),
        key_arg(KEY_SEARCH, &["/", "s"]),
        help,
        version,
    ]
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

fn add_expressions(command: Command) -> Command {
    command
        .arg(
            Arg::new(EXPRESSION_POSITIONAL.id)
                .help(EXPRESSION_POSITIONAL.h)
                .required_unless_present_any(DONT_NEED_REGEXP)
                .required_unless_present(EXPRESSION.id)
                .value_hint(ValueHint::Other)
                .help_heading("arguments")
                .index(1),
        )
        .arg(
            Arg::new(EXPRESSION.id)
                .long(EXPRESSION.id)
                .short(EXPRESSION.s.unwrap())
                .help(EXPRESSION.h)
                .value_name("")
                .value_hint(ValueHint::Other)
                .required_unless_present_any(DONT_NEED_REGEXP)
                .required_unless_present(EXPRESSION_POSITIONAL.id)
                .action(ArgAction::Append),
        )
        .group(
            ArgGroup::new(EXPRESSION_GROUP_ID)
                .id(EXPRESSION_GROUP_ID)
                .args([EXPRESSION_POSITIONAL.id, EXPRESSION.id])
                .multiple(true),
        )
}

fn add_paths(command: Command) -> Command {
    command
        .arg(
            Arg::new(PATH_POSITIONAL.id)
                .help(PATH_POSITIONAL.h)
                .value_hint(ValueHint::AnyPath)
                .value_parser(value_parser!(PathBuf))
                .help_heading("arguments")
                .index(2),
        )
        .arg(
            Arg::new(PATH.id)
                .long(PATH.id)
                .short(PATH.s.unwrap())
                .help(PATH.h)
                .value_name("")
                .value_parser(value_parser!(PathBuf))
                .value_hint(ValueHint::AnyPath),
        )
        .group(
            ArgGroup::new(TARGET_GROUP_ID)
                .id(TARGET_GROUP_ID)
                .args([PATH_POSITIONAL.id, PATH.id])
                .multiple(false),
        )
}
