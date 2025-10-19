// SPDX-License-Identifier: MIT

use crate::{
    args::{self, OpenStrategy, REPEAT_FILE, generate_command},
    errors::Message,
    mes,
    searchers::Searchers,
    style,
};
use clap::{ArgMatches, Error};
use crossterm::style::Color;
use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

pub struct Characters {
    pub bl: char,
    pub br: char,
    pub tl: char,
    pub tr: char,
    pub v: char,
    pub h: char,
    pub match_with_next: String,
    pub match_no_next: String,
    pub spacer_vert: String,
    pub spacer: String,
    pub selected_indicator: &'static str,
}

pub struct Colors {
    pub file: Color,
    pub dir: Color,
    pub line_number: Color,
    pub text: Option<Color>,
    pub branch: Option<Color>,
    pub selected_indicator: Option<Color>,
    pub selected_bg: Color,
    pub matches: Vec<Color>,
}

impl args::Color {
    fn get(&self) -> Color {
        match *self {
            args::Color::Black => Color::Black,
            args::Color::White => Color::White,
            args::Color::Red => Color::Red,
            args::Color::Green => Color::Green,
            args::Color::Yellow => Color::Yellow,
            args::Color::Blue => Color::Blue,
            args::Color::Magenta => Color::Magenta,
            args::Color::Cyan => Color::Cyan,
            args::Color::Grey => Color::Grey,
            args::Color::Rgb(r, g, b) => Color::Rgb { r, g, b },
            args::Color::Ansi(value) => Color::AnsiValue(value),
        }
    }
}

pub struct Config {
    pub path: PathBuf,
    pub selection_file: Option<PathBuf>,
    pub repeat_file: Option<PathBuf>,
    pub long_branch: bool,
    pub with_bold: bool,
    pub with_colors: bool,
    pub is_dir: bool,
    pub regexps: Vec<String>,
    pub globs: Vec<String>,
    pub searcher: Searchers,
    pub searcher_path: Option<PathBuf>,
    pub count: bool,
    pub hidden: bool,
    pub line_number: bool,
    pub select: bool,
    pub menu: bool,
    pub files: bool,
    pub just_files: bool,
    pub overview: bool,
    pub links: bool,
    pub ignore: bool,
    pub pcre2: bool,
    pub max_depth: Option<usize>,
    pub threads: Option<usize>,
    pub max_length: Option<usize>,
    pub prefix_len: usize,
    pub long_branch_each: usize,
    pub trim: bool,
    pub editor: Option<String>,
    pub open_like: Option<OpenStrategy>,
    pub completion_target: Option<clap_complete::Shell>,
    pub repeat: bool,
    pub all_args: Vec<OsString>,
    pub chars: Characters,
    pub colors: Colors,
}

fn canonicalize(p: &Path) -> Result<PathBuf, Message> {
    dunce::canonicalize(p).map_err(|_| {
        mes!(
            "failed to canonicalize path `{}`",
            p.to_string_lossy().into_owned()
        )
    })
}

fn process_path(input: &Path, check_exists: bool) -> Result<PathBuf, Message> {
    let mut components = input.components();
    let mut path = PathBuf::new();
    match components.next() {
        Some(Component::Normal(c)) => {
            if c == "~" {
                path.push(std::env::var("HOME").map_err(|e| mes!("{}", e.to_string()))?);
            } else {
                path.push(c);
            }
        }
        Some(c) => path.push(c),
        _ => {}
    }

    for c in components {
        path.push(c);
    }
    if check_exists {
        path.exists()
            .then(|| canonicalize(&path))
            .ok_or_else(|| mes!("failed to find path `{}`", path.to_string_lossy()))?
    } else {
        Ok(path)
    }
}

fn get_usize_option(matches: &ArgMatches, name: &str) -> Result<Option<usize>, Message> {
    matches.get_one::<String>(name).map_or(Ok(None), |s| {
        s.parse::<usize>().map(Some).map_err(|_| {
            mes!(
                "failed to parse `{}` to a usize for option `{}`",
                s.to_string(),
                name.to_string()
            )
        })
    })
}

fn get_usize_option_with_default(matches: &ArgMatches, name: &str) -> Result<usize, Message> {
    Ok(get_usize_option(matches, name)?.unwrap())
}

