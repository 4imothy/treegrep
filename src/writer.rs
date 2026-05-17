// SPDX-License-Identifier: MIT

use crate::{
    config::Config,
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
    sync::{Arc, atomic::Ordering},
};

fn filter_matches_in(text: &str, filter: &str) -> Vec<(usize, usize)> {
    if filter.is_empty() {
        return Vec::new();
    }
    let filter_lower = filter.to_lowercase();
    let filter_chars: Vec<char> = filter_lower.chars().collect();
    let flen = filter_chars.len();
    let text_chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut result = Vec::new();
    let mut i = 0;
    while i + flen <= text_chars.len() {
        let matched = (0..flen).all(|k| {
            let mut lc = text_chars[i + k].1.to_lowercase();
            lc.next() == Some(filter_chars[k]) && lc.next().is_none()
        });
        if matched {
            let start = text_chars[i].0;
            let end = if i + flen < text_chars.len() {
                text_chars[i + flen].0
            } else {
                text.len()
            };
            result.push((start, end));
            i += flen;
        } else {
            i += 1;
        }
    }
    result
}

enum HighlightEvent<'a> {
    RegexStart(&'a Match),
    RegexEnd,
    FilterStart,
    FilterEnd,
}

impl HighlightEvent<'_> {
    fn priority(&self) -> u8 {
        match self {
            Self::RegexEnd => 0,
            Self::FilterEnd => 1,
            Self::FilterStart => 2,
            Self::RegexStart(_) => 3,
        }
    }
}

fn write_segment(
    f: &mut fmt::Formatter,
    seg: &str,
    regex_m: Option<&Match>,
    in_filter: bool,
    text_fg: Option<Color>,
    config: &Config,
) -> fmt::Result {
    let sh = config.colors.filter_highlight;
    match (regex_m, in_filter) {
        (Some(m), true) => {
            let fg = config.colors.matches[m.regexp_id % config.colors.matches.len()];
            write!(f, "{}", style::style_with_on(seg, fg, sh, config))
        }
        (Some(m), false) => write!(f, "{}", style::match_substring(seg, m.regexp_id, config)),
        (None, true) => match text_fg {
            Some(c) => write!(f, "{}", style::style_with_on(seg, c, sh, config)),
            None => write!(f, "{}", style::style_on(seg, sh, config)),
        },
        (None, false) => match text_fg {
            Some(c) => write!(f, "{}", style::style_with(seg, c, config)),
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
    filter: &str,
    config: &Config,
) -> fmt::Result {
    let filter_ranges = filter_matches_in(content, filter);

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
    for &(ss, se) in &filter_ranges {
        events.push((ss, HighlightEvent::FilterStart));
        events.push((se, HighlightEvent::FilterEnd));
    }
    events.sort_by_key(|(pos, e)| (*pos, e.priority()));

    let mut pos = 0;
    let mut regex_m: Option<&Match> = None;
    let mut in_filter = false;
    for (event_pos, event_type) in &events {
        if *event_pos > pos {
            write_segment(
                f,
                &content[pos..*event_pos],
                regex_m,
                in_filter,
                text_fg,
                config,
            )?;
            pos = *event_pos;
        }
        match event_type {
            HighlightEvent::RegexStart(m) => regex_m = Some(m),
            HighlightEvent::RegexEnd => regex_m = None,
            HighlightEvent::FilterStart => in_filter = true,
            HighlightEvent::FilterEnd => in_filter = false,
        }
    }
    if pos < content.len() {
        write_segment(f, &content[pos..], regex_m, in_filter, text_fg, config)?;
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
    fn render(&self, f: &mut fmt::Formatter, filter: &str) -> fmt::Result;
    fn open_info(&self) -> Result<OpenInfo<'_>, Message>;
    fn depth(&self) -> usize;
    fn is_path(&self) -> bool;
    fn filter_text(&self) -> Cow<'_, str>;
}

pub struct WithFilter<'a> {
    pub entry: &'a dyn Entry,
    pub filter: &'a str,
}

impl Display for WithFilter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entry.render(f, self.filter)
    }
}

