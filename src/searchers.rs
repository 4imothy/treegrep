// SPDX-License-Identifier: CC-BY-4.0

use crate::args::names;
use crate::config::Config;
use crate::errors::{bail, Message, SUBMIT_ISSUE};
use std::env;
#[cfg(target_os = "windows")]
use std::env::consts::EXE_SUFFIX;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
fn get_exe_path_with_extension(bin: &str, ext: &str, mut p: PathBuf) -> Option<OsString> {
    p.push(format!("{}{}", bin, ext));
    fs::metadata(&p).ok().map(|_| p.into_os_string())
}

fn get_exe_path(bin: &str) -> Option<OsString> {
    env::var("PATH").ok().and_then(|path| {
        env::split_paths(&path).find_map(|p| {
            let mut p_buf = PathBuf::from(p);
            #[cfg(target_os = "windows")]
            if let Some(p) = get_exe_path_with_extension(bin, EXE_SUFFIX, p_buf.clone()) {
                return Some(p);
            }
            p_buf.push(bin);
            fs::metadata(&p_buf).ok().map(|_| p_buf.into_os_string())
        })
    })
}

trait Options {
    fn colors(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn line_num(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn pcre2(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn hidden(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn links(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn files(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn ignore(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn max_depth(cmd: &mut Command, md: Option<usize>) -> Result<(), Message>;
    fn threads(cmd: &mut Command, threads: Option<usize>) -> Result<(), Message>;
    fn add_args(cmd: &mut Command, config: &Config) -> Result<(), Message>;

    fn add_options(cmd: &mut Command, config: &Config) -> Result<(), Message> {
        Self::colors(cmd, config.colors)?;
        Self::line_num(cmd, config.line_number)?;
        Self::pcre2(cmd, config.pcre2)?;
        Self::hidden(cmd, config.hidden)?;
        Self::max_depth(cmd, config.max_depth)?;
        Self::threads(cmd, config.threads)?;
        Self::links(cmd, config.links)?;
        Self::files(cmd, config.just_files)?;
        Self::ignore(cmd, config.ignore)?;
        Ok(())
    }
}

struct Rg;

impl Options for Rg {
    fn add_args(cmd: &mut Command, config: &Config) -> Result<(), Message> {
        cmd.arg("--json");
        Rg::add_options(cmd, config)?;

        for p in &config.patterns {
            cmd.arg(format!("--regexp={}", p));
        }
        cmd.arg(&config.path);
        Ok(())
    }

    fn files(_cmd: &mut Command, want: bool) -> Result<(), Message> {
        if want {}
        Ok(())
    }

    fn max_depth(cmd: &mut Command, md: Option<usize>) -> Result<(), Message> {
        if let Some(d) = md {
            cmd.arg(format!("--max-depth={}", d));
        }
        Ok(())
    }

    fn threads(cmd: &mut Command, threads: Option<usize>) -> Result<(), Message> {
        if let Some(t) = threads {
            cmd.arg(format!("--threads={}", t));
        }
        Ok(())
    }

    fn colors(cmd: &mut Command, _want: bool) -> Result<(), Message> {
        cmd.arg("--color=never");
        Ok(())
    }

    fn line_num(cmd: &mut Command, _want: bool) -> Result<(), Message> {
        cmd.arg("--line-number");
        Ok(())
    }

    fn pcre2(cmd: &mut Command, want: bool) -> Result<(), Message> {
        if want {
            cmd.arg("--pcre2");
        }
        Ok(())
    }

    fn ignore(cmd: &mut Command, want: bool) -> Result<(), Message> {
        if !want {
            cmd.arg("--no-ignore");
        }
        Ok(())
    }

    fn hidden(cmd: &mut Command, want: bool) -> Result<(), Message> {
        if want {
            cmd.arg("--hidden");
        }
        Ok(())
    }

    fn links(cmd: &mut Command, want: bool) -> Result<(), Message> {
        if want {
            cmd.arg("--follow");
        }
        Ok(())
    }
}

pub enum Searchers {
    RipGrep,
    TreeGrep,
}

impl Searchers {
    pub fn get_searcher(chosen: Option<&String>) -> Result<(Self, Option<OsString>), Message> {
        if let Some(c) = chosen {
            if c == &Searchers::TreeGrep.to_str() {
                return Ok((Searchers::TreeGrep, None));
            }
            if let Some(path) = get_exe_path(c) {
                return Ok((Searchers::from_str(c)?, Some(path)));
            }
            return Err(bail!("failed to find searcher `{}`", c.to_owned()));
        } else {
            for exec in Searchers::all() {
                match exec {
                    Searchers::TreeGrep => {
                        return Ok((Searchers::TreeGrep, None));
                    }
                    _ => {
                        if let Some(path) = get_exe_path(&exec.to_str()) {
                            return Ok((exec, Some(path)));
                        }
                    }
                }
            }
        }
        Err(bail!(
            "no supported searcher found, tried `{}`",
            Searchers::all_to_str().join(", ")
        ))
    }

    pub fn generate_command(config: &Config, starter: OsString) -> Result<Command, Message> {
        let mut cmd = Command::new(starter);

        match config.exec {
            Searchers::RipGrep => Rg::add_args(&mut cmd, config)?,
            Searchers::TreeGrep => {
                return Err(bail!(
                    "tried to use external command when using the treegrep searcher {SUBMIT_ISSUE}"
                ))
            }
        }
        Ok(cmd)
    }

    pub fn to_str(&self) -> String {
        match self {
            Searchers::RipGrep => "rg".to_owned(),
            Searchers::TreeGrep => names::BIN_NAME.to_owned(),
        }
    }

    fn from_str(s: &str) -> Result<Self, Message> {
        match (s, cfg!(target_os = "windows")) {
            ("rg", _) | ("rg.exe", true) => Ok(Searchers::RipGrep),
            _ => Err(bail!(
                "searcher `{}` is invalid, tried `{}`",
                s.to_string(),
                Searchers::all_to_str().join(", ")
            )),
        }
    }

    fn all() -> Vec<Searchers> {
        vec![Searchers::RipGrep, Searchers::TreeGrep]
    }

    fn all_to_str<'a>() -> Vec<String> {
        let mut all: Vec<String> = Vec::new();
        for e in Searchers::all() {
            let s = e.to_str();
            #[cfg(target_os = "windows")]
            all.push(format!("{}{}", s, EXE_SUFFIX));
            all.push(s);
        }
        all
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