fn get_all_args(mut args: Vec<OsString>) -> Vec<OsString> {
    if let Some(env_args) = std::env::var_os(args::DEFAULT_OPTS_ENV_NAME) {
        args.splice(
            0..0,
            env_args
                .into_string()
                .unwrap_or_default()
                .split_whitespace()
                .map(OsString::from),
        );
    }
    args
}

pub fn get_matches(
    args: Vec<OsString>,
    with_env: bool,
) -> Result<(ArgMatches, Vec<OsString>), Error> {
    let all_args = if with_env { get_all_args(args) } else { args };
    generate_command()
        .try_get_matches_from(&all_args)
        .map(|m| (m, all_args))
}

impl Config {
    pub fn get_styling(matches: &ArgMatches) -> (bool, bool) {
        (
            !matches.get_flag(args::NO_BOLD.id),
            !matches.get_flag(args::NO_COLORS.id),
        )
    }

    pub fn handle_repeat(&self) -> Result<Option<Self>, Message> {
        if let Some(f) = &self.repeat_file {
            if self.repeat {
                let data = std::fs::read(f).map_err(|e| mes!("{}", e))?;
                unsafe {
                    let mut pos = 0;
                    let mut args = Vec::new();
                    while pos < data.len() {
                        let len = u32::from_le_bytes(
                            data[pos..pos + size_of::<u32>()].try_into().unwrap(),
                        ) as usize;
                        pos += size_of::<u32>();
                        let bytes = &data[pos..pos + len];
                        pos += len;
                        args.push(OsString::from_encoded_bytes_unchecked(bytes.to_vec()));
                    }
                    let (matches, all_args) =
                        get_matches(args, false).map_err(|e| mes!("{}", e))?;
                    let (bold, colors) = Self::get_styling(&matches);

                    Ok(Some(Self::get_config(matches, all_args, bold, colors)?))
                }
            } else if self.menu {
                Ok(None)
            } else {
                let mut buffer = Vec::new();
                for arg in self.all_args.iter() {
                    let bytes = arg.as_encoded_bytes();
                    let len = bytes.len() as u32;
                    buffer.extend_from_slice(&len.to_le_bytes());
                    buffer.extend_from_slice(bytes);
                }
                std::fs::write(f, buffer).map_err(|e| mes!("{}", e))?;
                Ok(None)
            }
        } else if self.repeat {
            Err(mes!("cannot repeat without a {} specified", REPEAT_FILE.id))
        } else {
            Ok(None)
        }
    }

