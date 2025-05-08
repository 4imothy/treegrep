// SPDX-License-Identifier: MIT

use crate::config;
use crate::errors::{mes, Message};
use crate::formats;
use crate::match_system::{Directory, File, Match, Matches};
use crate::term::TERM_WIDTH;
use core::fmt::{self, Display};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::Ordering;

enum PrefixComponent {
    MatchWithNext,
    MatchNoNext,
    SpacerVert,
    Spacer,
}

impl Clone for PrefixComponent {
    fn clone(&self) -> Self {
        match self {
            PrefixComponent::MatchWithNext => PrefixComponent::MatchWithNext,
            PrefixComponent::MatchNoNext => PrefixComponent::MatchNoNext,
            PrefixComponent::SpacerVert => PrefixComponent::SpacerVert,
            PrefixComponent::Spacer => PrefixComponent::Spacer,
        }
    }
}

pub struct OpenInfo<'a> {
    pub path: &'a Path,
    pub line: Option<usize>,
}

pub trait Entry: Display {
    fn open_info(&self) -> Result<OpenInfo, Message>;
    fn set_term_width(&mut self, width: u16);
}

struct FileEntry<'a> {
    prefix: Vec<PrefixComponent>,
    name: &'a str,
    path: &'a Path,
    linked: Option<&'a OsStr>,
    term_width: Option<u16>,
    count: usize,
    new_line: bool,
}

impl<'a> Entry for FileEntry<'a> {
    fn open_info(&self) -> Result<OpenInfo, Message> {
        Ok(OpenInfo {
            path: self.path,
            line: None,
        })
    }

    fn set_term_width(&mut self, width: u16) {
        self.term_width = Some(width);
    }
}

impl<'a> Display for FileEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write_prefix(f, &self.prefix)?;
        write_path(f, self.name, self.linked, false, self.count, self.new_line)
    }
}

struct DirEntry<'a> {
    prefix: Vec<PrefixComponent>,
    name: &'a str,
    path: &'a Path,
    term_width: Option<u16>,
    linked: Option<&'a OsStr>,
    count: usize,
    new_line: bool,
}

impl<'a> Entry for DirEntry<'a> {
    fn open_info(&self) -> Result<OpenInfo, Message> {
        Ok(OpenInfo {
            path: self.path,
            line: None,
        })
    }

    fn set_term_width(&mut self, width: u16) {
        self.term_width = Some(width);
    }
}

impl<'a> Display for DirEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write_prefix(f, &self.prefix)?;
        write_path(f, self.name, self.linked, true, self.count, self.new_line)
    }
}

fn write_path(
    f: &mut fmt::Formatter,
    name: &str,
    linked: Option<&OsStr>,
    dir: bool,
    count: usize,
    new_line: bool,
) -> fmt::Result {
    write!(
        f,
        "{}",
        if dir {
            formats::dir_name(name)
        } else {
            formats::file_name(name)
        }
    )?;
    if let Some(l) = linked {
        let link: &str = &l.to_string_lossy();
        write!(
            f,
            " -> {}",
            if dir {
                formats::dir_name(link)
            } else {
                formats::file_name(link)
            }
        )?;
    }
    let file_in_tree: bool = config().tree && count == 0;
    if config().count && !file_in_tree {
        write!(f, ": {}", count)?;
    }
    if new_line {
        writeln!(f)?;
    }
    Ok(())
}

fn write_prefix(f: &mut fmt::Formatter, prefix_components: &Vec<PrefixComponent>) -> fmt::Result {
    for c in prefix_components {
        let s = match c {
            PrefixComponent::MatchWithNext => &config().c.match_with_next,
            PrefixComponent::MatchNoNext => &config().c.match_no_next,
            PrefixComponent::SpacerVert => &config().c.spacer_vert,
            PrefixComponent::Spacer => &config().c.spacer,
        };
        f.write_str(s)?;
    }
    Ok(())
}

struct LineEntry<'a> {
    prefix: Vec<PrefixComponent>,
    content: &'a str,
    path: &'a Path,
    term_width: Option<u16>,
    matches: &'a [Match],
    line_num: usize,
    new_line: bool,
}

