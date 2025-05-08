// SPDX-License-Identifier: MIT

use clap::builder::PossibleValue;
use clap::{Arg, ArgAction, ArgGroup, Command, ValueHint};
use clap_complete;

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
pub const HIDE_CONTENT_GROUP_ID: &str = "hide_contents";
pub const CHAR_STYLE_OPTIONS: [&str; 6] = ["ascii", "single", "double", "heavy", "rounded", "none"];

arg_info!(
    TREE,
    "tree",
    "display the files that would be search in tree format",
    't'
);

arg_info!(
    LONG_BRANCHES,
    "long-branch",
    "multiple files from the same directory are shown on the same branch"
);
arg_info!(
    LONG_BRANCHES_EACH,
    "long-branch-each",
    "number of files to print on each branch"
);
arg_info!(NO_BOLD, "no-bold", "don't bold anything");
pub const PATH_HELP: &str = "the path to search. If not provided, search the current directory.";
arg_info!(PATH_POSITIONAL, "positional target", PATH_HELP);
arg_info!(PATH, "path", PATH_HELP, 'p');
pub const EXPR_HELP: &str = "the regex expression";
arg_info!(EXPRESSION_POSITIONAL, "positional regexp", EXPR_HELP);
arg_info!(EXPRESSION, "regexp", EXPR_HELP, 'e');
arg_info!(NO_COLORS, "no-color", "don't use colors");
arg_info!(
    SHOW_COUNT,
    "count",
    "display number of files matched in directory and number of lines matched in a file",
    'c'
);
arg_info!(HIDDEN, "hidden", "search hidden files", '.');
arg_info!(LINE_NUMBER, "line-number", "show line number of match", 'n');
arg_info!(MENU, "menu", MENU_HELP, 'm');
arg_info!(FILES, "files", "don't show matched content", 'f');
arg_info!(MAX_DEPTH, "max-depth", "the max depth to search");
arg_info!(SEARCHER, "searcher", "executable to do the searching", 's');
arg_info!(CHAR_STYLE, "char-style", "style of characters to use");
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

const HELP: &str = concat!(
    "{name} {version}

by {author}

home page: ",
    env!("CARGO_PKG_HOMEPAGE"),
    "

{about}

{usage}
{all-args}{after-help}"
);

pub const MENU_HELP: &str = "open results in a menu to be edited with $EDITOR
navigate through the menu using the following commands:
\u{0020}- move up/down: k/j, p/n, up arrow/down arrow
\u{0020}- move up/down with a bigger jump: K/J, P/N
\u{0020}- move up/down paths: {/}, [/]
\u{0020}- move to the start/end: g/G, </>, home/end
\u{0020}- move up/down a page: b/f, pageup/pagedown
\u{0020}- center cursor: z/l
\u{0020}- quit: q, ctrl + c";

pub const DEFAULT_OPTS_ENV_NAME: &str = "TREEGREP_DEFAULT_OPTS";

pub fn generate_command() -> Command {
    let mut command = Command::new(env!("CARGO_PKG_NAME"))
        .no_binary_name(true)
        .bin_name(names::TREEGREP_BIN)
        .help_template(HELP.to_owned())
        .args_override_self(true)
        .after_help(
            "any of the above can be set using the ".to_string()
                + DEFAULT_OPTS_ENV_NAME
                + " environment variable",
        )
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"));

    if tree_arg_present() {
        command = command.allow_missing_positional(true);
    }

    command = command.group(
        ArgGroup::new(HIDE_CONTENT_GROUP_ID)
            .id(HIDE_CONTENT_GROUP_ID)
            .args([TREE.id, FILES.id]),
    );

    command = add_expressions(command);
    command = add_targets(command);

    command = command.arg(
        Arg::new(COMPLETIONS.id)
            .long(COMPLETIONS.id)
            .help(COMPLETIONS.h)
            .action(ArgAction::Set)
            .value_name("shell")
            .value_parser(clap::value_parser!(clap_complete::Shell))
            .exclusive(true),
    );

    for opt in get_args() {
        command = command.arg(opt);
    }
    return command;
}

fn tree_arg_present() -> bool {
    std::env::args().any(|arg| {
        arg == format!("--{}", TREE.id)
            || (arg.starts_with('-')
                && arg.chars().nth(1) != Some('-')
                && arg.chars().skip(1).any(|c| c == TREE.s.unwrap()))
    })
}

pub fn completions_arg_present() -> bool {
    std::env::args().any(|arg| {
        arg == format!("--{}", COMPLETIONS.id)
            || arg == format!("--{}", COMPLETIONS.id)
            || arg.starts_with(&format!("--{}=", COMPLETIONS.id))
    })
}

