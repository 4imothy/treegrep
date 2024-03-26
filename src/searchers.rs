// SPDX-License-Identifier: CC-BY-4.0

use crate::args::names;
use crate::config::Config;
use crate::errors::{bail, Message, SUBMIT_ISSUE};
use crate::options::{Options, Rg};
use std::env;
#[cfg(target_os = "windows")]
use std::env::consts::EXE_SUFFIX;
use std::ffi::OsString;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Command;

struct ShortName(String);

impl ShortName {
    fn new(name: &str) -> Self {
        assert!(name == names::TREEGREP_BIN || name == names::RIPGREP_BIN);
        ShortName(name.to_owned())
    }
}

impl Deref for ShortName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(target_os = "windows")]
fn get_exe_path_with_extension(bin: &ShortName, ext: &str, mut p: PathBuf) -> Option<OsString> {
    p.push(format!("{}{}", &bin.0, ext));
    fs::metadata(&p).ok().map(|_| p.into_os_string())
}

fn get_exe_path(bin: &ShortName) -> Option<OsString> {
    env::var("PATH").ok().and_then(|path| {
        env::split_paths(&path).find_map(|p| {
            let mut p_buf = PathBuf::from(p);
            #[cfg(target_os = "windows")]
            if let Some(p) = get_exe_path_with_extension(bin, EXE_SUFFIX, p_buf.clone()) {
                return Some(p);
            }
            p_buf.push(&bin.0);
            fs::metadata(&p_buf).ok().map(|_| p_buf.into_os_string())
        })
    })
}

pub enum Searchers {
    RipGrep,
    TreeGrep,
}

fn bin_name(chosen: Option<&String>) -> Result<Option<ShortName>, Message> {
    match chosen {
        Some(s) if s == &names::TREEGREP_BIN || s == &names::TREEGREP => {
            Ok(Some(ShortName(names::TREEGREP_BIN.to_owned())))
        }
        Some(s) if s == &names::RIPGREP_BIN || s == &names::RIPGREP => {
            Ok(Some(ShortName(names::RIPGREP_BIN.to_owned())))
        }
        #[cfg(target_os = "windows")]
        Some(s)
            if s == &(names::TREEGREP_BIN.to_owned() + &EXE_SUFFIX)
                || s == &(names::TREEGREP.to_owned() + &EXE_SUFFIX) =>
        {
            Ok(Some(ShortName(names::TREEGREP_BIN.to_owned())))
        }
        #[cfg(target_os = "windows")]
        Some(s)
            if s == &(names::RIPGREP_BIN.to_owned() + &EXE_SUFFIX)
                || s == &(names::RIPGREP.to_owned() + &EXE_SUFFIX) =>
        {
            Ok(Some(ShortName(names::RIPGREP_BIN.to_owned())))
        }
        Some(s) => Err(bail!(
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
                    _ => Err(bail!("failed to find searcher `{}`", c.to_owned())),
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

    pub fn generate_command(config: &Config, starter: OsString) -> Result<Command, Message> {
        let mut cmd = Command::new(starter);

        match config.exec {
            Searchers::RipGrep => Rg::add_args(&mut cmd, config)?,
            Searchers::TreeGrep => panic!(
                "tried to use external command when using the treegrep searcher {SUBMIT_ISSUE}"
            ),
        }
        Ok(cmd)
    }

    fn to_short_name(&self) -> ShortName {
        ShortName::new(&self.to_str())
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
                #[cfg(target_os = "windows")]
                vec.push(format!("{}{}", s, EXE_SUFFIX));
                vec
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_add_args_rg() {
        let mut cmd = Command::new("rg");
        let config = Config {
            colors: true,
            line_number: true,
            pcre2: true,
            hidden: true,
            max_depth: Some(5),
            threads: Some(8),
            max_length: None,
            links: true,
            just_files: true,
            ignore: false,
            patterns: vec!["pattern1".to_string(), "pattern2".to_string()],
            path: PathBuf::from("test_path"),
            is_dir: true,
            exec: Searchers::TreeGrep,
            count: true,
            menu: true,
            trim: true,
        };

        assert!(Rg::add_args(&mut cmd, &config).is_ok());

        let expected_args = vec![
            "--json",
            "--color=never",
            "--line-number",
            "--pcre2",
            "--hidden",
            "--max-depth=5",
            "--threads=8",
            "--follow",
            "--no-ignore",
            "--regexp=pattern1",
            "--regexp=pattern2",
            "test_path",
        ];

        assert_eq!(
            cmd.get_args()
                .map(|s| s.to_str().unwrap())
                .collect::<Vec<&str>>(),
            expected_args
        );
    }

    #[test]
    fn test_all_to_str() {
        let res = Searchers::all_to_str();
        #[cfg(target_os = "windows")]
        assert_eq!(res, vec!["rg.exe", "rg", "tgrep.exe", "tgrep"]);
        #[cfg(not(target_os = "windows"))]
        assert_eq!(res, vec!["rg", "tgrep"]);
    }
}