    pub fn get_config(
        matches: ArgMatches,
        all_args: Vec<OsString>,
        bold: bool,
        colors: bool,
    ) -> Result<Self, Message> {
        let mut regexps: Vec<String> = Vec::new();
        if let Some(expr) = matches.get_one::<String>(args::EXPRESSION_POSITIONAL.id) {
            regexps.push(expr.to_owned());
        }
        if let Some(exprs) = matches.get_many::<String>(args::EXPRESSION.id) {
            for expr in exprs.into_iter() {
                regexps.push(expr.to_owned());
            }
        }

        let globs: Vec<String> = matches
            .get_many::<String>(args::GLOB.id)
            .map(|exprs| exprs.map(String::to_owned).collect())
            .unwrap_or_default();

        let long_branch: bool = matches.get_flag(args::LONG_BRANCHES.id);
        let count: bool = matches.get_flag(args::COUNT.id);
        let hidden: bool = matches.get_flag(args::HIDDEN.id);
        let line_number: bool = matches.get_flag(args::LINE_NUMBER.id);
        let files: bool = matches.get_flag(args::FILES.id);
        let links: bool = matches.get_flag(args::LINKS.id);
        let trim: bool = matches.get_flag(args::TRIM_LEFT.id);
        let pcre2: bool = matches.get_flag(args::PCRE2.id);
        let ignore: bool = !matches.get_flag(args::NO_IGNORE.id);
        let overview: bool = matches.get_flag(args::OVERVIEW.id);
        let repeat: bool = matches.get_flag(args::REPEAT.id);
        let menu: bool = matches.get_flag(args::MENU.id);
        let select: bool = matches.get_flag(args::SELECT.id);

        let max_depth: Option<usize> = get_usize_option(&matches, args::MAX_DEPTH.id)?;
        let threads: Option<usize> = get_usize_option(&matches, args::THREADS.id)?;
        let max_length: Option<usize> = get_usize_option(&matches, args::MAX_LENGTH.id)?;
        let long_branch_each: usize =
            get_usize_option_with_default(&matches, args::LONG_BRANCHES_EACH.id)?;

        let (searcher, searcher_path) =
            Searchers::get_searcher_and_path(matches.get_one::<String>(args::SEARCHER.id))?;

        if let Searchers::TreeGrep = searcher
            && threads.is_some_and(|t| t > 1)
        {
            return Err(mes!("treegrep searcher does not support multithreading"));
        }

        if let Searchers::TreeGrep = searcher
            && pcre2
        {
            return Err(mes!("treegrep searcher does not support pcre2"));
        }

        let editor = matches.get_one::<String>(args::EDITOR.id).cloned();
        let open_like = matches.get_one::<OpenStrategy>(args::OPEN_LIKE.id).cloned();

        let path = matches
            .get_one::<PathBuf>(args::PATH_POSITIONAL.id)
            .or_else(|| matches.get_one::<PathBuf>(args::PATH.id));

        let path = if let Some(p) = path {
            process_path(p, true)?
        } else {
            canonicalize(
                &std::env::current_dir()
                    .map_err(|_| mes!("failed to get current working directory"))?,
            )?
        };

        let selection_file = matches
            .get_one::<PathBuf>(args::SELECTION_FILE.id)
            .map(|p| process_path(p, false))
            .transpose()?;
        let repeat_file = matches
            .get_one::<PathBuf>(args::REPEAT_FILE.id)
            .map(|p| process_path(p, false))
            .transpose()?;

        let is_dir = path.is_dir();
        let prefix_len = get_usize_option_with_default(&matches, args::PREFIX_LEN.id)?;
        let just_files = files && regexps.is_empty();

        let completion_target = matches
            .get_one::<clap_complete::Shell>(args::COMPLETIONS.id)
            .copied();

        Ok(Config {
            path,
            selection_file,
            repeat_file,
            long_branch,
            is_dir,
            just_files,
            with_bold: bold,
            searcher,
            searcher_path,
            regexps,
            line_number,
            with_colors: colors,
            pcre2,
            count,
            hidden,
            select,
            files,
            links,
            max_depth,
            threads,
            trim,
            globs,
            ignore,
            prefix_len,
            max_length,
            long_branch_each,
            editor,
            open_like,
            overview,
            completion_target,
            menu,
            repeat,
            all_args,
            chars: Config::get_characters(
                matches.get_one::<args::CharacterStyle>(args::CHAR_STYLE.id),
                prefix_len,
            ),
            colors: Config::get_colors(&matches),
        })
    }

    fn get_colors(matches: &ArgMatches) -> Colors {
        Colors {
            file: matches
                .get_one::<args::Color>(args::FILE_COLOR.id)
                .map(|v| v.get())
                .unwrap_or(style::FILE_COLOR_DEFAULT),
            dir: matches
                .get_one::<args::Color>(args::DIR_COLOR.id)
                .map(|v| v.get())
                .unwrap_or(style::DIR_COLOR_DEFAULT),
            line_number: matches
                .get_one::<args::Color>(args::LINE_NUMBER_COLOR.id)
                .map(|v| v.get())
                .unwrap_or(style::LINE_NUMBER_COLOR_DEFAULT),
            text: matches
                .get_one::<args::Color>(args::TEXT_COLOR.id)
                .map(|v| v.get()),
            branch: matches
                .get_one::<args::Color>(args::BRANCH_COLOR.id)
                .map(|v| v.get()),
            selected_bg: matches
                .get_one::<args::Color>(args::SELECTED_BG_COLOR.id)
                .map(|v| v.get())
                .unwrap_or(style::SELECTED_BG_DEFAULT),
            selected_indicator: matches
                .get_one::<args::Color>(args::SELECTED_INDICATOR_COLOR.id)
                .map(|v| v.get()),
            matches: matches
                .get_many::<args::Color>(args::MATCH_COLORS.id)
                .map(|vals| vals.cloned().map(|v| v.get()).collect::<Vec<_>>())
                .unwrap_or_else(|| style::MATCHED_COLORS_DEFAULT.to_vec()),
        }
    }

