// SPDX-License-Identifier: MIT

use crate::args::names;
use crate::config;
use crate::errors::{mes, Message, SUBMIT_ISSUE};
use crate::options::{Options, Rg};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_exe_path(bin: &str) -> Option<PathBuf> {
    env::var("PATH").ok().and_then(|path| {
        env::split_paths(&path).find_map(|mut p| {
            p.push(bin);
            if p.exists() {
                return Some(p);
            }
            if cfg!(windows) {
                p.set_extension(&env::consts::EXE_SUFFIX[1..]);
                if p.exists() {
                    return Some(p);
                }
            }
            None
        })
    })
}

pub enum Searchers {
    RipGrep,
    TreeGrep,
}

impl Searchers {
    pub fn get_searcher_and_path(
        chosen: Option<&String>,
    ) -> Result<(Self, Option<PathBuf>), Message> {
        match Searchers::from_str(chosen)? {
            Some(Searchers::TreeGrep) => Ok((Searchers::TreeGrep, None)),
            Some(Searchers::RipGrep) => match get_exe_path(names::RIPGREP_BIN) {
                Some(path) => Ok((Searchers::RipGrep, Some(path))),
                _ => Err(mes!("failed to find searcher `{}`", names::RIPGREP_BIN)),
            },
            None => {
                if let Some(p) = get_exe_path(names::RIPGREP_BIN) {
                    Ok((Searchers::RipGrep, Some(p)))
                } else {
                    Ok((Searchers::TreeGrep, None))
                }
            }
        }
    }

    fn from_str(chosen: Option<&String>) -> Result<Option<Searchers>, Message> {
        match chosen {
            Some(s) => match s.as_str() {
                s if s == names::TREEGREP
                    || s == names::TREEGREP_BIN
                    || (cfg!(windows)
                        && s == format!("{}{}", names::TREEGREP, env::consts::EXE_SUFFIX))
                    || (cfg!(windows)
                        && s == format!("{}{}", names::TREEGREP_BIN, env::consts::EXE_SUFFIX)) =>
                {
                    Ok(Some(Searchers::TreeGrep))
                }
                s if s == names::RIPGREP
                    || s == names::RIPGREP_BIN
                    || (cfg!(windows)
                        && s == format!("{}{}", names::RIPGREP, env::consts::EXE_SUFFIX))
                    || (cfg!(windows)
                        && s == format!("{}{}", names::RIPGREP_BIN, env::consts::EXE_SUFFIX)) =>
                {
                    Ok(Some(Searchers::RipGrep))
                }
                _ => Err(mes!("searcher `{}` is invalid", s)),
            },
            _ => Ok(None),
        }
    }

    pub fn generate_command(starter: &Path) -> Result<Command, Message> {
        let mut cmd = Command::new(starter);

        match config().searcher {
            Searchers::RipGrep => Rg::add_options(&mut cmd)?,
            Searchers::TreeGrep => panic!(
                "tried to use external command when using the treegrep searcher {SUBMIT_ISSUE}"
            ),
        }
        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::generate_command;
    use crate::config::Config;
    use crate::CONFIG;

    #[test]
    fn test_options_add_args_rg() {
        let mut cmd = Command::new("rg");
        let c = Config::get_config(
            generate_command().get_matches_from([
                "--regexp=pattern1",
                "--regexp=pattern2",
                "--glob=globbing",
                "--glob=glob2",
                "--line-number",
                "--max-depth=5",
                "--pcre2",
                "--no-ignore",
                "--hidden",
                "--threads=8",
                "--count",
                "--links",
                "--trim",
            ]),
            Vec::new(),
            false,
            false,
        )
        .ok()
        .unwrap();
        CONFIG.set(c).ok().unwrap();

        assert!(Rg::add_options(&mut cmd).is_ok());

        let expected_args = vec![
            "--json",
            "--regexp=pattern1",
            "--regexp=pattern2",
            config().path.to_str().unwrap(),
            "--glob=globbing",
            "--glob=glob2",
            "--color=never",
            "--line-number",
            "--pcre2",
            "--hidden",
            "--max-depth=5",
            "--threads=8",
            "--follow",
            "--no-ignore",
        ];

        assert_eq!(
            cmd.get_args()
                .take(cmd.get_args().len())
                .map(|s| s.to_str().unwrap())
                .collect::<Vec<&str>>(),
            expected_args
        );
    }
}
