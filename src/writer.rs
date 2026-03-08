// SPDX-License-Identifier: MIT

use crate::{
    config,
    errors::Message,
    match_system::{Directory, File, Match, Matches},
    mes, style,
    term::{TERM_WIDTH, Term},
};
use core::fmt::{self, Display};
use crossterm::style::Color;
use std::{
    borrow::Cow,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

fn search_matches_in(text: &str, search: &str) -> Vec<(usize, usize)> {
    if search.is_empty() {
        return Vec::new();
    }
    let lower = text.to_lowercase();
    let mut result = Vec::new();
    let mut pos = 0;
    while let Some(found) = lower[pos..].find(search) {
        let start = pos + found;
        let end = start + search.len();
        result.push((start, end));
        pos = end;
    }
    result
}

enum HighlightEvent<'a> {
    RegexStart(&'a Match),
    RegexEnd,
    SearchStart,
    SearchEnd,
}

impl HighlightEvent<'_> {
    fn priority(&self) -> u8 {
        match self {
            Self::RegexEnd => 0,
            Self::SearchEnd => 1,
            Self::SearchStart => 2,
            Self::RegexStart(_) => 3,
        }
    }
}

fn write_segment(
    f: &mut fmt::Formatter,
    seg: &str,
    regex_m: Option<&Match>,
    in_search: bool,
    text_fg: Option<Color>,
) -> fmt::Result {
    let sh = config().colors.search_highlight;
    match (regex_m, in_search) {
        (Some(m), true) => {
            let fg = config().colors.matches[m.regexp_id % config().colors.matches.len()];
            write!(f, "{}", style::style_with_on(seg, fg, sh))
        }
        (Some(m), false) => write!(f, "{}", style::match_substring(seg, m.regexp_id)),
        (None, true) => match text_fg {
            Some(c) => write!(f, "{}", style::style_with_on(seg, c, sh)),
            None => write!(f, "{}", style::style_on(seg, sh)),
        },
        (None, false) => match text_fg {
            Some(c) => write!(f, "{}", style::style_with(seg, c)),
            None => f.write_str(seg),
        },
    }
}

fn write_content_with_highlights(
    f: &mut fmt::Formatter,
    content: &str,
    matches: &[Match],
    cut: usize,
    text_fg: Option<Color>,
    search: &str,
) -> fmt::Result {
    let search_ranges = search_matches_in(content, search);

    let mut events: Vec<(usize, HighlightEvent)> = Vec::new();
    for m in matches {
        if m.start < m.end {
            let ms = m.start.saturating_sub(cut);
            let me = m.end.saturating_sub(cut).min(content.len());
            if ms < content.len() {
                events.push((ms, HighlightEvent::RegexStart(m)));
                events.push((me, HighlightEvent::RegexEnd));
            }
        }
    }
    for &(ss, se) in &search_ranges {
        events.push((ss, HighlightEvent::SearchStart));
        events.push((se, HighlightEvent::SearchEnd));
    }
    events.sort_by_key(|(pos, e)| (*pos, e.priority()));

    let mut pos = 0;
    let mut regex_m: Option<&Match> = None;
    let mut in_search = false;
    for (event_pos, event_type) in &events {
        if *event_pos > pos {
            write_segment(f, &content[pos..*event_pos], regex_m, in_search, text_fg)?;
            pos = *event_pos;
        }
        match event_type {
            HighlightEvent::RegexStart(m) => regex_m = Some(m),
            HighlightEvent::RegexEnd => regex_m = None,
            HighlightEvent::SearchStart => in_search = true,
            HighlightEvent::SearchEnd => in_search = false,
        }
    }
    if pos < content.len() {
        write_segment(f, &content[pos..], regex_m, in_search, text_fg)?;
    }
    Ok(())
}

#[derive(Clone)]
pub enum PrefixComponent {
    MatchWithNext,
    MatchNoNext,
    SpacerVert,
    Spacer,
}

pub struct OpenInfo<'a> {
    pub path: &'a Path,
    pub line: Option<usize>,
}

