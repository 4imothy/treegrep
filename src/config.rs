// SPDX-License-Identifier: MIT

use crate::args::{self, generate_command};
use crate::errors::{mes, Message};
use crate::formats;
use crate::searchers::Searchers;
use clap::ArgMatches;
use dunce;
use std::ffi::OsString;
use std::path::PathBuf;

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

pub struct Config {
    pub cwd: PathBuf,
    pub path: PathBuf,
    pub long_branch: bool,
    pub bold: bool,
    pub colors: bool,
    pub is_dir: bool,
    pub patterns: Vec<String>,
    pub globs: Vec<String>,
    pub searcher: Searchers,
    pub count: bool,
    pub hidden: bool,
    pub line_number: bool,
    pub menu: bool,
    pub files: bool,
    pub just_files: bool,
    pub links: bool,
    pub ignore: bool,
    pub pcre2: bool,
    pub max_depth: Option<usize>,
    pub threads: Option<usize>,
    pub max_length: Option<usize>,
    pub prefix_len: usize,
    pub long_branch_each: usize,
    pub trim: bool,
    pub c: Characters,
}

pub fn canonicalize(p: PathBuf) -> Result<PathBuf, Message> {
    dunce::canonicalize(&p).map_err(|_| {
        mes!(
            "failed to canonicalize path `{}`",
            p.to_string_lossy().to_owned()
        )
    })
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

impl Config {
    pub fn get_matches_from(args: Vec<OsString>) -> ArgMatches {
        let mut full_args = args; // TODO can probably do a check for completions here than reading
                                  // from the cmdline
        if !args::completions_arg_present() {
            if let Some(env_args) = std::env::var_os(args::DEFAULT_OPTS_ENV_NAME) {
                if !env_args.is_empty() {
                    full_args.splice(
                        0..0,
                        env_args
                            .into_string()
                            .unwrap_or_default()
                            .split_whitespace()
                            .map(OsString::from)
                            .collect::<Vec<_>>(),
                    );
                }
            }
        }

        generate_command().get_matches_from(full_args)
    }

    pub fn get_matches() -> ArgMatches {
        Self::get_matches_from(std::env::args_os().skip(1).collect())
    }

    pub fn get_styling(matches: &ArgMatches) -> (bool, bool) {
        (
            !matches.get_flag(args::NO_BOLD.id),
            !matches.get_flag(args::NO_COLORS.id),
        )
    }

    pub fn get_config(
        matches: ArgMatches,
        bold: bool,
        colors: bool,
    ) -> Result<(Self, Option<OsString>), Message> {
        let mut patterns: Vec<String> = Vec::new();
        if let Some(expr) = matches.get_one::<String>(args::EXPRESSION_POSITIONAL.id) {
            patterns.push(expr.to_owned());
        }
        if let Some(exprs) = matches.get_many::<String>(args::EXPRESSION.id) {
            for expr in exprs.into_iter() {
                patterns.push(expr.to_owned());
            }
        }

        let globs: Vec<String> = matches
            .get_many::<String>(args::GLOB.id)
            .map(|exprs| exprs.map(String::to_owned).collect())
            .unwrap_or_else(Vec::new);

        let long_branch: bool = matches.get_flag(args::LONG_BRANCHES.id);
        let count: bool = matches.get_flag(args::SHOW_COUNT.id);
        let hidden: bool = matches.get_flag(args::HIDDEN.id);
        let line_number: bool = matches.get_flag(args::LINE_NUMBER.id);
        let menu: bool = matches.get_flag(args::MENU.id);
        let files: bool = matches.get_flag(args::FILES.id);
        let links: bool = matches.get_flag(args::LINKS.id);
        let trim: bool = matches.get_flag(args::TRIM_LEFT.id);
        let pcre2: bool = matches.get_flag(args::PCRE2.id);
        let ignore: bool = !matches.get_flag(args::NO_IGNORE.id);

        let max_depth: Option<usize> = get_usize_option(&matches, args::MAX_DEPTH.id)?;
        let threads: Option<usize> = get_usize_option(&matches, args::THREADS.id)?;
        let max_length: Option<usize> = get_usize_option(&matches, args::MAX_LENGTH.id)?;
        let long_branch_each: usize =
            get_usize_option_with_default(&matches, args::LONG_BRANCHES_EACH.id)?;

        let (searcher, searcher_path) =
            Searchers::get_searcher(matches.get_one::<String>(args::SEARCHER.id))?;

        if let Searchers::TreeGrep = searcher {
            if threads.map_or(false, |t| t > 1) {
                return Err(mes!("treegrep searcher does not support multithreading"));
            }
        }

        if let Searchers::TreeGrep = searcher {
            if pcre2 {
                return Err(mes!("treegrep searcher does not support pcre2"));
            }
        }

        let path: Option<String> = matches
            .get_one::<String>(args::PATH_POSITIONAL.id)
            .or_else(|| matches.get_one::<String>(args::PATH.id))
            .map(|value| value.to_string());

        let cwd = canonicalize(
            std::env::current_dir().map_err(|_| mes!("failed to get current working directory"))?,
        )?;

        let path = if let Some(p) = path {
            let path = PathBuf::from(p);
            if !path.exists() {
                return Err(mes!(
                    "failed to find path `{}`",
                    path.to_string_lossy().to_string()
                ));
            }
            canonicalize(path)?
        } else {
            cwd.clone()
        };

        let is_dir = path.is_dir();
        let prefix_len = get_usize_option_with_default(&matches, args::PREFIX_LEN.id)?;
        let just_files = files && patterns.len() == 0;

        Ok((
            Config {
                cwd,
                path,
                long_branch,
                is_dir,
                just_files,
                bold,
                searcher,
                patterns,
                line_number,
                colors,
                pcre2,
                count,
                hidden,
                menu,
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
                c: Config::get_characters(
                    matches.get_one::<String>(args::CHAR_STYLE.id),
                    prefix_len,
                ),
            },
            searcher_path,
        ))
    }

    fn get_characters(t: Option<&String>, spacer: usize) -> Characters {
        let chars = match t.map(|s| s.as_str()) {
            Some("single") | None => formats::SINGLE,
            Some("ascii") => formats::ASCII,
            Some("double") => formats::DOUBLE,
            Some("heavy") => formats::HEAVY,
            Some("rounded") => formats::ROUNDED,
            Some("none") => formats::NONE,
            _ => panic!(
                "{} option {} not implemented",
                args::CHAR_STYLE.id,
                t.unwrap()
            ),
        };
        Characters {
            bl: chars.bl,
            br: chars.br,
            tl: chars.tl,
            tr: chars.tr,
            v: chars.v,
            h: chars.h,
            selected_indicator: chars.selected_indicator,
            match_with_next: format!("{}{}", chars.tee, formats::repeat(chars.h, spacer - 1),),
            match_no_next: format!("{}{}", chars.bl, formats::repeat(chars.h, spacer - 1),),
            spacer_vert: format!("{}{}", chars.v, formats::repeat(' ', spacer - 1)),
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
        "--menu",
        "--files",
        "--searcher=rg",
        "--regexp=pattern1",
        "--regexp=pattern2",
    ];

    pub fn get_config_from<I, T>(args: I) -> Config
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = generate_command().get_matches_from(args);
        let (bold, colors) = Config::get_styling(&matches);
        let (config, _) = Config::get_config(matches, bold, colors).ok().unwrap();
        config
    }

    #[test]
    fn test_env_opts() {
        std::env::set_var(args::DEFAULT_OPTS_ENV_NAME, EXAMPLE_LONG_OPTS.join(" "));
        let matches = Config::get_matches();
        let (bold, colors) = Config::get_styling(&matches);
        let (config, _) = Config::get_config(matches, bold, colors).ok().unwrap();
        check_parsed_config_from_example_opts(config);
    }

    #[test]
    fn test_default_opts() {
        let config = get_config_from(["expression"]);
        assert!(
            config.c.spacer == " ".repeat(args::DEFAULT_PREFIX_LEN.parse::<usize>().ok().unwrap())
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
        assert!(config.colors);
        assert!(config.menu);
        assert!(config.files);
        match config.searcher {
            Searchers::RipGrep => {}
            _ => panic!("wrong searcher"),
        }
        assert_eq!(config.patterns, vec!["posexpr", "pattern1", "pattern2"]);
    }

    #[test]
    fn test_shorts() {
        let config = get_config_from(["posexpr", "-n.cmf", "-e=pattern1", "-e=pattern2"]);
        assert!(config.line_number);
        assert!(config.hidden);
        assert!(config.count);
        assert!(config.menu);
        assert!(config.files);
        assert_eq!(config.patterns, vec!["posexpr", "pattern1", "pattern2"]);
    }

    #[test]
    fn test_longs_files() {
        let config = get_config_from([
            "--files",
            "--max-depth=5",
            "--no-ignore",
            "--hidden",
            "--links",
            "--menu",
        ]);
        assert_eq!(config.max_depth, Some(5));
        assert!(!config.ignore);
        assert!(config.hidden);
        assert!(config.links);
        assert!(config.colors);
        assert!(config.menu);
    }

    #[test]
    fn test_shorts_files() {
        let config = get_config_from(["-.fm"]);
        assert!(config.hidden);
        assert!(config.menu);
    }
}