impl<'a> Entry for LineEntry<'a> {
    fn open_info(&self) -> Result<OpenInfo, Message> {
        Ok(OpenInfo {
            path: self.path,
            line: Some(self.line_num),
        })
    }

    fn set_term_width(&mut self, width: u16) {
        self.term_width = Some(width);
    }
}

impl<'a> Display for LineEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write_prefix(f, &self.prefix)?;
        if config().colors || config().bold {
            write!(f, "{}", formats::RESET)?;
        }

        let mut content: &str = self.content;

        let cut = if config().trim {
            let trimmed = content.trim_start();
            let cut_amount = content.len() - trimmed.len();
            content = trimmed;
            cut_amount
        } else {
            0
        };

        if config().menu {
            let term_width = TERM_WIDTH.load(Ordering::SeqCst) as usize;
            let content_width = term_width.saturating_sub(config().prefix_len * self.prefix.len());
            let max_len = config()
                .max_length
                .unwrap_or(content_width)
                .min(content_width)
                .min(content.len());
            content = &content[..max_len];
        } else if let Some(max) = config().max_length {
            content = &content[..max.min(content.len())];
        }

        if config().line_number {
            write!(f, "{}: ", formats::line_number(self.line_num))?;
        }

        if !config().colors {
            f.write_str(content)?;
        } else {
            let mut last = 0;

            for m in self.matches.iter() {
                if m.start >= m.end || m.start >= content.len() {
                    continue;
                }

                let m_start = m.start.saturating_sub(cut);
                let m_end = m.end.saturating_sub(cut).min(content.len());
                if m_start >= content.len() {
                    continue;
                }

                if last < m_start {
                    f.write_str(&content[last..m_start])?;
                }

                let styled = formats::match_substring(&content[m_start..m_end], m.pattern_id);
                write!(f, "{}", styled)?;

                last = m_end;
            }

            if last < content.len() {
                f.write_str(&content[last..])?;
            }
        }

        if self.new_line {
            writeln!(f)?;
        }

        Ok(())
    }
}

struct LongBranchEntry<'a> {
    prefix: Vec<PrefixComponent>,
    files: &'a [File],
    term_width: Option<u16>,
    new_line: bool,
}

impl<'a> Entry for LongBranchEntry<'a> {
    fn open_info(&self) -> Result<OpenInfo, Message> {
        Err(mes!("can't open a long branch"))
    }

    fn set_term_width(&mut self, width: u16) {
        self.term_width = Some(width);
    }
}

impl<'a> Display for LongBranchEntry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write_prefix(f, &self.prefix)?;
        for (i, file) in self.files.iter().enumerate() {
            write_path(
                f,
                &file.name,
                file.linked.as_deref(),
                false,
                file.lines.len(),
                false,
            )?;
            if i + 1 != self.files.len() {
                f.write_str(formats::LONG_BRANCH_FILE_SEPARATOR)?;
            }
        }
        if self.new_line {
            writeln!(f)?;
        }
        Ok(())
    }
}

fn with_push(mut v: Vec<PrefixComponent>, item: PrefixComponent) -> Vec<PrefixComponent> {
    v.push(item);
    v
}