pub trait Entry: Display {
    fn set_search(&mut self, search: &str);
    fn open_info(&self) -> Result<OpenInfo<'_>, Message>;
    fn depth(&self) -> usize;
    fn is_path(&self) -> bool;
    fn search_text(&self) -> Cow<'_, str>;
}

struct PathDisplay<'a> {
    prefix: Option<Vec<PrefixComponent>>,
    name: &'a str,
    color: Color,
    path: &'a Path,
    linked: Option<&'a str>,
    count: usize,
    new_line: bool,
    search: String,
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
        let color = if dir {
            config().colors.dir
        } else {
            config().colors.file
        };
        let name = path_name(path)?;
        let linked = linked.as_ref().map(|l| path_name(l)).transpose()?;
        Ok(PathDisplay {
            prefix,
            name,
            color,
            path,
            linked,
            count,
            new_line,
            search: String::new(),
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
    fn is_path(&self) -> bool {
        true
    }
    fn set_search(&mut self, search: &str) {
        self.search = search.to_string();
    }
    fn search_text(&self) -> Cow<'_, str> {
        match self.linked {
            Some(l) => Cow::Owned(format!("{} {}", self.name, l)),
            None => Cow::Borrowed(self.name),
        }
    }
}

impl<'a> Display for PathDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(p) = &self.prefix {
            write_prefix(f, p)?;
        }
        write_path(
            f,
            self.name,
            self.color,
            self.linked,
            self.count,
            self.new_line,
            &self.search,
        )
    }
}

fn write_path(
    f: &mut fmt::Formatter,
    name: &str,
    color: Color,
    linked: Option<&str>,
    count: usize,
    new_line: bool,
    search: &str,
) -> fmt::Result {
    if cfg!(feature = "test") {
        write!(f, "[ps]{}[pe]", name)?;
        if let Some(l) = linked {
            write!(f, " -> [ps]{}[pe]", l)?;
        }
    } else {
        write_content_with_highlights(f, name, &[], 0, Some(color), search)?;
        if let Some(l) = linked {
            f.write_str(" -> ")?;
            write_content_with_highlights(f, l, &[], 0, Some(color), search)?;
        }
    }
    if config().count && count > 0 {
        write!(f, ": {}", count)?;
    }
    if config().with_colors || config().with_bold {
        write!(f, "{}", style::RESET)?;
    }
    if new_line {
        writeln!(f)?;
    }
    Ok(())
}

fn write_prefix(f: &mut fmt::Formatter, prefix_components: &[PrefixComponent]) -> fmt::Result {
    let mut prefix: String = String::new();
    prefix.reserve(config().chars.spacer.len() * prefix_components.len());
    for c in prefix_components {
        prefix.push_str(match c {
            PrefixComponent::MatchWithNext => &config().chars.match_with_next,
            PrefixComponent::MatchNoNext => &config().chars.match_no_next,
            PrefixComponent::SpacerVert => &config().chars.spacer_vert,
            PrefixComponent::Spacer => &config().chars.spacer,
        });
    }
    if let Some(c) = config().colors.branch {
        write!(f, "{}", style::style_with(&prefix, c))
    } else {
        f.write_str(&prefix)
    }
}

struct LineDisplay<'a> {
    prefix: Vec<PrefixComponent>,
    content: &'a str,
    path: &'a Path,
    matches: &'a [Match],
    line_num: usize,
    context_offset: Option<isize>,
    new_line: bool,
    search: String,
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
    fn is_path(&self) -> bool {
        false
    }
    fn set_search(&mut self, search: &str) {
        self.search = search.to_string();
    }
    fn search_text(&self) -> Cow<'_, str> {
        Cow::Borrowed(if config().trim {
            self.content.trim_start()
        } else {
            self.content
        })
    }
}

