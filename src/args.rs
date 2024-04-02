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

// TODO Options to Support
// excluding directories explicitly --exclude=path which accepts multiple
// then check if it hard to pass to rg though, so we should implement
// same frontend as rg
// `export FZF_DEFAULT_COMMAND="rg --files --hidden --follow --glob '!.git'"`
// could just do the exclude thing and then do --glob '!{path given}'
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
pub const COLORS_ALWAYS: &str = "always";
pub const COLORS_NEVER: &str = "never";
arg_info!(COLORS, "color", "set whether to color output");
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

const HELP: &str = "{name} {version}
by {author}

{about}

{usage}

{all-args}";

const MENU_HELP: &str = "open results in a menu to be edited with $EDITOR
navigate through the menu using the following commands:
\t- move up/down: k/j, p/n, up arrow/down arrow
\t- move up/down with a bigger jump: K/J, P/N
\t- move up/down paths: {/}, [/]
\t- move to the start/end: g/G, </>, home/end
\t- move up/down a page: ctrl + b/ctrl + f, pageup/pagedown";

pub fn generate_command() -> Command {
    let mut command = Command::new(crate_name!())
        .help_template(HELP.to_owned())
        .author(crate_authors!(", "))
        .about(crate_description!())
        .version(crate_version!());

    if std::env::args().any(|arg| {
        arg == format!("--{}", TREE.id)
            || (arg.starts_with('-')
                && arg.chars().nth(1) != Some('-')
                && arg.chars().skip(1).any(|c| c == TREE.s.unwrap()))
    }) {
        command = command.allow_missing_positional(true);
    }

    command = add_expressions(command);
    command = add_targets(command);

    let arg = Arg::new(TREE.id)
        .long(TREE.id)
        .help("display the files that would be search in tree format")
        .action(ArgAction::SetTrue)
        .conflicts_with(EXPRESSION_GROUP_ID)
        .short('l');
    command = command.arg(arg);

    for opt in get_args() {
        command = command.arg(opt);
    }
    return command;
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

fn set_arg(info: &ArgInfo, requires_expr: bool) -> Arg {
    let mut arg = Arg::new(info.id)
        .long(info.id)
        .help(info.h)
        .action(ArgAction::Set);
    if requires_expr {
        arg = arg.requires(EXPRESSION_GROUP_ID);
    }
    arg
}

fn get_args() -> Vec<Arg> {
    let color = Arg::new(COLORS.id)
        .long(COLORS.id)
        .help(COLORS.h)
        .value_parser([
            PossibleValue::new(COLORS_ALWAYS),
            PossibleValue::new(COLORS_NEVER),
        ]);
    let searcher = Arg::new(SEARCHER.id)
        .long(SEARCHER.id)
        .short(SEARCHER.s.unwrap())
        .help(format!(
            "executable to do the searching, currently supports {} and {}",
            names::RIPGREP_BIN,
            names::TREEGREP_BIN
        ))
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
        set_arg(&MAX_DEPTH, false),
        set_arg(&THREADS, false),
        set_arg(&MAX_LENGTH, true),
        color,
        searcher,
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
                .value_hint(ValueHint::AnyPath),
        )
        .group(
            ArgGroup::new("target_group")
                .id("targets")
                .args([TARGET_POSITIONAL.id, TARGET.id])
                .multiple(false),
        )
}
