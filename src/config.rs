// SPDX-License-Identifier: CC-BY-4.0

use crate::args::{arg_strs, generate_command};
use crate::errors::{bail, Message};
use crate::searchers::Searchers;
use clap::ArgMatches;
use dunce;
use std::ffi::OsString;
use std::path::PathBuf;

pub struct Config {
    pub path: PathBuf,
    pub is_dir: bool,
    pub patterns: Vec<String>,
    pub exec: Searchers,
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
        generate_command().get_matches()
    }

    pub fn get_colors(matches: &ArgMatches) -> bool {
        matches
            .get_one::<String>(arg_strs::COLORS)
            .map(|s| s == arg_strs::COLORS_ALWAYS)
            .unwrap_or(true)
    }

    pub fn get_config(
        matches: ArgMatches,
        colors: bool,
    ) -> Result<(Self, Option<OsString>), Message> {
        let mut patterns: Vec<String> = Vec::new();
        if let Some(expr) = matches.get_one::<String>(arg_strs::EXPRESSION_POSITIONAL) {
            patterns.push(expr.to_owned());
        }
        if let Some(exprs) = matches.get_many::<String>(arg_strs::EXPRESSION) {
            for e in exprs.into_iter() {
                patterns.push(e.to_owned());
            }
        }

        let count: bool = *matches.get_one::<bool>(arg_strs::SHOW_COUNT).unwrap();
        let hidden: bool = *matches.get_one::<bool>(arg_strs::HIDDEN).unwrap();
        let line_number: bool = *matches.get_one::<bool>(arg_strs::LINE_NUMBER).unwrap();
        let menu: bool = *matches.get_one::<bool>(arg_strs::MENU).unwrap();
        let just_files: bool = *matches.get_one::<bool>(arg_strs::FILES).unwrap();
        let links: bool = *matches.get_one::<bool>(arg_strs::LINKS).unwrap();
        let trim: bool = *matches.get_one::<bool>(arg_strs::TRIM_LEFT).unwrap();
        let pcre2: bool = *matches.get_one::<bool>(arg_strs::PCRE2).unwrap();
        let ignore: bool = !*matches.get_one::<bool>(arg_strs::NO_IGNORE).unwrap();

        let max_depth: Option<usize> = get_usize_option(&matches, arg_strs::MAX_DEPTH)?;
        let threads: Option<usize> = get_usize_option(&matches, arg_strs::THREADS)?;
        let max_length: Option<usize> = get_usize_option(&matches, arg_strs::MAX_LENGTH)?;

        let (exec, starter) =
            Searchers::get_searcher(matches.get_one::<String>(arg_strs::SEARCHER))?;

        let target: Option<String> = matches
            .get_one::<String>(arg_strs::TARGET_POSITIONAL)
            .or_else(|| matches.get_one::<String>(arg_strs::TARGET))
            .map(|value| value.to_string());

        let p = if let Some(target) = target {
            let path = PathBuf::from(target);
            if !path.exists() {
                return Err(bail!(
                    "failed to find path {}",
                    path.to_string_lossy().to_string()
                ));
            }
            path
        } else {
            std::env::current_dir().map_err(|_| bail!("failed to get current working directory"))?
        };

        let path = dunce::canonicalize(&p).map_err(|_| {
            bail!(
                "failed to canonicalize given path `{}`",
                p.to_string_lossy().to_owned()
            )
        })?;

        let is_dir = path.is_dir();

        Ok((
            Config {
                path,
                is_dir,
                exec,
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
                ignore,
                max_length,
            },
            starter,
        ))
    }
}