impl<'a> Display for LineDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_prefix(f, &self.prefix)?;
        if config().with_colors || config().with_bold {
            write!(f, "{}", style::RESET)?;
        }

        let mut content: &str = self.content;

        let cut = if config().trim {
            let trimmed = content.trim_start();
            let cut = content.len() - trimmed.len();
            content = trimmed;
            cut
        } else {
            0
        };

        if config().select {
            let term_width = TERM_WIDTH.load(Ordering::SeqCst) as usize;
            let prefix_len = config().prefix_len * self.prefix.len();
            let content_width = term_width.saturating_sub(prefix_len);
            let cap = config()
                .max_length
                .map_or(content_width, |m| (m + prefix_len).min(content_width));
            content = &content[..content.floor_char_boundary(cap.min(content.len()))];
        } else if let Some(max) = config().max_length {
            content = &content[..content.floor_char_boundary(max.min(content.len()))];
        }

        if config().line_number {
            let n = self
                .context_offset
                .map_or_else(|| self.line_num.to_string(), |o| format!("{:+}", o));
            write!(f, "{}: ", style::style_with(n, config().colors.line_number))?;
        }

        if self.context_offset.is_some() && config().with_colors {
            write!(f, "{}", style::DIM)?;
        }

        if cfg!(feature = "test") {
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
                    let text = &content[last..m_start];
                    if let Some(c) = config().colors.text {
                        write!(f, "{}", style::style_with(text, c))?;
                    } else {
                        f.write_str(text)?;
                    }
                }
                let styled = style::match_substring(&content[m_start..m_end], m.regexp_id);
                write!(f, "[m{}s]{}[m{}e]", m.regexp_id, styled, m.regexp_id)?;
                last = m_end;
            }
            if last < content.len() {
                let text = &content[last..];
                if let Some(c) = config().colors.text {
                    write!(f, "{}", style::style_with(text, c))?;
                } else {
                    f.write_str(text)?;
                }
            }
        } else {
            write_content_with_highlights(
                f,
                content,
                self.matches,
                cut,
                config().colors.text,
                &self.search,
            )?;
        }

        if self.context_offset.is_some() && config().with_colors {
            write!(f, "{}", style::RESET)?;
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
    search: String,
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
    fn is_path(&self) -> bool {
        true
    }
    fn set_search(&mut self, search: &str) {
        self.search = search.to_string();
    }
    fn search_text(&self) -> Cow<'_, str> {
        let parts: Vec<&str> = self
            .files
            .iter()
            .flat_map(|f| std::iter::once(f.name).chain(f.linked))
            .collect();
        if parts.len() == 1 {
            Cow::Borrowed(parts[0])
        } else {
            Cow::Owned(parts.join(" "))
        }
    }
}

