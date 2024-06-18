// SPDX-License-Identifier: CC-BY-4.0

use crate::args::{self, generate_command};
use crate::errors::{bail, Message};
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
}

pub struct Config {
    pub cwd: PathBuf,
    pub path: PathBuf,
    pub tree: bool,
    pub is_dir: bool,
    pub patterns: Vec<String>,
    pub globs: Vec<String>,
    pub searcher: Searchers,
    pub colors: bool,
    pub count: bool,
    pub hidden: bool,
    pub line_number: bool,
    pub menu: bool,
    pub just_files: bool,
    pub links: bool,
    pub ignore: bool,
    pub pcre2: bool,
    pub max_depth: Option<usize>,
    pub threads: Option<usize>,
    pub max_length: Option<usize>,
    pub trim: bool,
    pub c: Characters,
}

pub fn canonicalize(p: PathBuf) -> Result<PathBuf, Message> {
    dunce::canonicalize(&p).map_err(|_| {
        bail!(
            "failed to canonicalize path `{}`",
            p.to_string_lossy().to_owned()
        )
    })
}

fn get_usize_option(matches: &ArgMatches, name: &str) -> Result<Option<usize>, Message> {
    matches.get_one::<String>(name).map_or(Ok(None), |s| {
        s.parse::<usize>().map(Some).map_err(|_| {
            bail!(
                "failed to parse `{}` to a usize for option `{}`",
                s.to_string(),
                name.to_string()
            )
        })
    })
}

impl Config {
    pub fn get_matches() -> ArgMatches {
        let mut all_args: Vec<OsString> = std::env::var(args::DEFAULT_OPTS_ENV_NAME)
            .ok()
            .filter(|val| !val.is_empty())
            .map(|val| val.split_whitespace().map(OsString::from).collect())
            .unwrap_or_default();

        let mut args_os = std::env::args_os();
        if let Some(cmd) = args_os.next() {
            all_args.insert(0, cmd);
        }
        all_args.extend(args_os);

        generate_command().get_matches_from(all_args)
    }

    pub fn use_color(matches: &ArgMatches) -> bool {
        !matches.get_flag(args::NO_COLORS.id)
    }

    pub fn get_config(
        matches: ArgMatches,
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

        let tree: bool = matches.get_flag(args::TREE.id);
        let count: bool = matches.get_flag(args::SHOW_COUNT.id);
        let hidden: bool = matches.get_flag(args::HIDDEN.id);
        let line_number: bool = matches.get_flag(args::LINE_NUMBER.id);
        let menu: bool = matches.get_flag(args::MENU.id);
        let just_files: bool = matches.get_flag(args::FILES.id);
        let links: bool = matches.get_flag(args::LINKS.id);
        let trim: bool = matches.get_flag(args::TRIM_LEFT.id);
        let pcre2: bool = matches.get_flag(args::PCRE2.id);
        let ignore: bool = !matches.get_flag(args::NO_IGNORE.id);

        let max_depth: Option<usize> = get_usize_option(&matches, args::MAX_DEPTH.id)?;
        let threads: Option<usize> = get_usize_option(&matches, args::THREADS.id)?;
        let max_length: Option<usize> = get_usize_option(&matches, args::MAX_LENGTH.id)?;

        let (searcher, searcher_path) =
            Searchers::get_searcher(matches.get_one::<String>(args::SEARCHER.id))?;

        if let Searchers::TreeGrep = searcher {
            if threads.map_or(false, |t| t > 1) {
                return Err(bail!("treegrep searcher does not support multithreading"));
            }
        }

        if let Searchers::TreeGrep = searcher {
            if pcre2 {
                return Err(bail!("treegrep searcher does not support pcre2"));
            }
        }

        let target: Option<String> = matches
            .get_one::<String>(args::TARGET_POSITIONAL.id)
            .or_else(|| matches.get_one::<String>(args::TARGET.id))
            .map(|value| value.to_string());

        let cwd = canonicalize(
            std::env::current_dir()
                .map_err(|_| bail!("failed to get current working directory"))?,
        )?;

        let path = if let Some(target) = target {
            let path = PathBuf::from(target);
            if !path.exists() {
                return Err(bail!(
                    "failed to find path `{}`",
                    path.to_string_lossy().to_string()
                ));
            }
            canonicalize(path)?
        } else {
            cwd.clone()
        };

        let is_dir = path.is_dir();

        Ok((
            Config {
                cwd,
                path,
                tree,
                is_dir,
                searcher,
                patterns,
                line_number,
                colors,
                pcre2,
                count,
                hidden,
                menu,
                just_files,
                links,
                max_depth,
                threads,
                trim,
                globs,
                ignore,
                max_length,
                c: Config::get_characters(
                    matches.get_one::<String>(args::BOX_CHARS.id),
                    get_usize_option(&matches, args::PREFIX_LEN.id)?,
                ),
            },
            searcher_path,
        ))
    }

