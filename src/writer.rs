// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::formats::{self, BRANCH_END, BRANCH_HAS_NEXT, SPACER, VER_LINE_SPACER};
use crate::match_system::{Directory, File, Line, Matches};
use std::io::{self, Write};

impl Directory {
    fn write(
        &self,
        out: &mut impl Write,
        prefix: String,
        dirs: &Vec<Directory>,
        config: &Config,
    ) -> io::Result<()> {
        let children = &self.children;
        let files = &self.files;
        let flen = files.len();
        let clen = children.len();
        if clen > 0 || flen > 0 {
            self.write_name(out, config)?;
        }
        let mut i: usize = 0;
        for child_id in children {
            i += 1;
            let dir = dirs.get(*child_id).unwrap();
            if i != clen || flen > 0 {
                write!(out, "{}{}", prefix, BRANCH_HAS_NEXT)?;
                dir.write(
                    out,
                    (prefix.clone() + VER_LINE_SPACER).clone(),
                    dirs,
                    config,
                )?;
            } else {
                write!(out, "{}{}", prefix, BRANCH_END)?;
                dir.write(out, (prefix.clone() + SPACER).clone(), dirs, config)?;
            }
        }
        for (i, file) in files.iter().enumerate() {
            if i + 1 != flen {
                file.write(out, prefix.clone(), true, config)?;
            } else {
                file.write(out, prefix.clone(), false, config)?;
            }
        }
        Ok(())
    }

    fn write_name(&self, out: &mut impl Write, config: &Config) -> io::Result<()> {
        if config.colors {
            write!(out, "{}", formats::dir_name(&self.name))?;
        } else {
            write!(out, "{}", self.name)?;
        }
        if config.count {
            write!(out, ": {}", self.files.len() + self.children.len())?;
        }
        writeln!(out)?;

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
    ) -> io::Result<()> {
        if !config.is_dir {
            self.write_name(out, config)?;
        } else if parent_has_next {
            write!(out, "{}{}", prefix, BRANCH_HAS_NEXT)?;
            self.write_name(out, config)?;
            prefix += VER_LINE_SPACER;
        } else {
            write!(out, "{}{}", prefix, BRANCH_END)?;
            self.write_name(out, config)?;
            prefix += SPACER;
        }

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
        }

        Ok(())
    }

    pub fn write_name(&self, out: &mut impl Write, config: &Config) -> io::Result<()> {
        if let Some(linked) = &self.linked {
            if config.colors {
                write!(out, "{} -> ", formats::file_name(&self.name))?;
            } else {
                write!(out, "{} -> ", self.name)?
            }
            if config.colors {
                write!(out, "{}", formats::file_name(&linked.to_string_lossy()))?;
            } else {
                write!(out, "{}", linked.to_string_lossy())?;
            }
        } else {
            if config.colors {
                write!(out, "{}", formats::file_name(&self.name))?;
            } else {
                write!(out, "{}", self.name)?;
            }
        }
        if config.count {
            write!(out, ": {}", self.lines.len())?;
        }
        writeln!(out)?;

        Ok(())
    }
}

impl Line {
    pub fn write(&self, out: &mut impl Write, config: &Config) -> std::io::Result<()> {
        let contents: &[u8] = self.contents.as_ref().unwrap();
        let need_new_line;
        if !contents.ends_with(&[formats::NEW_LINE as u8]) {
            need_new_line = true;
        } else {
            need_new_line = false;
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
                // TODO dont think this else actually does anything
                write!(out, "{}", formats::RESET_COLOR)?;
            }
            write!(out, "{}", String::from_utf8_lossy(&contents))?;
        }

        if need_new_line {
            writeln!(out)?;
        }
        Ok(())
    }
}

pub fn write_results(out: &mut impl Write, result: &Matches, config: &Config) -> io::Result<()> {
    let prefix: String = "".into();
    match &result {
        Matches::Dir(dirs) => dirs.get(0).unwrap().write(out, prefix, dirs, config)?,
        Matches::File(file) => {
            file.write(out, "".to_string(), false, config)?;
        }
    }

    Ok(())
}
