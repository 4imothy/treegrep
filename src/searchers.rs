// SPDX-License-Identifier: MIT

use crate::args::names;
use crate::config;
use crate::errors::{mes, Message, SUBMIT_ISSUE};
use crate::options::{Options, Rg};
use std::env;
use std::ffi::OsString;
use std::ops::Deref;
use std::process::Command;

struct ShortName(String);

impl ShortName {
    fn new(name: &str) -> Self {
        assert!(name == names::TREEGREP_BIN || name == names::RIPGREP_BIN);
        ShortName(name.to_owned())
    }
}

impl std::fmt::Display for ShortName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ShortName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn get_exe_path(bin: &ShortName) -> Option<OsString> {
    env::var("PATH").ok().and_then(|path| {
        env::split_paths(&path).find_map(|mut p| {
            p.push(&bin.0);
            if p.exists() {
                return Some(p.into_os_string());
            }
            if cfg!(target_os = "windows") {
                p.set_extension(&env::consts::EXE_SUFFIX[1..]);
                if p.exists() {
                    return Some(p.into_os_string());
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

fn bin_name(chosen: Option<&String>) -> Result<Option<ShortName>, Message> {
    match chosen {
        Some(s) if s == names::TREEGREP_BIN || s == names::TREEGREP => {
            Ok(Some(ShortName(names::TREEGREP_BIN.to_owned())))
        }
        Some(s) if s == names::RIPGREP_BIN || s == names::RIPGREP => {
            Ok(Some(ShortName(names::RIPGREP_BIN.to_owned())))
        }
        Some(s)
            if cfg!(target_os = "windows")
                && (s == &(names::TREEGREP_BIN.to_owned() + env::consts::EXE_SUFFIX)
                    || s == &(names::TREEGREP.to_owned() + env::consts::EXE_SUFFIX)) =>
        {
            Ok(Some(ShortName(names::TREEGREP_BIN.to_owned())))
        }
        Some(s)
            if cfg!(target_os = "windows")
                && (s == &(names::RIPGREP_BIN.to_owned() + env::consts::EXE_SUFFIX)
                    || s == &(names::RIPGREP.to_owned() + env::consts::EXE_SUFFIX)) =>
        {
            Ok(Some(ShortName(names::RIPGREP_BIN.to_owned())))
        }
        Some(s) => Err(mes!(
            "searcher `{}` is invalid, tried `{}`",
            s,
            Searchers::all_to_str().join(", ")
        )),
        _ => Ok(None),
    }
}

impl Searchers {
    pub fn get_searcher(chosen: Option<&String>) -> Result<(Self, Option<OsString>), Message> {
        match bin_name(chosen)? {
            Some(c) => match c.0.as_str() {
                names::TREEGREP_BIN => Ok((Searchers::TreeGrep, None)),
                _ => match get_exe_path(&c) {
                    Some(path) => Ok((Searchers::from_str(&c), Some(path))),
                    _ => Err(mes!("failed to find searcher `{}`", c)),
                },
            },
            None => {
                for exec in Searchers::all() {
                    match exec {
                        Searchers::TreeGrep => return Ok((Searchers::TreeGrep, None)),
                        _ => {
                            if let Some(path) = get_exe_path(&exec.to_short_name()) {
                                return Ok((exec, Some(path)));
                            }
                        }
                    }
                }
                panic!(
                    "at this point in code treegrep would be found if you get this {SUBMIT_ISSUE}"
                )
            }
        }
    }

    pub fn generate_command(starter: OsString) -> Result<Command, Message> {
        let mut cmd = Command::new(starter);

        match config().searcher {
            Searchers::RipGrep => Rg::add_options(&mut cmd)?,
            Searchers::TreeGrep => panic!(
                "tried to use external command when using the treegrep searcher {SUBMIT_ISSUE}"
            ),
        }
        Ok(cmd)
    }

    fn to_short_name(&self) -> ShortName {
        ShortName::new(self.to_str())
    }

    pub fn to_str(&self) -> &str {
        match self {
            Searchers::RipGrep => names::RIPGREP_BIN,
            Searchers::TreeGrep => names::TREEGREP_BIN,
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            names::RIPGREP_BIN => Searchers::RipGrep,
            _ => panic!("calling from_str only happens after knowing given str is correct, if you get this {}", SUBMIT_ISSUE)
        }
    }

    fn all() -> Vec<Searchers> {
        vec![Searchers::RipGrep, Searchers::TreeGrep]
    }

    fn all_to_str() -> Vec<String> {
        Searchers::all()
            .iter()
            .flat_map(|e| {
                let s = e.to_str();
                let mut vec = Vec::new();
                vec.push(s.to_string());
                if cfg!(target_os = "windows") {
                    vec.push(format!("{}{}", s, env::consts::EXE_SUFFIX));
                }
                vec
            })
            .collect()
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
        let (c, _) = Config::get_config(
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

    #[test]
    fn test_all_to_str() {
        let res = Searchers::all_to_str();
        if cfg!(target_os = "windows") {
            assert_eq!(res, vec!["rg", "rg.exe", "tgrep", "tgrep.exe"]);
        } else {
            assert_eq!(res, vec!["rg", "tgrep"]);
        }
    }
}
