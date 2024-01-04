// SPDX-License-Identifier: CC-BY-4.0

use clap::builder::PossibleValue;
use clap::{
    crate_authors, crate_description, crate_name, crate_version, Arg, ArgAction, ArgGroup, Command,
    ValueHint,
};

pub mod names {
    pub const BIN_NAME: &str = "tgrep";
}

// TODO Options to Support
//
// excluding directories explicitly

pub mod arg_strs {
    pub const TARGET_POSITIONAL: &str = "target-positional";
    pub const TARGET: &str = "target";
    pub const EXPRESSION_POSITIONAL: &str = "regex expression-positional";
    pub const EXPRESSION: &str = "regex expression";
    pub const COLORS: &str = "color";
    pub const COLORS_ALWAYS: &str = "always";
    pub const COLORS_NEVER: &str = "never";
    pub const SHOW_COUNT: &str = "count";
    pub const HIDDEN: &str = "hidden";
    pub const LINE_NUMBER: &str = "line-number";
    pub const MENU: &str = "menu";
    pub const FILES: &str = "files";
    pub const MAX_DEPTH: &str = "max-depth";
    pub const SEARCHER: &str = "searcher";
    pub const LINKS: &str = "links";
    pub const TRIM_LEFT: &str = "trim";
    pub const PCRE2: &str = "pcre2";
    pub const THREADS: &str = "threads";
    pub const NO_IGNORE: &str = "no-ignore";
}

const HELP: &str = "{name} {version}
by {author}

{about}

{usage}

{all-args}";

pub fn generate_command() -> Command {
    let mut command = Command::new(crate_name!())
        .author(crate_authors!(", "))
        .about(crate_description!())
        .help_template(HELP.to_owned())
        .version(crate_version!());

    command = add_expressions(command);
    command = add_paths(command);
    for opt in get_args() {
        command = command.arg(opt);
    }
    return command;
}

fn create_bool_arg(long: &'static str, short: Option<char>, help: &'static str) -> Arg {
    let arg = Arg::new(long)
        .long(long)
        .help(help)
        .action(ArgAction::SetTrue);

    match short {
        Some(s) => arg.short(s),
        None => arg,
    }
}

fn create_set_arg(long: &'static str, help: &'static str) -> Arg {
    Arg::new(long).long(long).help(help).action(ArgAction::Set)
}

fn get_args() -> Vec<Arg> {
    vec![
    create_bool_arg(arg_strs::SHOW_COUNT, Some('c'), "display number of files matched in directory and number of lines matched in a file if present"),
    create_bool_arg(arg_strs::HIDDEN, Some('.'), "search hidden files"),
    create_bool_arg(arg_strs::LINE_NUMBER, Some('n'), "show line number of match if present"),
    create_bool_arg(arg_strs::MENU, Some('m'), "open results in a menu to be opened with $EDITOR, move with j/k, n/p, up/down"),
    create_bool_arg(arg_strs::FILES, Some('f'), "show the paths that have matches"),
    create_bool_arg(arg_strs::LINKS, None, "show linked paths for symbolic links"),
    create_bool_arg(arg_strs::TRIM_LEFT, None, "trim whitespace at beginning of lines"),
    create_bool_arg(arg_strs::PCRE2, None, "enable pcre2 if the searcher supports it"),
    create_bool_arg(arg_strs::NO_IGNORE, None, "don't use ignore files"),
    create_set_arg(arg_strs::MAX_DEPTH, "the max depth to search"),
    create_set_arg(arg_strs::THREADS, "set appropriate number of threads to use"),
    Arg::new(arg_strs::COLORS)
        .long(arg_strs::COLORS)
        .help("set whether to color output")
        .value_parser([
                      PossibleValue::new(arg_strs::COLORS_ALWAYS),
                      PossibleValue::new(arg_strs::COLORS_NEVER)
        ]),
    Arg::new(arg_strs::SEARCHER)
        .long(arg_strs::SEARCHER)
        .short('s')
        .help(format!("executable to do the searching, currently supports rg  and {}", names::BIN_NAME))
        .action(ArgAction::Set)
    ]
}

fn add_expressions(command: Command) -> Command {
    let help = "specify the regex expression";
    command
        .arg(
            Arg::new(arg_strs::EXPRESSION_POSITIONAL)
                .help(help)
                .index(1),
        )
        .arg(
            Arg::new(arg_strs::EXPRESSION)
                .short('e')
                .long("regexp")
                .help(help)
                .action(ArgAction::Append),
        )
        .group(
            ArgGroup::new("expression_group")
                .id("expressions")
                .args([arg_strs::EXPRESSION_POSITIONAL, arg_strs::EXPRESSION])
                .multiple(true)
                .required(true),
        )
}

fn add_paths(command: Command) -> Command {
    let help = "specify the search target. If none provided, search the current directory.";
    command
        .arg(
            Arg::new(arg_strs::TARGET_POSITIONAL)
                .help(help)
                .value_hint(ValueHint::AnyPath)
                .index(2),
        )
        .arg(
            Arg::new(arg_strs::TARGET)
                .short('t')
                .long(arg_strs::TARGET)
                .help(help)
                .value_hint(ValueHint::AnyPath),
        )
        .group(
            ArgGroup::new("target_group")
                .id("targets")
                .args([arg_strs::TARGET_POSITIONAL, arg_strs::TARGET]),
        )
}
