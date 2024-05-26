// SPDX-License-Identifier: CC-BY-4.0

use crate::config;
use crate::errors::Message;
use std::path::Path;
use std::process::Command;

pub trait Options {
    fn json(cmd: &mut Command);
    fn colors(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn line_num(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn pcre2(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn hidden(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn links(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn files(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn ignore(cmd: &mut Command, want: bool) -> Result<(), Message>;
    fn max_depth(cmd: &mut Command, md: Option<usize>) -> Result<(), Message>;
    fn threads(cmd: &mut Command, threads: Option<usize>) -> Result<(), Message>;
    fn patterns(cmd: &mut Command, patterns: &Vec<String>);
    fn path(cmd: &mut Command, path: &Path);
    fn globs(cmd: &mut Command, globs: &Vec<String>);

    fn add_options(cmd: &mut Command) -> Result<(), Message> {
        let config = config();
        Self::json(cmd);
        Self::patterns(cmd, &config.patterns);
        Self::path(cmd, &config.path);
        Self::globs(cmd, &config.globs);
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

pub struct Rg;

impl Options for Rg {
    fn json(cmd: &mut Command) {
        cmd.arg("--json");
    }

    fn globs(cmd: &mut Command, globs: &Vec<String>) {
        for g in globs {
            cmd.arg(format!("--glob={}", g));
        }
    }

    fn patterns(cmd: &mut Command, patterns: &Vec<String>) {
        for p in patterns {
            cmd.arg(format!("--regexp={}", p));
        }
    }

    fn path(cmd: &mut Command, path: &Path) {
        cmd.arg(path);
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
