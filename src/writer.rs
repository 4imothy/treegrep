// SPDX-License-Identifier: MIT

use crate::config;
use crate::errors::{Message, mes};
use crate::formats;
use crate::match_system::{Directory, File, Match, Matches};
use crate::term::TERM_WIDTH;
use crate::term::Term;
use core::fmt::{self, Display};
use crossterm::style::StyledContent;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

pub enum PrefixComponent {
    MatchWithNext,
    MatchNoNext,
    SpacerVert,
    Spacer,
}

impl Clone for PrefixComponent {
    fn clone(&self) -> Self {
        match self {
            Self::MatchWithNext => Self::MatchWithNext,
            Self::MatchNoNext => Self::MatchNoNext,
            Self::SpacerVert => Self::SpacerVert,
            Self::Spacer => Self::Spacer,
        }
    }
}

pub struct OpenInfo<'a> {
    pub path: &'a Path,
    pub line: Option<usize>,
}

pub trait Entry: Display {
    fn open_info(&self) -> Result<OpenInfo<'_>, Message>;
    fn depth(&self) -> usize;
}

struct PathDisplay<'a> {
    prefix: Option<Vec<PrefixComponent>>,
    name: StyledContent<String>,
    path: &'a Path,
    linked: Option<StyledContent<String>>,
    count: usize,
    new_line: bool,
}

impl<'a> PathDisplay<'a> {
    fn new(
        prefix: Option<Vec<PrefixComponent>>,
        path: &'a PathBuf,
        linked: &'a Option<PathBuf>,
        count: usize,
        new_line: bool,
        dir: bool,
    ) -> Result<PathDisplay<'a>, Message> {
        let (name, linked) = {
            let p = path_name(path)?;
            let l = linked.as_ref().map(|l| path_name(l.as_ref())).transpose()?;
            if dir {
                (formats::dir_name(p), l.map(formats::dir_name))
            } else {
                (formats::file_name(p), l.map(formats::file_name))
            }
        };

        Ok(PathDisplay {
            prefix,
            name,
            path,
            linked,
            count,
            new_line,
        })
    }
}

impl<'a> Entry for PathDisplay<'a> {
    fn open_info(&self) -> Result<OpenInfo<'_>, Message> {
        Ok(OpenInfo {
            path: self.path,
            line: None,
        })
    }
    fn depth(&self) -> usize {
        self.prefix.as_ref().map_or(0, |p| p.len())
    }
}

impl<'a> Display for PathDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        if let Some(p) = &self.prefix {
            write_prefix(f, p)?;
        }
        write_path(f, &self.name, &self.linked, self.count, self.new_line)
    }
}

fn write_path(
    f: &mut fmt::Formatter,
    name: &StyledContent<String>,
    linked: &Option<StyledContent<String>>,
    count: usize,
    new_line: bool,
) -> fmt::Result {
    if cfg!(feature = "test") {
        write!(f, "[ps]{}[pe]", name)?;
        if let Some(l) = linked {
            write!(f, " -> [ps]{}[pe]", l)?;
        }
    } else {
        write!(f, "{}", name)?;
        if let Some(l) = linked {
            write!(f, " -> {}", l)?;
        }
    }
    if config().count && count > 0 {
        write!(f, ": {}", count)?;
    }
    if config().colors || config().bold {
        write!(f, "{}", formats::RESET)?;
    }

    if new_line {
        writeln!(f)?;
    }
    Ok(())
}

fn write_prefix(f: &mut fmt::Formatter, prefix_components: &[PrefixComponent]) -> fmt::Result {
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

struct LineDisplay<'a> {
    prefix: Vec<PrefixComponent>,
    content: &'a str,
    path: &'a Path,
    matches: &'a [Match],
    line_num: usize,
    new_line: bool,
}

impl<'a> Entry for LineDisplay<'a> {
    fn open_info(&self) -> Result<OpenInfo<'_>, Message> {
        Ok(OpenInfo {
            path: self.path,
            line: Some(self.line_num),
        })
    }
    fn depth(&self) -> usize {
        self.prefix.len()
    }
}

impl<'a> Display for LineDisplay<'a> {
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

        if config().select {
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
            if cfg!(feature = "test") {
                write!(f, "[m{}s]{}[m{}e]", m.pattern_id, styled, m.pattern_id)?;
            } else {
                write!(f, "{}", styled)?;
            }

            last = m_end;
        }

