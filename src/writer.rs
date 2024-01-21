// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::formats::{self, BRANCH_END, BRANCH_HAS_NEXT, SPACER, VER_LINE_SPACER};
use crate::match_system::{Directory, File, Line, Matches};
use crossterm::style::StyledContent;
use std::ffi::OsString;
use std::io::{self, Write};

impl Directory {
    fn write(
        &self,
        out: &mut impl Write,
        prefix: String,
        dirs: &Vec<Directory>,
        config: &Config,
        path_ids: &mut Option<&mut Vec<usize>>,
        cur_id: &mut usize,
    ) -> io::Result<()> {
        let children = &self.children;
        let files = &self.files;
        let flen = files.len();
        let clen = children.len();
        if clen > 0 || flen > 0 {
            path_ids.as_mut().map(|p| p.push(*cur_id));
            *cur_id += 1;
            write_name(
                &self.name,
                &self.linked,
                self.children.len() + self.files.len(),
                &formats::dir_name,
                out,
                config,
            )?;
        }
        let mut i: usize = 0;
        for child_id in children {
            i += 1;
            let dir = dirs.get(*child_id).unwrap();
            let new_prefix: String;
            if i != clen || flen > 0 {
                write!(out, "{}{}", prefix, BRANCH_HAS_NEXT)?;
                new_prefix = (prefix.clone() + VER_LINE_SPACER).clone();
            } else {
                write!(out, "{}{}", prefix, BRANCH_END)?;
                new_prefix = (prefix.clone() + SPACER).clone();
            }
            dir.write(out, new_prefix, dirs, config, path_ids, cur_id)?;
        }
        for (i, file) in files.iter().enumerate() {
            file.write(out, prefix.clone(), i + 1 != flen, config, path_ids, cur_id)?;
        }
        Ok(())
    }
}

impl File {
    pub fn write(
        &self,
        out: &mut impl Write,
        mut prefix: String,
        parent_has_next: bool,
        config: &Config,
        path_ids: &mut Option<&mut Vec<usize>>,
        cur_id: &mut usize,
    ) -> io::Result<()> {
        if config.is_dir {
            if parent_has_next {
                write!(out, "{}{}", prefix, BRANCH_HAS_NEXT)?;
                prefix += VER_LINE_SPACER;
            } else {
                write!(out, "{}{}", prefix, BRANCH_END)?;
                prefix += SPACER;
            }
        }
        path_ids.as_mut().map(|p| p.push(*cur_id));
        *cur_id += 1;
        write_name(
            &self.name,
            &self.linked,
            self.lines.len(),
            &formats::file_name,
            out,
            config,
        )?;

        if config.just_files {
            return Ok(());
        }
        let len = self.lines.len();

        for (i, line) in self.lines.iter().enumerate() {
            if i + 1 != len {
                write!(out, "{}{}", prefix, BRANCH_HAS_NEXT,)?;
            } else {
                write!(out, "{}{}", prefix, BRANCH_END)?;
            }
            line.write(out, config)?;
            *cur_id += 1;
        }

        Ok(())
    }
}

fn write_name(
    name: &String,
    linked: &Option<OsString>,
    count: usize,
    format: &dyn Fn(&str) -> StyledContent<&str>,
    out: &mut impl Write,
    config: &Config,
) -> io::Result<()> {
    if let Some(l) = linked {
        if config.colors {
            write!(out, "{} -> ", format(&name))?;
        } else {
            write!(out, "{} -> ", name)?
        }
        if config.colors {
            write!(out, "{}", format(&l.to_string_lossy()))?;
        } else {
            write!(out, "{}", l.to_string_lossy())?;
        }
    } else {
        if config.colors {
            write!(out, "{}", format(&name))?;
        } else {
            write!(out, "{}", name)?;
        }
    }
    if config.count {
        write!(out, ": {}", count)?;
    }
    writeln!(out)?;

    Ok(())
}

impl Line {
    pub fn write(&self, out: &mut impl Write, config: &Config) -> std::io::Result<()> {
        let contents: &[u8] = self.contents.as_ref().unwrap();
        let mut need_new_line = false;
        if !contents.ends_with(&[formats::NEW_LINE as u8]) {
            need_new_line = true;
        }
        let line_num = self.line_num;
        if !config.colors {
            if config.line_number {
                write!(out, "{}: ", line_num.unwrap())?;
            }
            write!(out, "{}", String::from_utf8_lossy(&contents))?;
        } else {
            if config.line_number {
                write!(out, "{}", formats::line_number(line_num.unwrap()))?;
            } else if config.menu {
                write!(out, "{}", formats::RESET_COLOR)?;
            }
            write!(out, " {}", String::from_utf8_lossy(&contents))?;
        }

        if need_new_line {
            writeln!(out)?;
        }
        Ok(())
    }
}

pub fn write_results(
    out: &mut impl Write,
    result: &Matches,
    config: &Config,
    mut path_ids: Option<&mut Vec<usize>>,
) -> io::Result<()> {
    let prefix: String = "".into();
    let mut cur_id: usize = 0;
    match &result {
        Matches::Dir(dirs) => {
            dirs.get(0)
                .unwrap()
                .write(out, prefix, dirs, config, &mut path_ids, &mut cur_id)?
        }
        Matches::File(file) => {
            file.write(
                out,
                "".to_string(),
                false,
                config,
                &mut path_ids,
                &mut cur_id,
            )?;
        }
    }

    Ok(())
}
