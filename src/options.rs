// SPDX-License-Identifier: MIT

use crate::{config, errors::Message};
use std::{path::Path, process::Command};

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
    fn regexps(cmd: &mut Command, regexps: &[String]);
    fn path(cmd: &mut Command, path: &Path);
    fn globs(cmd: &mut Command, globs: &[String]);

    fn add_options(cmd: &mut Command) -> Result<(), Message> {
        let config = config();
        Self::json(cmd);
        Self::regexps(cmd, &config.regexps);
        Self::path(cmd, &config.path);
        Self::globs(cmd, &config.globs);
        Self::colors(cmd, config.with_colors)?;
        Self::line_num(cmd, config.line_number)?;
        Self::pcre2(cmd, config.pcre2)?;
        Self::hidden(cmd, config.hidden)?;
        Self::max_depth(cmd, config.max_depth)?;
        Self::threads(cmd, config.threads)?;
        Self::links(cmd, config.links)?;
        Self::files(cmd, config.files)?;
        Self::ignore(cmd, config.ignore)?;
        Ok(())
    }
}

pub struct Rg;

impl Options for Rg {
    fn json(cmd: &mut Command) {
        cmd.arg("--json");
    }

    fn globs(cmd: &mut Command, globs: &[String]) {
        for g in globs {
            cmd.arg(format!("--glob={}", g));
        }
    }

    fn regexps(cmd: &mut Command, regexps: &[String]) {
        for r in regexps {
            cmd.arg(format!("--regexp={}", r));
        }
    }

    fn path(cmd: &mut Command, path: &Path) {
        cmd.arg(path);
    }

    fn files(_cmd: &mut Command, _want: bool) -> Result<(), Message> {
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
