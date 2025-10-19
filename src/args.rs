// SPDX-License-Identifier: MIT

use clap::{
    Arg, ArgAction, ArgGroup, Command, Error, ValueEnum, ValueHint,
    builder::PossibleValue,
    error::{ContextKind, ContextValue, ErrorKind},
    value_parser,
};
use std::{ffi::OsStr, path::PathBuf};

pub const DEFAULT_PREFIX_LEN: &str = "3";
pub const DEFAULT_LONG_BRANCH_EACH: &str = "5";

pub mod names {
    pub const TREEGREP: &str = "treegrep";
    pub const TREEGREP_BIN: &str = "tgrep";
    pub const RIPGREP: &str = "ripgrep";
    pub const RIPGREP_BIN: &str = "rg";
}

pub struct ArgInfo {
    pub id: &'static str,
    pub h: &'static str,
    pub s: Option<char>,
}

impl ArgInfo {
    const fn new(id: &'static str, h: &'static str, s: Option<char>) -> Self {
        ArgInfo { id, h, s }
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

#[derive(Clone, Copy)]
pub enum CharacterStyle {
    Single,
    Rounded,
    Heavy,
    Double,
    Ascii,
    None,
}

impl ValueEnum for CharacterStyle {
    fn value_variants<'a>() -> &'a [Self] {
        static VARIANTS: [CharacterStyle; 6] = [
            CharacterStyle::Single,
            CharacterStyle::Rounded,
            CharacterStyle::Heavy,
            CharacterStyle::Double,
            CharacterStyle::Ascii,
            CharacterStyle::None,
        ];
        &VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let name = match self {
            CharacterStyle::Single => "single",
            CharacterStyle::Rounded => "rounded",
            CharacterStyle::Heavy => "heavy",
            CharacterStyle::Double => "double",
            CharacterStyle::Ascii => "ascii",
            CharacterStyle::None => "none",
        };
        Some(PossibleValue::new(name))
    }
}

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

macro_rules! arg_info {
    ($var_name:ident, $name:expr, $description:expr) => {
        pub const $var_name: ArgInfo = ArgInfo::new($name, $description, None);
    };
    ($var_name:ident, $name:expr, $description:expr, $short:expr) => {
        pub const $var_name: ArgInfo = ArgInfo::new($name, $description, Some($short));
    };
}

pub const EXPRESSION_GROUP_ID: &str = "expressions";
pub const TARGET_GROUP_ID: &str = "targets";

arg_info!(
    LONG_BRANCHES_EACH,
    "long-branch-each",
    "number of files to print on each branch"
);
arg_info!(NO_BOLD, "no-bold", "don't bold anything");
pub const PATH_HELP: &str = "the path to search, if not provided, search the current directory";
arg_info!(PATH_POSITIONAL, "positional target", PATH_HELP);
arg_info!(PATH, "path", PATH_HELP, 'p');
pub const EXPR_HELP: &str = "the regex expression";
arg_info!(EXPRESSION_POSITIONAL, "positional regexp", EXPR_HELP);
arg_info!(EXPRESSION, "regexp", EXPR_HELP, 'e');
arg_info!(NO_COLORS, "no-color", "don't use colors");
arg_info!(
    COUNT,
    "count",
    "display number of files matched in directory and number of lines matched in a file",
    'c'
);
arg_info!(HIDDEN, "hidden", "search hidden files", '.');
arg_info!(LINE_NUMBER, "line-number", "show line number of match", 'n');
arg_info!(
    SELECT,
    "select",
    "results are shown in a selection interface for opening",
    's'
);
arg_info!(
    MENU,
    "menu",
    "provide arguments and select results through an interface"
);
arg_info!(HELP, "help", "print help", 'h');
arg_info!(VERSION, "version", "print version", 'V');
arg_info!(
    FILES,
    "files",
    "if an expression is given, hide matched content, otherwise, show the files that would be searched",
    'f'
);
arg_info!(MAX_DEPTH, "max-depth", "the max depth to search");
arg_info!(SEARCHER, "searcher", "executable to do the searching");
arg_info!(CHAR_STYLE, "char-style", "style of characters to use");
arg_info!(EDITOR, "editor", "command used to open selections");
arg_info!(
    OPEN_LIKE,
    "open-like",
    "command line syntax for opening a file at a line"
);
arg_info!(
    PREFIX_LEN,
    "prefix-len",
    "number of characters to show before a match"
);
arg_info!(LINKS, "links", "search linked paths");
arg_info!(
    TRIM_LEFT,
    "trim",
    "trim whitespace at the beginning of lines"
);
arg_info!(PCRE2, "pcre2", "enable PCRE2");
arg_info!(
    THREADS,
    "threads",
    "set the appropriate number of threads to use"
);
arg_info!(NO_IGNORE, "no-ignore", "don't use ignore files");
arg_info!(
    MAX_LENGTH,
    "max-length",
    "set the max length for a matched line"
);
arg_info!(
    GLOB,
    "glob",
    "rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic"
);
arg_info!(
    COMPLETIONS,
    "completions",
    "generate completions for given shell"
);
arg_info!(
    SELECTION_FILE,
    "selection-file",
    "file to write selection to (first line: file path, second line: line number if applicable)"
);
arg_info!(REPEAT, "repeat", "repeats the last saved search");
arg_info!(REPEAT_FILE, "repeat-file", "file where arguments are saved");
arg_info!(OVERVIEW, "overview", "conclude results with an overview");
arg_info!(
    LONG_BRANCHES,
    "long-branch",
    "multiple files from the same directory are shown on the same branch"
);
arg_info!(FILE_COLOR, "file-color", COLOR_HELP);
arg_info!(DIR_COLOR, "dir-color", COLOR_HELP);
arg_info!(TEXT_COLOR, "text-color", COLOR_HELP);
arg_info!(BRANCH_COLOR, "branch-color", COLOR_HELP);
arg_info!(LINE_NUMBER_COLOR, "line-number-color", COLOR_HELP);
arg_info!(
    SELECTED_INDICATOR_COLOR,
    "selected-indicator-color",
    COLOR_HELP
);
arg_info!(SELECTED_BG_COLOR, "selected-bg-color", COLOR_HELP);
arg_info!(MATCH_COLORS, "match-colors", COLOR_HELP);

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
        .after_help(
            "arguments are prefixed with the contents of the ".to_string()
                + DEFAULT_OPTS_ENV_NAME
                + " environment variable",
        )
        .author(env!("CARGO_PKG_AUTHORS"))
        .disable_help_flag(true)
        .disable_version_flag(true)
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"));

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
}

fn usize_arg(info: &ArgInfo, default_value: Option<&'static str>) -> Arg {
    let mut arg = Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .value_name("")
        .action(ArgAction::Set);
    if let Some(dv) = default_value {
        arg = arg.default_value(dv);
    }
    arg
}

fn get_args() -> [Arg; 38] {
    let long_branches = Arg::new(LONG_BRANCHES.id)
        .long(LONG_BRANCHES.id)
        .help(LONG_BRANCHES.h)
        .requires(FILES.id)
        .action(ArgAction::SetTrue);

    let glob = Arg::new(GLOB.id)
        .long(GLOB.id)
        .help(GLOB.h)
        .value_name("")
        .action(ArgAction::Append);

    let searcher = Arg::new(SEARCHER.id)
        .long(SEARCHER.id)
        .help(SEARCHER.h)
        .value_parser([
            PossibleValue::new(names::RIPGREP_BIN).hide(false),
            PossibleValue::new(names::TREEGREP_BIN).hide(false),
            PossibleValue::new(names::RIPGREP).hide(true),
            PossibleValue::new(names::TREEGREP).hide(true),
        ])
        .value_name("")
        .action(ArgAction::Set);

    let char_style = Arg::new(CHAR_STYLE.id)
        .long(CHAR_STYLE.id)
        .help(CHAR_STYLE.h)
        .value_parser(value_parser!(CharacterStyle))
        .value_name("")
        .action(ArgAction::Set);

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

    [
        bool_arg(MENU),
        bool_arg(SELECT),
        glob,
        bool_arg(FILES),
        bool_arg(HIDDEN),
        bool_arg(LINE_NUMBER),
        bool_arg(LINKS),
        bool_arg(NO_IGNORE),
        bool_arg(COUNT),
        bool_arg(NO_COLORS),
        bool_arg(NO_BOLD),
        bool_arg(OVERVIEW),
        usize_arg(&MAX_DEPTH, None),
        usize_arg(&PREFIX_LEN, Some(DEFAULT_PREFIX_LEN)),
        usize_arg(&MAX_LENGTH, None).requires(EXPRESSION_GROUP_ID),
        bool_arg(TRIM_LEFT).requires(EXPRESSION_GROUP_ID),
        bool_arg(PCRE2).requires(EXPRESSION_GROUP_ID),
        usize_arg(&THREADS, None),
        long_branches,
        usize_arg(&LONG_BRANCHES_EACH, Some(DEFAULT_LONG_BRANCH_EACH)).requires(LONG_BRANCHES.id),
        editor,
        open_like,
        char_style,
        color_arg(FILE_COLOR),
        color_arg(DIR_COLOR),
        color_arg(TEXT_COLOR),
        color_arg(LINE_NUMBER_COLOR),
        color_arg(BRANCH_COLOR),
        color_arg(MATCH_COLORS).value_delimiter(','),
        color_arg(SELECTED_INDICATOR_COLOR),
        color_arg(SELECTED_BG_COLOR),
        completions,
        searcher,
        selection_file,
        repeat_file,
        bool_arg(REPEAT),
        help,
        version,
    ]
}

fn add_expressions(command: Command) -> Command {
    command
        .arg(
            Arg::new(EXPRESSION_POSITIONAL.id)
                .help(EXPRESSION_POSITIONAL.h)
                .required_unless_present_any(DONT_NEED_REGEXP)
                .required_unless_present(EXPRESSION.id)
                .value_hint(ValueHint::Other)
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