struct PathDisplay {
    prefix: Option<Vec<PrefixComponent>>,
    name: String,
    color: Color,
    path: PathBuf,
    linked: Option<String>,
    count: usize,
    new_line: bool,
    config: Arc<Config>,
}

impl PathDisplay {
    fn new(
        prefix: Option<Vec<PrefixComponent>>,
        path: &Path,
        linked: &Option<PathBuf>,
        count: usize,
        new_line: bool,
        dir: bool,
        config: Arc<Config>,
    ) -> Result<PathDisplay, Message> {
        let color = if dir {
            config.colors.dir
        } else {
            config.colors.file
        };
        let name = path_name(path)?.to_owned();
        let linked = linked
            .as_ref()
            .map(|l| path_name(l).map(|s| s.to_owned()))
            .transpose()?;
        Ok(PathDisplay {
            prefix,
            name,
            color,
            path: path.to_path_buf(),
            linked,
            count,
            new_line,
            config,
        })
    }
}

impl Entry for PathDisplay {
    fn render(&self, f: &mut fmt::Formatter, filter: &str) -> fmt::Result {
        if let Some(p) = &self.prefix {
            write_prefix(f, p, &self.config)?;
        }
        write_path(
            f,
            &self.name,
            self.color,
            self.linked.as_deref(),
            self.count,
            filter,
            &self.config,
        )?;
        if self.new_line {
            writeln!(f)?;
        }
        Ok(())
    }
    fn open_info(&self) -> Result<OpenInfo<'_>, Message> {
        Ok(OpenInfo {
            path: &self.path,
            line: None,
        })
    }
    fn depth(&self) -> usize {
        self.prefix.as_ref().map_or(0, |p| p.len())
    }
    fn is_path(&self) -> bool {
        true
    }
    fn filter_text(&self) -> Cow<'_, str> {
        match &self.linked {
            Some(l) => Cow::Owned(format!("{} {}", self.name, l)),
            None => Cow::Borrowed(&self.name),
        }
    }
}

impl Display for PathDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.render(f, "")
    }
}

fn write_path(
    f: &mut fmt::Formatter,
    name: &str,
    color: Color,
    linked: Option<&str>,
    count: usize,
    filter: &str,
    config: &Config,
) -> fmt::Result {
    if cfg!(feature = "test") {
        write!(f, "[ps]{}[pe]", name)?;
        if let Some(l) = linked {
            write!(f, " -> [ps]{}[pe]", l)?;
        }
    } else {
        write_content_with_highlights(f, name, &[], 0, Some(color), filter, config)?;
        if let Some(l) = linked {
            f.write_str(" -> ")?;
            write_content_with_highlights(f, l, &[], 0, Some(color), filter, config)?;
        }
    }
    if config.count && count > 0 {
        write!(f, ": {}", count)?;
    }
    if config.with_colors || config.with_bold {
        write!(f, "{}", style::RESET)?;
    }
    Ok(())
}

fn write_prefix(
    f: &mut fmt::Formatter,
    prefix_components: &[PrefixComponent],
    config: &Config,
) -> fmt::Result {
    let mut prefix: String = String::new();
    prefix.reserve(config.chars.spacer.len() * prefix_components.len());
    for c in prefix_components {
        prefix.push_str(match c {
            PrefixComponent::MatchWithNext => &config.chars.match_with_next,
            PrefixComponent::MatchNoNext => &config.chars.match_no_next,
            PrefixComponent::SpacerVert => &config.chars.spacer_vert,
            PrefixComponent::Spacer => &config.chars.spacer,
        });
    }
    if let Some(c) = config.colors.branch {
        write!(f, "{}", style::style_with(&prefix, c, config))
    } else {
        f.write_str(&prefix)
    }
}