        if last < content.len() {
            f.write_str(&content[last..])?;
        }

        if self.new_line {
            writeln!(f)?;
        }

        Ok(())
    }
}

struct LongBranchDisplay<'a> {
    prefix: Vec<PrefixComponent>,
    files: Vec<PathDisplay<'a>>,
    new_line: bool,
}

impl<'a> Entry for LongBranchDisplay<'a> {
    fn open_info(&self) -> Result<OpenInfo<'_>, Message> {
        match self.files.as_slice() {
            [file] => file.open_info(),
            _ => Err(mes!("can't open a long branch")),
        }
    }
    fn depth(&self) -> usize {
        self.prefix.len()
    }
}

impl<'a> Display for LongBranchDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write_prefix(f, &self.prefix)?;
        if config().colors || config().bold {
            write!(f, "{}", formats::RESET)?;
        }

        for (i, file) in self.files.iter().enumerate() {
            write_path(f, &file.name, &file.linked, file.count, false)?;
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

struct OverviewDisplay {
    dirs: usize,
    files: usize,
    lines: usize,
    count: usize,
    new_line: bool,
}

impl Entry for OverviewDisplay {
    fn open_info(&self) -> Result<OpenInfo<'_>, Message> {
        Err(mes!("can't open stats"))
    }
    fn depth(&self) -> usize {
        0
    }
}

impl Display for OverviewDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        if config().colors || config().bold {
            write!(f, "{}", formats::RESET)?;
        }
        if !config().just_files {
            write!(
                f,
                "found {} matches in {} lines across ",
                self.count, self.lines
            )?;
        }
        write!(
            f,
            "{} {}",
            self.files,
            if self.files == 1 { "file" } else { "files" }
        )?;
        if config().is_dir {
            write!(f, " within {} directories", self.dirs)?;
        }
        if self.new_line {
            writeln!(f)?;
        }
        Ok(())
    }
}

