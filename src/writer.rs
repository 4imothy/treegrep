// SPDX-License-Identifier: CC-BY-4.0

use crate::config;
use crate::formats;
use crate::match_system::{Directory, File, Line, Matches};
use crossterm::style::StyledContent;
use std::ffi::OsString;
use std::io::{self, Write};

const LONG_BRANCH_FILES_PER_LINE: usize = 5;

impl Directory {
    fn write(
        &self,
        out: &mut impl Write,
        prefix: String,
        dirs: &Vec<Directory>,
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
            )?;
            writeln!(out)?;
        }
        let mut i: usize = 0;
        for child_id in children {
            i += 1;
            let dir = dirs.get(*child_id).unwrap();
            let new_prefix: String;
            if i != clen || flen > 0 {
                write!(out, "{}{}", prefix, config().c.match_with_next)?;
                new_prefix = (prefix.clone() + &config().c.spacer_vert).clone();
            } else {
                write!(out, "{}{}", prefix, config().c.match_no_next)?;
                new_prefix = (prefix.clone() + &config().c.spacer).clone();
            }
            dir.write(out, new_prefix, dirs, path_ids, cur_id)?;
        }
        if files.len() > 0 {
            if config().long_branch {
                self.long_branch_files(out, prefix, files)?;
            } else {
                for (i, file) in files.iter().enumerate() {
                    file.write(out, prefix.clone(), i + 1 != flen, path_ids, cur_id)?;
                }
            }
        }
        Ok(())
    }

    fn long_branch_files(
        &self,
        out: &mut impl Write,
        prefix: String,
        files: &Vec<File>,
    ) -> io::Result<()> {
        let mut first_long_branch = true;
        let num_lines: usize =
            (files.len() + LONG_BRANCH_FILES_PER_LINE - 1) / LONG_BRANCH_FILES_PER_LINE;
        let mut line_id: usize = 0;

        for (i, file) in files.iter().enumerate() {
            if first_long_branch {
                if num_lines == 1 {
                    write!(out, "{}{}", prefix, config().c.match_no_next)?;
                } else {
                    write!(out, "{}{}", prefix, config().c.match_with_next)?;
                }
                first_long_branch = false;
            } else if i % LONG_BRANCH_FILES_PER_LINE == 0 {
                write!(out, "{}", formats::LONG_BRANCH_FILE_SEPARATOR)?;
                writeln!(out)?;
                line_id += 1;
                if line_id + 1 == num_lines {
                    write!(out, "{}{}", prefix, config().c.match_no_next)?;
                } else {
                    write!(out, "{}{}", prefix, config().c.match_with_next)?;
                }
            } else {
                write!(out, "{}", formats::LONG_BRANCH_FILE_SEPARATOR)?;
            }
            write_name(
                &file.name,
                &file.linked,
                file.lines.len(),
                &formats::file_name,
                out,
            )?;
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
        path_ids: &mut Option<&mut Vec<usize>>,
        cur_id: &mut usize,
    ) -> io::Result<()> {
        if config().is_dir {
            if parent_has_next {
                write!(out, "{}{}", prefix, config().c.match_with_next)?;
                prefix += &config().c.spacer_vert;
            } else {
                write!(out, "{}{}", prefix, config().c.match_no_next)?;
                prefix += &config().c.spacer;
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
        )?;
        writeln!(out)?;

        if config().just_files {
            return Ok(());
        }
        let len = self.lines.len();

        for (i, line) in self.lines.iter().enumerate() {
            if i + 1 != len {
                write!(out, "{}{}", prefix, config().c.match_with_next)?;
            } else {
                write!(out, "{}{}", prefix, config().c.match_no_next)?;
            }
            line.write(out)?;
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
) -> io::Result<()> {
    if let Some(l) = linked {
        if config().colors {
            write!(out, "{} -> ", format(&name))?;
        } else {
            write!(out, "{} -> ", name)?
        }
        if config().colors {
            write!(out, "{}", format(&l.to_string_lossy()))?;
        } else {
            write!(out, "{}", l.to_string_lossy())?;
        }
    } else {
        if config().colors {
            write!(out, "{}", format(&name))?;
        } else {
            write!(out, "{}", name)?;
        }
    }
    let file_in_tree: bool = config().tree && count == 0;
    if config().count && !file_in_tree {
        write!(out, ": {}", count)?;
    }

    Ok(())
}

impl Line {
    pub fn write(&self, out: &mut impl Write) -> std::io::Result<()> {
        let contents: &[u8] = self.contents.as_ref().unwrap();
        let mut need_new_line = false;
        if !contents.ends_with(&[formats::NEW_LINE as u8]) {
            need_new_line = true;
        }
        let line_num = self.line_num;
        if !config().colors {
            if config().line_number {
                write!(out, "{}: ", line_num.unwrap())?;
            }
            write!(out, "{}", String::from_utf8_lossy(&contents))?;
        } else {
            if config().line_number {
                write!(out, "{}", formats::line_number(line_num.unwrap()))?;
            } else if config().menu {
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

pub fn write_results(
    out: &mut impl Write,
    result: &Matches,
    mut path_ids: Option<&mut Vec<usize>>,
) -> io::Result<()> {
    let prefix: String = "".into();
    let mut cur_id: usize = 0;
    match &result {
        Matches::Dir(dirs) => {
            dirs.get(0)
                .unwrap()
                .write(out, prefix, dirs, &mut path_ids, &mut cur_id)?
        }
        Matches::File(file) => {
            file.write(out, "".to_string(), false, &mut path_ids, &mut cur_id)?;
        }
    }

    Ok(())
}