struct LineDisplay {
    prefix: Vec<PrefixComponent>,
    content: String,
    path: PathBuf,
    matches: Vec<Match>,
    line_num: usize,
    context_offset: Option<isize>,
    new_line: bool,
    config: Arc<Config>,
}

impl Entry for LineDisplay {
    fn render(&self, f: &mut fmt::Formatter, filter: &str) -> fmt::Result {
        write_prefix(f, &self.prefix, &self.config)?;
        if self.config.with_colors || self.config.with_bold {
            write!(f, "{}", style::RESET)?;
        }

        let mut content: &str = &self.content;

        let cut = if self.config.trim {
            let trimmed = content.trim_start();
            let cut = content.len() - trimmed.len();
            content = trimmed;
            cut
        } else {
            0
        };

        let line_num = if self.config.line_number {
            Some(
                self.context_offset
                    .map_or_else(|| format!("{}: ", self.line_num), |o| format!("{:+}: ", o)),
            )
        } else {
            None
        };

        if self.config.core.select || self.config.core.menu {
            let term_width = TERM_WIDTH.load(Ordering::SeqCst) as usize;
            let indicator_len = self.config.chars.selected_indicator.chars().count();
            let prefix_len = self.config.prefix_len * self.prefix.len();
            let line_num_len = line_num.as_ref().map_or(0, |n| n.len());
            let content_width =
                term_width.saturating_sub(indicator_len + prefix_len + line_num_len);
            let cap = self
                .config
                .max_length
                .map_or(content_width, |m| m.min(content_width));
            content = &content[..content.floor_char_boundary(cap.min(content.len()))];
        } else if let Some(max) = self.config.max_length {
            content = &content[..content.floor_char_boundary(max.min(content.len()))];
        }

        if let Some(n) = &line_num {
            write!(
                f,
                "{}",
                style::style_with(n.as_str(), self.config.colors.line_number, &self.config)
            )?;
        }

        if self.context_offset.is_some() && self.config.with_colors {
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
                    if let Some(c) = self.config.colors.text {
                        write!(f, "{}", style::style_with(text, c, &self.config))?;
                    } else {
                        f.write_str(text)?;
                    }
                }
                let styled =
                    style::match_substring(&content[m_start..m_end], m.regexp_id, &self.config);
                write!(f, "[m{}s]{}[m{}e]", m.regexp_id, styled, m.regexp_id)?;
                last = m_end;
            }
            if last < content.len() {
                let text = &content[last..];
                if let Some(c) = self.config.colors.text {
                    write!(f, "{}", style::style_with(text, c, &self.config))?;
                } else {
                    f.write_str(text)?;
                }
            }
        } else {
            write_content_with_highlights(
                f,
                content,
                &self.matches,
                cut,
                self.config.colors.text,
                filter,
                &self.config,
            )?;
        }

        if self.context_offset.is_some() && self.config.with_colors {
            write!(f, "{}", style::RESET)?;
        }
        if self.new_line {
            writeln!(f)?;
        }
        Ok(())
    }
    fn open_info(&self) -> Result<OpenInfo<'_>, Message> {
        Ok(OpenInfo {
            path: &self.path,
            line: Some(self.line_num),
        })
    }
    fn depth(&self) -> usize {
        self.prefix.len()
    }
    fn is_path(&self) -> bool {
        false
    }
    fn filter_text(&self) -> Cow<'_, str> {
        Cow::Borrowed(if self.config.trim {
            self.content.trim_start()
        } else {
            &self.content
        })
    }
}

impl Display for LineDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.render(f, "")
    }
}

struct LongBranchDisplay {
    prefix: Vec<PrefixComponent>,
    files: Vec<PathDisplay>,
    new_line: bool,
    config: Arc<Config>,
}

