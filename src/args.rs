// SPDX-License-Identifier: CC-BY-4.0

use clap::builder::PossibleValue;
use clap::{
    crate_authors, crate_description, crate_name, crate_version, Arg, ArgAction, ArgGroup, Command,
    ValueHint,
};

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

arg_info!(
    TREE,
    "tree",
    "display the files that would be search in tree format",
    'l'
);
pub const TARGET_HELP: &str =
    "specify the search target. If none provided, search the current directory.";
arg_info!(TARGET_POSITIONAL, "positional target", TARGET_HELP);
arg_info!(TARGET, "target", TARGET_HELP, 't');
pub const EXPR_HELP: &str = "specify the regex expression";
pub const EXPRESSION_GROUP_ID: &str = "expressions";
arg_info!(EXPRESSION_POSITIONAL, "positional regexp", EXPR_HELP);
arg_info!(EXPRESSION, "regexp", EXPR_HELP, 'e');
arg_info!(NO_COLORS, "no-color", "don't use colors if present");
arg_info!(
    SHOW_COUNT,
    "count",
    "display number of files matched in directory and number of lines matched in a file if present",
    'c'
);
arg_info!(HIDDEN, "hidden", "search hidden files", '.');
arg_info!(
    LINE_NUMBER,
    "line-number",
    "show line number of match if present",
    'n'
);
arg_info!(MENU, "menu", MENU_HELP, 'm');
arg_info!(FILES, "files", "show the paths that have matches", 'f');
arg_info!(MAX_DEPTH, "max-depth", "the max depth to search");
arg_info!(SEARCHER, "searcher", "executable to do the searching", 's');
arg_info!(LINKS, "links", "show linked paths for symbolic links");
arg_info!(
    TRIM_LEFT,
    "trim",
    "trim whitespace at the beginning of lines"
);
arg_info!(PCRE2, "pcre2", "enable PCRE2 if the searcher supports it");
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

const HELP: &str = "{name} {version}
by {author}

{about}

{usage}

{all-args}{after-help}";

const MENU_HELP: &str = "open results in a menu to be edited with $EDITOR
navigate through the menu using the following commands:
\t- move up/down: k/j, p/n, up arrow/down arrow
\t- move up/down with a bigger jump: K/J, P/N
\t- move up/down paths: {/}, [/]
\t- move to the start/end: g/G, </>, home/end
\t- move up/down a page: ctrl + b/ctrl + f, pageup/pagedown";

pub const DEFAULT_OPTS_ENV_NAME: &str = "TREEGREP_DEFAULT_OPTS";

pub fn generate_command() -> Command {
    let mut command = Command::new(crate_name!())
        .help_template(HELP.to_owned())
        .after_help(
            "Any of the above can be set using the ".to_string()
                + DEFAULT_OPTS_ENV_NAME
                + " environment variable",
        )
        .author(crate_authors!(", "))
        .about(crate_description!())
        .version(crate_version!());

    if tree_arg_present() {
        command = command.allow_missing_positional(true);
    }

    command = add_expressions(command);
    command = add_targets(command);

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

fn usize_arg(info: &ArgInfo, requires_expr: bool) -> Arg {
    let mut arg = Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .value_name("")
        .action(ArgAction::Set);
    if requires_expr {
        arg = arg.requires(EXPRESSION_GROUP_ID);
    }
    arg
}

fn get_args() -> Vec<Arg> {
    let tree = Arg::new(TREE.id)
        .long(TREE.id)
        .help("display the files that would be search in tree format")
        .action(ArgAction::SetTrue)
        .conflicts_with(EXPRESSION_GROUP_ID)
        .short('l');

    let glob = Arg::new(GLOB.id)
        .long(GLOB.id)
        .help(GLOB.h)
        .value_name("")
        .action(ArgAction::Append);

    let searcher = Arg::new(SEARCHER.id)
        .long(SEARCHER.id)
        .short(SEARCHER.s.unwrap())
        .help(format!("executable to do the searching"))
        .value_parser([
            PossibleValue::new(names::RIPGREP_BIN).hide(false),
            PossibleValue::new(names::TREEGREP_BIN).hide(false),
            PossibleValue::new(names::RIPGREP).hide(true),
            PossibleValue::new(names::TREEGREP).hide(true),
        ])
        .value_name("")
        .conflicts_with(TREE.id)
        .action(ArgAction::Set);

    [
        bool_arg(SHOW_COUNT, true),
        bool_arg(HIDDEN, false),
        bool_arg(LINE_NUMBER, false),
        bool_arg(MENU, false),
        bool_arg(FILES, true),
        bool_arg(LINKS, false),
        bool_arg(TRIM_LEFT, true),
        bool_arg(PCRE2, true),
        bool_arg(NO_IGNORE, false),
        bool_arg(NO_COLORS, false),
        usize_arg(&MAX_DEPTH, false),
        usize_arg(&THREADS, false),
        usize_arg(&MAX_LENGTH, true),
        searcher,
        tree,
        glob,
    ]
    .to_vec()
}

fn add_expressions(command: Command) -> Command {
    command
        .arg(
            Arg::new(EXPRESSION_POSITIONAL.id)
                .help(EXPRESSION_POSITIONAL.h)
                .required_unless_present_any([TREE.id, EXPRESSION.id])
                .index(1),
        )
        .arg(
            Arg::new(EXPRESSION.id)
                .long(EXPRESSION.id)
                .short(EXPRESSION.s.unwrap())
                .help(EXPRESSION.h)
                .value_name("")
                .required_unless_present_any([TREE.id, EXPRESSION_POSITIONAL.id])
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
            Arg::new(TARGET_POSITIONAL.id)
                .help(TARGET_POSITIONAL.h)
                .value_hint(ValueHint::AnyPath)
                .index(2),
        )
        .arg(
            Arg::new(TARGET.id)
                .long(TARGET.id)
                .short(TARGET.s.unwrap())
                .help(TARGET.h)
                .value_name("")
                .value_hint(ValueHint::AnyPath),
        )
        .group(
            ArgGroup::new("target_group")
                .id("targets")
                .args([TARGET_POSITIONAL.id, TARGET.id])
                .multiple(false),
        )
}