    fn get_characters(t: Option<&args::CharacterStyle>, spacer: usize) -> Characters {
        let chars = t.map_or(style::SINGLE, |c| match c {
            args::CharacterStyle::Single => style::SINGLE,
            args::CharacterStyle::Ascii => style::ASCII,
            args::CharacterStyle::Double => style::DOUBLE,
            args::CharacterStyle::Heavy => style::HEAVY,
            args::CharacterStyle::Rounded => style::ROUNDED,
            args::CharacterStyle::None => style::NONE,
        });
        Characters {
            bl: chars.bl,
            br: chars.br,
            tl: chars.tl,
            tr: chars.tr,
            v: chars.v,
            h: chars.h,
            selected_indicator: chars.selected_indicator,
            match_with_next: format!("{}{}", chars.tee, style::repeat(chars.h, spacer - 1),),
            match_no_next: format!("{}{}", chars.bl, style::repeat(chars.h, spacer - 1),),
            spacer_vert: format!("{}{}", chars.v, style::repeat(' ', spacer - 1)),
            spacer: " ".repeat(spacer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args;

    static EXAMPLE_LONG_OPTS: &[&str] = &[
        "posexpr",
        "--line-number",
        "--max-depth=5",
        "--max-length=20",
        "--pcre2",
        "--no-ignore",
        "--hidden",
        "--threads=8",
        "--count",
        "--links",
        "--trim",
        "--select",
        "--files",
        "--searcher=rg",
        "--regexp=regexp1",
        "--regexp=regexp2",
    ];

    pub fn get_config_from<I, T>(args: I) -> Config
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = generate_command().get_matches_from(args);
        let (bold, colors) = Config::get_styling(&matches);
        let config = Config::get_config(matches, Vec::new(), bold, colors)
            .ok()
            .unwrap();
        config
    }

    #[test]
    fn test_env_opts() {
        unsafe { std::env::set_var(args::DEFAULT_OPTS_ENV_NAME, EXAMPLE_LONG_OPTS.join(" ")) };
        let (matches, all_args) = get_matches(Vec::new(), true).unwrap();
        let (bold, colors) = Config::get_styling(&matches);
        let config = Config::get_config(matches, all_args, bold, colors)
            .ok()
            .unwrap();
        check_parsed_config_from_example_opts(config);
    }

    #[test]
    fn test_default_opts() {
        let config = get_config_from(["expression"]);
        assert!(
            config.chars.spacer
                == " ".repeat(args::DEFAULT_PREFIX_LEN.parse::<usize>().ok().unwrap())
        );
        assert!(
            config.long_branch_each
                == args::DEFAULT_LONG_BRANCH_EACH
                    .parse::<usize>()
                    .ok()
                    .unwrap()
        );
    }

    #[test]
    fn test_longs() {
        let config = get_config_from(EXAMPLE_LONG_OPTS);
        check_parsed_config_from_example_opts(config);
    }

    fn check_parsed_config_from_example_opts(config: Config) {
        assert!(config.line_number);
        assert_eq!(config.max_depth, Some(5));
        assert_eq!(config.max_length, Some(20));
        assert!(config.pcre2);
        assert!(!config.ignore);
        assert!(config.hidden);
        assert!(config.threads == Some(8));
        assert!(config.count);
        assert!(config.links);
        assert!(config.trim);
        assert!(config.with_colors);
        assert!(config.select);
        assert!(config.files);
        match config.searcher {
            Searchers::RipGrep => {}
            _ => panic!("wrong searcher"),
        }
        assert_eq!(config.regexps, vec!["posexpr", "regexp1", "regexp2"]);
    }

    #[test]
    fn test_shorts() {
        let config = get_config_from(["posexpr", "-n.csf", "-e=regexp1", "-e=regexp2"]);
        assert!(config.line_number);
        assert!(config.hidden);
        assert!(config.count);
        assert!(config.select);
        assert!(config.files);
        assert_eq!(config.regexps, vec!["posexpr", "regexp1", "regexp2"]);
    }

    #[test]
    fn test_longs_files() {
        let config = get_config_from([
            "--files",
            "--max-depth=5",
            "--no-ignore",
            "--hidden",
            "--links",
            "--select",
        ]);
        assert_eq!(config.max_depth, Some(5));
        assert!(!config.ignore);
        assert!(config.hidden);
        assert!(config.links);
        assert!(config.with_colors);
        assert!(config.select);
    }

    #[test]
    fn test_shorts_files() {
        let config = get_config_from(["-.fs"]);
        assert!(config.hidden);
        assert!(config.select);
    }
}