    fn get_characters(t: Option<&String>, pl: Option<usize>) -> Characters {
        let chars = match t.map(|s| s.as_str()) {
            Some("single") => formats::SINGLE,
            Some("double") => formats::DOUBLE,
            Some("heavy") => formats::HEAVY,
            Some("rounded") => formats::ROUNDED,
            Some("none") => formats::NONE,
            _ => formats::SINGLE,
        };
        let spacer: usize = pl.unwrap_or(formats::PREFIX_LEN_DEFAULT);
        Characters {
            bl: chars.bl,
            br: chars.br,
            tl: chars.tl,
            tr: chars.tr,
            v: chars.v,
            h: chars.h,
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
    use crate::args::names;

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

    #[test]
    fn test_default_opts() {
        std::env::set_var(args::DEFAULT_OPTS_ENV_NAME, EXAMPLE_LONG_OPTS.join(" "));
        let (config, _) = Config::get_config(Config::get_matches(), true)
            .ok()
            .unwrap();
        check_parsed_config_from_default_opts(config);
    }

    #[test]
    fn test_longs() {
        let (config, _) = Config::get_config(
            generate_command()
                .get_matches_from([&[names::TREEGREP_BIN], EXAMPLE_LONG_OPTS].concat()),
            true,
        )
        .ok()
        .unwrap();
        check_parsed_config_from_default_opts(config);
    }

    fn check_parsed_config_from_default_opts(config: Config) {
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
        assert!(config.just_files);
        match config.searcher {
            Searchers::RipGrep => {}
            _ => panic!("wrong searcher"),
        }
        assert_eq!(config.patterns, vec!["posexpr", "pattern1", "pattern2"]);
    }

    #[test]
    fn test_shorts() {
        let (config, _) = Config::get_config(
            generate_command().get_matches_from([
                names::TREEGREP_BIN,
                "posexpr",
                "-n.cmf",
                "-e=pattern1",
                "-e=pattern2",
            ]),
            true,
        )
        .ok()
        .unwrap();
        assert!(config.line_number);
        assert!(config.hidden);
        assert!(config.count);
        assert!(config.menu);
        assert!(config.just_files);
        assert_eq!(config.patterns, vec!["posexpr", "pattern1", "pattern2"]);
    }

    #[test]
    fn test_longs_tree() {
        let (config, _) = Config::get_config(
            generate_command().get_matches_from([
                names::TREEGREP_BIN,
                "--tree",
                "--max-depth=5",
                "--no-ignore",
                "--hidden",
                "--links",
                "--menu",
            ]),
            true,
        )
        .ok()
        .unwrap();
        assert_eq!(config.max_depth, Some(5));
        assert!(!config.ignore);
        assert!(config.hidden);
        assert!(config.tree);
        assert!(config.links);
        assert!(config.colors);
        assert!(config.menu);
        assert!(generate_command()
            .try_get_matches_from([names::TREEGREP_BIN, "--tree", "posexpr"])
            .is_err());
    }

    #[test]
    fn test_shorts_tree() {
        let (config, _) = Config::get_config(
            generate_command().get_matches_from([names::TREEGREP_BIN, "-.lm"]),
            true,
        )
        .ok()
        .unwrap();
        assert!(config.tree);
        assert!(config.hidden);
        assert!(config.menu);
        assert!(generate_command()
            .try_get_matches_from([names::TREEGREP_BIN, "posexpr", "-l"])
            .is_err());
    }
}