impl<'a> Display for LongBranchDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_prefix(f, &self.prefix)?;
        if config().with_colors || config().with_bold {
            write!(f, "{}", style::RESET)?;
        }
        for (i, file) in self.files.iter().enumerate() {
            write_path(
                f,
                file.name,
                file.color,
                file.linked,
                file.count,
                false,
                &self.search,
            )?;
            if i + 1 != self.files.len() {
                f.write_str(style::LONG_BRANCH_FILE_SEPARATOR)?;
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
    fn is_path(&self) -> bool {
        false
    }
    fn search_text(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }
    fn set_search(&mut self, _: &str) {}
}

impl Display for OverviewDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if config().with_colors || config().with_bold {
            write!(f, "{}", style::RESET)?;
        }
        if !config().files {
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

fn path_name(path: &Path) -> Result<&str, Message> {
    let err = || {
        mes!(
            "failed to get name of `{}`",
            path.as_os_str().to_string_lossy()
        )
    };
    path.file_name().ok_or_else(err)?.to_str().ok_or_else(err)
}

fn with_push(prefix: &[PrefixComponent], item: PrefixComponent) -> Vec<PrefixComponent> {
    let mut v = prefix.to_vec();
    v.push(item);
    v
}

impl Directory {
    fn to_lines<'a>(
        &'a self,
        lines: &mut Vec<Box<dyn Entry + 'a>>,
        cur_prefix: &[PrefixComponent],
        child_prefix: &[PrefixComponent],
        dirs: &'a Vec<Directory>,
        overview: &mut Option<&mut Box<OverviewDisplay>>,
    ) -> Result<(), Message> {
        let children = &self.children;
        let files = &self.files;
        let flen = files.len();
        let clen = children.len();
        if clen > 0 || flen > 0 {
            lines.push(Box::new(PathDisplay::new(
                Some(cur_prefix.to_vec()),
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
                    with_push(child_prefix, PrefixComponent::MatchWithNext),
                    with_push(child_prefix, PrefixComponent::SpacerVert),
                )
            } else {
                (
                    with_push(child_prefix, PrefixComponent::MatchNoNext),
                    with_push(child_prefix, PrefixComponent::Spacer),
                )
            };
            dir.to_lines(lines, &cur_prefix, &new_child_prefix, dirs, overview)?;
        }
        if !files.is_empty() {
            if config().long_branch {
                self.long_branch_files_to_lines(lines, child_prefix)?;
            } else {
                for (i, file) in files.iter().enumerate() {
                    file.to_lines(lines, child_prefix, i + 1 != flen, overview)?;
                }
            }
        }
        Ok(())
    }

    fn long_branch_files_to_lines<'a>(
        &'a self,
        lines: &mut Vec<Box<dyn Entry + 'a>>,
        prefix: &[PrefixComponent],
    ) -> Result<(), Message> {
        let long_branch_files_per_line: usize = config().long_branch_each;
        let num_lines: usize = self.files.len().div_ceil(long_branch_files_per_line);

        let prefix_no_next = with_push(prefix, PrefixComponent::MatchNoNext);
        let prefix_next = with_push(prefix, PrefixComponent::MatchWithNext);

        for (i, branch) in self.files.chunks(long_branch_files_per_line).enumerate() {
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
                search: String::new(),
            }));
        }
        Ok(())
    }
}

impl File {
    fn to_lines<'a>(
        &'a self,
        lines: &mut Vec<Box<dyn Entry + 'a>>,
        prefix: &[PrefixComponent],
        parent_has_next: bool,
        overview: &mut Option<&mut Box<OverviewDisplay>>,
    ) -> Result<(), Message> {
        if let Some(o) = overview {
            o.lines += self
                .lines
                .iter()
                .filter(|l| l.context_offset.is_none())
                .count();
            o.count += self.lines.iter().map(|l| l.matches.len()).sum::<usize>();
        }
        let (cur_p, line_p) = if config().is_dir {
            if parent_has_next {
                (
                    with_push(prefix, PrefixComponent::MatchWithNext),
                    with_push(prefix, PrefixComponent::SpacerVert),
                )
            } else {
                (
                    with_push(prefix, PrefixComponent::MatchNoNext),
                    with_push(prefix, PrefixComponent::Spacer),
                )
            }
        } else {
            (prefix.to_vec(), prefix.to_vec())
        };

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
                    with_push(&line_p, PrefixComponent::MatchWithNext)
                } else {
                    with_push(&line_p, PrefixComponent::MatchNoNext)
                };
                lines.push(Box::new(LineDisplay {
                    prefix,
                    content: &line.content,
                    path: &self.path,
                    matches: &line.matches,
                    line_num: line.line_num,
                    context_offset: line.context_offset,
                    new_line: !config().select,
                    search: String::new(),
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
            dirs.first()
                .unwrap()
                .to_lines(&mut lines, &[], &[], dirs, &mut overview.as_mut())?;
        }
        Matches::File(file) => {
            file.to_lines(&mut lines, &[], false, &mut overview.as_mut())?;
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
        assert_eq!(path_name(path).ok(), Some("file.txt"));

        path = Path::new("/path/to/unicode_åß∂ƒ.txt");
        assert_eq!(path_name(path).ok(), Some("unicode_åß∂ƒ.txt"));

        path = Path::new("/path/to/directory/");
        assert_eq!(path_name(path).ok(), Some("directory"));

        path = Path::new("/");
        assert_eq!(path_name(path).ok(), None);
    }
}