impl Entry for LongBranchDisplay {
    fn render(&self, f: &mut fmt::Formatter, filter: &str) -> fmt::Result {
        write_prefix(f, &self.prefix, &self.config)?;
        if self.config.with_colors || self.config.with_bold {
            write!(f, "{}", style::RESET)?;
        }
        for (i, file) in self.files.iter().enumerate() {
            write_path(
                f,
                &file.name,
                file.color,
                file.linked.as_deref(),
                file.count,
                filter,
                &self.config,
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
    fn filter_text(&self) -> Cow<'_, str> {
        let parts: Vec<&str> = self
            .files
            .iter()
            .flat_map(|f| std::iter::once(f.name.as_str()).chain(f.linked.as_deref()))
            .collect();
        if parts.len() == 1 {
            Cow::Borrowed(parts[0])
        } else {
            Cow::Owned(parts.join(" "))
        }
    }
}

impl Display for LongBranchDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.render(f, "")
    }
}

struct OverviewDisplay {
    dirs: usize,
    files: usize,
    lines: usize,
    count: usize,
    new_line: bool,
    config: Arc<Config>,
}

impl Entry for OverviewDisplay {
    fn render(&self, f: &mut fmt::Formatter, _filter: &str) -> fmt::Result {
        if self.config.with_colors || self.config.with_bold {
            write!(f, "{}", style::RESET)?;
        }
        write!(f, "{}", self.filter_text())?;
        if self.new_line {
            writeln!(f)?;
        }
        Ok(())
    }
    fn open_info(&self) -> Result<OpenInfo<'_>, Message> {
        Err(mes!("can't open stats"))
    }
    fn depth(&self) -> usize {
        0
    }
    fn is_path(&self) -> bool {
        false
    }
    fn filter_text(&self) -> Cow<'_, str> {
        let mut s = String::new();
        if !self.config.files {
            s.push_str(&format!(
                "found {} matches in {} lines across ",
                self.count, self.lines
            ));
        }
        s.push_str(&format!(
            "{} {}",
            self.files,
            if self.files == 1 { "file" } else { "files" }
        ));
        if self.config.is_dir {
            s.push_str(&format!(" within {} directories", self.dirs));
        }
        Cow::Owned(s)
    }
}

impl Display for OverviewDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.render(f, "")
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
    fn to_lines(
        &self,
        lines: &mut Vec<Box<dyn Entry>>,
        cur_prefix: &[PrefixComponent],
        child_prefix: &[PrefixComponent],
        dirs: &Vec<Directory>,
        overview: &mut Option<&mut Box<OverviewDisplay>>,
        config: &Arc<Config>,
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
                !config.core.select && !config.core.menu,
                true,
                Arc::clone(config),
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
            dir.to_lines(
                lines,
                &cur_prefix,
                &new_child_prefix,
                dirs,
                overview,
                config,
            )?;
        }
        if !files.is_empty() {
            if config.branch_each > 1 {
                self.long_branch_files_to_lines(lines, child_prefix, config)?;
            } else {
                for (i, file) in files.iter().enumerate() {
                    file.to_lines(lines, child_prefix, i + 1 != flen, overview, config)?;
                }
            }
        }
        Ok(())
    }

    fn long_branch_files_to_lines(
        &self,
        lines: &mut Vec<Box<dyn Entry>>,
        prefix: &[PrefixComponent],
        config: &Arc<Config>,
    ) -> Result<(), Message> {
        let num_lines: usize = self.files.len().div_ceil(config.branch_each);

        let prefix_no_next = with_push(prefix, PrefixComponent::MatchNoNext);
        let prefix_next = with_push(prefix, PrefixComponent::MatchWithNext);

        for (i, branch) in self.files.chunks(config.branch_each).enumerate() {
            lines.push(Box::new(LongBranchDisplay {
                prefix: if i + 1 == num_lines {
                    prefix_no_next.clone()
                } else {
                    prefix_next.clone()
                },
                files: branch
                    .iter()
                    .map(|f| {
                        PathDisplay::new(
                            None,
                            &f.path,
                            &f.linked,
                            f.count(),
                            false,
                            false,
                            Arc::clone(config),
                        )
                    })
                    .collect::<Result<Vec<PathDisplay>, Message>>()?,
                new_line: !config.core.select && !config.core.menu,
                config: Arc::clone(config),
            }));
        }
        Ok(())
    }
}