fn bool_arg(info: ArgInfo, requires_expr: bool) -> Arg {
    let mut arg = Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .action(ArgAction::SetTrue);

    if let Some(s) = info.s {
        arg = arg.short(s);
    }

    if requires_expr {
        arg = arg.requires(EXPRESSION_GROUP_ID);
    }

    arg
}

fn usize_arg(info: &ArgInfo, requires_expr: bool, default_value: Option<&'static str>) -> Arg {
    let mut arg = Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .value_name("")
        .action(ArgAction::Set);
    if let Some(dv) = default_value {
        arg = arg.default_value(dv);
    }
    if requires_expr {
        arg = arg.requires(EXPRESSION_GROUP_ID);
    }
    arg
}

fn get_args<'a>() -> [Arg; 21] {
    let tree = Arg::new(TREE.id)
        .long(TREE.id)
        .help(TREE.h)
        .action(ArgAction::SetTrue)
        .conflicts_with(EXPRESSION_GROUP_ID)
        .short(TREE.s);

    let long = Arg::new(LONG_BRANCHES.id)
        .long(LONG_BRANCHES.id)
        .help(LONG_BRANCHES.h)
        .value_name("")
        .requires(HIDE_CONTENT_GROUP_ID)
        .action(ArgAction::SetTrue);

    let glob = Arg::new(GLOB.id)
        .long(GLOB.id)
        .help(GLOB.h)
        .value_name("")
        .action(ArgAction::Append);

    let searcher = Arg::new(SEARCHER.id)
        .long(SEARCHER.id)
        .short(SEARCHER.s.unwrap())
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
        .value_parser(
            CHAR_STYLE_OPTIONS
                .iter()
                .map(|&s| PossibleValue::new(s).hide(false))
                .collect::<Vec<_>>(),
        )
        .value_name("")
        .action(ArgAction::Set);
    [
        tree,
        glob,
        searcher,
        usize_arg(&THREADS, false, None),
        bool_arg(HIDDEN, false),
        bool_arg(LINE_NUMBER, false),
        bool_arg(FILES, true),
        bool_arg(LINKS, false),
        bool_arg(TRIM_LEFT, true),
        bool_arg(PCRE2, true),
        bool_arg(NO_IGNORE, false),
        bool_arg(SHOW_COUNT, false),
        bool_arg(NO_COLORS, false),
        bool_arg(NO_BOLD, false),
        usize_arg(&MAX_DEPTH, false, None),
        usize_arg(&PREFIX_LEN, false, Some(DEFAULT_PREFIX_LEN)),
        usize_arg(&MAX_LENGTH, true, None),
        usize_arg(&LONG_BRANCHES_EACH, true, Some(DEFAULT_LONG_BRANCH_EACH)),
        char_style,
        long,
        bool_arg(MENU, false),
    ]
}

fn add_expressions(command: Command) -> Command {
    command
        .arg(
            Arg::new(EXPRESSION_POSITIONAL.id)
                .help(EXPRESSION_POSITIONAL.h)
                .required_unless_present_any([TREE.id, LONG_BRANCHES.id, EXPRESSION.id])
                .index(1),
        )
        .arg(
            Arg::new(EXPRESSION.id)
                .long(EXPRESSION.id)
                .short(EXPRESSION.s.unwrap())
                .help(EXPRESSION.h)
                .value_name("")
                .required_unless_present_any([TREE.id, LONG_BRANCHES.id, EXPRESSION_POSITIONAL.id])
                .action(ArgAction::Append),
        )
        .group(
            ArgGroup::new(EXPRESSION_GROUP_ID)
                .id(EXPRESSION_GROUP_ID)
                .args([EXPRESSION_POSITIONAL.id, EXPRESSION.id])
                .multiple(true),
        )
}

fn add_targets(command: Command) -> Command {
    command
        .arg(
            Arg::new(PATH_POSITIONAL.id)
                .help(PATH_POSITIONAL.h)
                .value_hint(ValueHint::AnyPath)
                .index(2),
        )
        .arg(
            Arg::new(PATH.id)
                .long(PATH.id)
                .short(PATH.s.unwrap())
                .help(PATH.h)
                .value_name("")
                .value_hint(ValueHint::AnyPath),
        )
        .group(
            ArgGroup::new(TARGET_GROUP_ID)
                .id(TARGET_GROUP_ID)
                .args([PATH_POSITIONAL.id, PATH.id])
                .multiple(false),
        )
}