impl Directory {
    fn to_lines<'a>(
        &'a self,
        lines: &mut Vec<Box<dyn Entry + 'a>>,
        cur_prefix: Vec<PrefixComponent>,
        child_prefix: Vec<PrefixComponent>,
        dirs: &'a Vec<Directory>,
        path_ids: &mut Option<&mut Vec<usize>>,
    ) {
        let children = &self.children;
        let files = &self.files;
        let flen = files.len();
        let clen = children.len();
        if clen > 0 || flen > 0 {
            path_ids.as_mut().map(|p| p.push(lines.len()));
            lines.push(Box::new(DirEntry {
                prefix: cur_prefix.clone(),
                name: &self.name,
                path: &self.path,
                linked: self.linked.as_deref(),
                count: self.children.len() + self.files.len(),
                term_width: None,
                new_line: !config().menu,
            }));
        }
        for (i, child_id) in children.iter().enumerate() {
            let dir = dirs.get(*child_id).unwrap();
            let (cur_prefix, new_child_prefix) = if i + 1 != clen || flen > 0 {
                (
                    with_push(child_prefix.clone(), PrefixComponent::MatchWithNext),
                    with_push(child_prefix.clone(), PrefixComponent::SpacerVert),
                )
            } else {
                (
                    with_push(child_prefix.clone(), PrefixComponent::MatchNoNext),
                    with_push(child_prefix.clone(), PrefixComponent::Spacer),
                )
            };
            dir.to_lines(lines, cur_prefix, new_child_prefix, dirs, path_ids);
        }
        if files.len() > 0 {
            if config().long_branch {
                self.long_branch_files_to_lines(lines, child_prefix);
            } else {
                for (i, file) in files.iter().enumerate() {
                    file.to_lines(lines, child_prefix.clone(), i + 1 != flen, path_ids);
                }
            }
        }
    }

    fn long_branch_files_to_lines<'a>(
        &'a self,
        lines: &mut Vec<Box<dyn Entry + 'a>>,
        prefix: Vec<PrefixComponent>,
    ) {
        let long_branch_files_per_line: usize = config().long_branch_each;
        let num_lines: usize =
            (self.files.len() + long_branch_files_per_line - 1) / long_branch_files_per_line;

        let prefix_no_next = with_push(prefix.clone(), PrefixComponent::MatchNoNext);
        let prefix_next = with_push(prefix.clone(), PrefixComponent::MatchWithNext);

        for (i, branch) in self.files.chunks(long_branch_files_per_line).enumerate() {
            lines.push(Box::new(LongBranchEntry {
                prefix: if i + 1 == num_lines {
                    prefix_no_next.clone()
                } else {
                    prefix_next.clone()
                },
                files: branch,
                term_width: None,
                new_line: !config().menu,
            }));
        }
    }
}

impl File {
    fn to_lines<'a>(
        &'a self,
        lines: &mut Vec<Box<dyn Entry + 'a>>,
        prefix: Vec<PrefixComponent>,
        parent_has_next: bool,
        path_ids: &mut Option<&mut Vec<usize>>,
    ) {
        let (cur_p, line_p) = if config().is_dir {
            if parent_has_next {
                (
                    with_push(prefix.clone(), PrefixComponent::MatchWithNext),
                    with_push(prefix.clone(), PrefixComponent::SpacerVert),
                )
            } else {
                (
                    with_push(prefix.clone(), PrefixComponent::MatchNoNext),
                    with_push(prefix.clone(), PrefixComponent::Spacer),
                )
            }
        } else {
            (prefix.clone(), prefix)
        };

        path_ids.as_mut().map(|p| p.push(lines.len()));
        lines.push(Box::new(FileEntry {
            prefix: cur_p,
            name: &self.name,
            path: &self.path,
            linked: self.linked.as_deref(),
            term_width: None,
            count: self.lines.len(),
            new_line: !config().menu,
        }));

        if config().just_files {
            return;
        }

        for (i, line) in self.lines.iter().enumerate() {
            let prefix = if i + 1 != self.lines.len() {
                with_push(line_p.clone(), PrefixComponent::MatchWithNext)
            } else {
                with_push(line_p.clone(), PrefixComponent::MatchNoNext)
            };
            lines.push(Box::new(LineEntry {
                prefix,
                content: &line.content,
                path: &self.path,
                term_width: None,
                matches: &line.matches,
                line_num: line.line_num,
                new_line: !config().menu,
            }));
        }
    }
}

pub fn matches_to_display_lines<'a>(
    result: &'a Matches,
    mut path_ids: Option<&mut Vec<usize>>,
) -> Vec<Box<dyn Entry + 'a>> {
    let mut lines: Vec<Box<dyn Entry + 'a>> = Vec::new();
    match &result {
        Matches::Dir(dirs) => {
            dirs.get(0)
                .unwrap()
                .to_lines(&mut lines, Vec::new(), Vec::new(), dirs, &mut path_ids);
        }
        Matches::File(file) => {
            file.to_lines(&mut lines, Vec::new(), false, &mut path_ids);
        }
    }
    lines
}

pub fn write_results<'a>(
    out: &mut io::StdoutLock,
    lines: &Vec<Box<dyn Entry + 'a>>,
) -> io::Result<()> {
    for line in lines {
        write!(out, "{}", line)?;
    }
    Ok(())
}