impl File {
    fn to_lines(
        &self,
        lines: &mut Vec<Box<dyn Entry>>,
        prefix: &[PrefixComponent],
        parent_has_next: bool,
        overview: &mut Option<&mut Box<OverviewDisplay>>,
        config: &Arc<Config>,
    ) -> Result<(), Message> {
        if let Some(o) = overview {
            o.lines += self
                .lines
                .iter()
                .filter(|l| l.context_offset.is_none())
                .count();
            o.count += self.lines.iter().map(|l| l.matches.len()).sum::<usize>();
        }
        let (cur_p, line_p) = if config.is_dir {
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
            !config.core.select && !config.core.menu,
            false,
            Arc::clone(config),
        )?));

        if !config.files {
            for (i, line) in self.lines.iter().enumerate() {
                let prefix = if i + 1 != self.lines.len() {
                    with_push(&line_p, PrefixComponent::MatchWithNext)
                } else {
                    with_push(&line_p, PrefixComponent::MatchNoNext)
                };
                lines.push(Box::new(LineDisplay {
                    prefix,
                    content: line.content.clone(),
                    path: self.path.clone(),
                    matches: line.matches.clone(),
                    line_num: line.line_num,
                    context_offset: line.context_offset,
                    new_line: !config.core.select && !config.core.menu,
                    config: Arc::clone(config),
                }));
            }
        }
        Ok(())
    }

    fn count(&self) -> usize {
        self.lines.iter().map(|l| l.matches.len()).sum()
    }
}

pub fn matches_to_display_lines(
    result: &Matches,
    config: Arc<Config>,
) -> Result<Vec<Box<dyn Entry>>, Message> {
    let mut lines: Vec<Box<dyn Entry>> = Vec::new();
    let mut overview: Option<Box<OverviewDisplay>> = config
        .overview
        .then(|| OverviewDisplay {
            dirs: 0,
            files: usize::from(!config.is_dir),
            lines: 0,
            count: 0,
            new_line: !config.core.select && !config.core.menu,
            config: Arc::clone(&config),
        })
        .map(Box::new);
    match &result {
        Matches::Dir(dirs) => {
            dirs.first().unwrap().to_lines(
                &mut lines,
                &[],
                &[],
                dirs,
                &mut overview.as_mut(),
                &config,
            )?;
        }
        Matches::File(file) => {
            file.to_lines(&mut lines, &[], false, &mut overview.as_mut(), &config)?;
        }
    }
    if let Some(o) = overview.take() {
        lines.push(o);
    }
    Ok(lines)
}

pub fn write_results(out: &mut Term, lines: &[Box<dyn Entry>]) -> io::Result<()> {
    for line in lines {
        write!(out, "{}", line)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matches_in() {
        assert_eq!(filter_matches_in("Hello World", "hello"), vec![(0, 5)]);
        assert_eq!(filter_matches_in("Hello World", "world"), vec![(6, 11)]);
        assert_eq!(filter_matches_in("aAbBaA", "aa"), vec![(0, 2), (4, 6)]);
        assert_eq!(filter_matches_in("aaa", "aa"), vec![(0, 2)]);
        assert_eq!(filter_matches_in("", "foo"), vec![]);
        assert_eq!(filter_matches_in("foo", ""), vec![]);
        assert_eq!(filter_matches_in("café", "café"), vec![(0, 5)]);
        assert_eq!(filter_matches_in("CAFÉ", "café"), vec![(0, 5)]);
        assert_eq!(filter_matches_in("İstanbul", "istanbul"), vec![]);
        let _ = filter_matches_in("İ", "i");
    }

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