fn path_name(path: &Path) -> Result<String, Message> {
    let name = path.file_name().ok_or(mes!(
        "failed to get name of `{}`",
        path.as_os_str().to_string_lossy()
    ))?;

    name.to_os_string().into_string().map_err(|_| {
        mes!(
            "failed to get name of `{}`",
            path.as_os_str().to_string_lossy()
        )
    })
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
        overview: &mut Option<&mut Box<OverviewDisplay>>,
    ) -> Result<(), Message> {
        let children = &self.children;
        let files = &self.files;
        let flen = files.len();
        let clen = children.len();
        if clen > 0 || flen > 0 {
            if let Some(p) = path_ids.as_mut() {
                p.push(lines.len())
            }
            lines.push(Box::new(PathDisplay::new(
                Some(cur_prefix.clone()),
                &self.path,
                &self.linked,
                self.children.len() + self.files.len(),
                !config().select,
                true,
            )?));
        }

        if let Some(o) = overview {
            o.dirs += 1;
            o.files += files.len();
        }

        for (i, child_id) in children.iter().enumerate() {
            let dir = &dirs[*child_id];
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
            dir.to_lines(
                lines,
                cur_prefix,
                new_child_prefix,
                dirs,
                path_ids,
                overview,
            )?;
        }
        if !files.is_empty() {
            if config().long_branch {
                self.long_branch_files_to_lines(lines, child_prefix, path_ids)?;
            } else {
                for (i, file) in files.iter().enumerate() {
                    file.to_lines(
                        lines,
                        child_prefix.clone(),
                        i + 1 != flen,
                        path_ids,
                        overview,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn long_branch_files_to_lines<'a>(
        &'a self,
        lines: &mut Vec<Box<dyn Entry + 'a>>,
        prefix: Vec<PrefixComponent>,
        path_ids: &mut Option<&mut Vec<usize>>,
    ) -> Result<(), Message> {
        let long_branch_files_per_line: usize = config().long_branch_each;
        let num_lines: usize = self.files.len().div_ceil(long_branch_files_per_line);

        let prefix_no_next = with_push(prefix.clone(), PrefixComponent::MatchNoNext);
        let prefix_next = with_push(prefix, PrefixComponent::MatchWithNext);

        for (i, branch) in self.files.chunks(long_branch_files_per_line).enumerate() {
            if let Some(p) = path_ids {
                p.push(lines.len());
            }
            lines.push(Box::new(LongBranchDisplay {
                prefix: if i + 1 == num_lines {
                    prefix_no_next.clone()
                } else {
                    prefix_next.clone()
                },
                files: branch
                    .iter()
                    .map(|f| PathDisplay::new(None, &f.path, &f.linked, f.count(), false, false))
                    .collect::<Result<Vec<PathDisplay<'_>>, Message>>()?,
                new_line: !config().select,
            }));
        }
        Ok(())
    }
}

impl File {
    fn to_lines<'a>(
        &'a self,
        lines: &mut Vec<Box<dyn Entry + 'a>>,
        prefix: Vec<PrefixComponent>,
        parent_has_next: bool,
        path_ids: &mut Option<&mut Vec<usize>>,
        overview: &mut Option<&mut Box<OverviewDisplay>>,
    ) -> Result<(), Message> {
        if let Some(o) = overview {
            o.lines += self.lines.len();
            o.count += self.lines.iter().map(|l| l.matches.len()).sum::<usize>();
        }
        let (cur_p, line_p) = if config().is_dir {
            if parent_has_next {
                (
                    with_push(prefix.clone(), PrefixComponent::MatchWithNext),
                    with_push(prefix, PrefixComponent::SpacerVert),
                )
            } else {
                (
                    with_push(prefix.clone(), PrefixComponent::MatchNoNext),
                    with_push(prefix, PrefixComponent::Spacer),
                )
            }
        } else {
            (prefix.clone(), prefix)
        };

        if let Some(p) = path_ids.as_mut() {
            p.push(lines.len())
        }
        lines.push(Box::new(PathDisplay::new(
            Some(cur_p),
            &self.path,
            &self.linked,
            self.count(),
            !config().select,
            false,
        )?));

        if !config().files {
            for (i, line) in self.lines.iter().enumerate() {
                let prefix = if i + 1 != self.lines.len() {
                    with_push(line_p.clone(), PrefixComponent::MatchWithNext)
                } else {
                    with_push(line_p.clone(), PrefixComponent::MatchNoNext)
                };
                lines.push(Box::new(LineDisplay {
                    prefix,
                    content: &line.content,
                    path: &self.path,
                    matches: &line.matches,
                    line_num: line.line_num,
                    new_line: !config().select,
                }));
            }
        }
        Ok(())
    }

    fn count(&self) -> usize {
        self.lines.iter().map(|l| l.matches.len()).sum()
    }
}

pub fn matches_to_display_lines<'a>(
    result: &'a Matches,
    mut path_ids: Option<&mut Vec<usize>>,
) -> Result<Vec<Box<dyn Entry + 'a>>, Message> {
    let mut lines: Vec<Box<dyn Entry + 'a>> = Vec::new();
    let mut overview: Option<Box<OverviewDisplay>> = config()
        .overview
        .then(|| OverviewDisplay {
            dirs: 0,
            files: usize::from(!config().is_dir),
            lines: 0,
            count: 0,
            new_line: !config().select,
        })
        .map(Box::new);
    match &result {
        Matches::Dir(dirs) => {
            dirs.first().unwrap().to_lines(
                &mut lines,
                Vec::new(),
                Vec::new(),
                dirs,
                &mut path_ids,
                &mut overview.as_mut(),
            )?;
        }
        Matches::File(file) => {
            file.to_lines(
                &mut lines,
                Vec::new(),
                false,
                &mut path_ids,
                &mut overview.as_mut(),
            )?;
        }
    }
    if let Some(o) = overview.take() {
        lines.push(o);
    }
    Ok(lines)
}

pub fn write_results<'a>(out: &mut Term, lines: &[Box<dyn Entry + 'a>]) -> io::Result<()> {
    for line in lines {
        write!(out, "{}", line)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_name() {
        let mut path = Path::new("/path/to/file.txt");
        assert_eq!(path_name(path).ok(), Some("file.txt".to_string()));

        path = Path::new("/path/to/unicode_åß∂ƒ.txt");
        assert_eq!(path_name(path).ok(), Some("unicode_åß∂ƒ.txt".to_string()));

        path = Path::new("/path/to/directory/");
        assert_eq!(path_name(path).ok(), Some("directory".to_string()));

        path = Path::new("/");
        assert_eq!(path_name(path).ok(), None);
    }
}
