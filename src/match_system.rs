// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::errors::Errors;
use crate::formats::{bold, get_color, reset_bold_and_fg};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn wrap_dirs(dirs: Vec<Directory>) -> Option<Matches> {
    if dirs.get(0).unwrap().children.is_empty() && dirs.get(0).unwrap().files.is_empty() {
        return None;
    }
    Some(Matches::Dir(dirs))
}

pub fn wrap_file(file: Option<File>) -> Option<Matches> {
    file.filter(|f| !f.lines.is_empty()).map(Matches::File)
}

fn path_name(path: &Path) -> Result<String, Errors> {
    let name = path.file_name().ok_or(Errors::FailedToGetName {
        info: path.as_os_str().to_owned(),
    })?;

    name.to_os_string()
        .into_string()
        .map_err(|_| Errors::FailedToGetName {
            info: path.as_os_str().to_owned(),
        })
}

pub enum Matches {
    Dir(Vec<Directory>),
    File(File),
}

pub struct Directory {
    pub name: String,
    pub path: PathBuf,
    pub linked: Option<OsString>,
    pub children: Vec<usize>,
    pub files: Vec<File>,
    pub to_add: bool,
}

impl Directory {
    pub fn new(path: &Path, config: &Config) -> Result<Self, Errors> {
        Ok(Directory {
            name: path_name(path)?,
            path: path.to_path_buf(),
            linked: get_linked(path, config),
            children: Vec::new(),
            files: Vec::new(),
            to_add: true,
        })
    }
}

pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub lines: Vec<Line>,
    pub linked: Option<OsString>,
}

impl File {
    pub fn new(path: &Path, config: &Config) -> Result<Self, Errors> {
        Ok(File {
            name: path_name(path)?,
            linked: get_linked(path, config),
            path: path.to_path_buf(),
            lines: Vec::new(),
        })
    }
}

fn get_linked(path: &Path, config: &Config) -> Option<OsString> {
    if config.links {
        if let Some(p_str) = path.as_os_str().to_str() {
            PathBuf::from(p_str)
                .read_link()
                .ok()
                .and_then(|target_path| match std::env::var("HOME") {
                    Ok(home) => {
                        if target_path.starts_with(&home) {
                            target_path
                                .strip_prefix(&home)
                                .ok()
                                .map(|clean_path| PathBuf::from("~").join(clean_path))
                        } else {
                            Some(target_path)
                        }
                    }
                    Err(_) => Some(target_path),
                })
                .map(|v| v.as_os_str().to_owned())
        } else {
            None
        }
    } else {
        None
    }
}

pub struct Match {
    pub pattern_id: usize,
    pub start: usize,
    pub end: usize,
}

impl Match {
    pub fn new(pattern_id: usize, start: usize, end: usize) -> Self {
        Match {
            pattern_id,
            start,
            end,
        }
    }

    fn remove_overlapping(matches: &mut Vec<Match>) {
        matches.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));
        let mut current_max_end = matches[0].end;
        for m_id in 1..matches.len() {
            if matches[m_id].start <= current_max_end {
                matches[m_id].start = current_max_end;
                matches[m_id].end = current_max_end.max(matches[m_id].end);
            }
            current_max_end = current_max_end.max(matches[m_id].end);
        }
    }
}

pub struct Line {
    pub line_num: Option<usize>,
    pub contents: Option<Vec<u8>>,
}

impl Line {
    pub fn new(contents: Option<Vec<u8>>, line_num: Option<usize>) -> Self {
        Line { contents, line_num }
    }

    pub fn style_line(
        mut contents: &[u8],
        mut matches: Vec<Match>,
        line_num: usize,
        config: &Config,
    ) -> Self {
        let cut;
        if config.trim {
            (contents, cut) = contents.trim_left();
        } else {
            cut = 0;
        }
        if let Some(max_len) = config.max_length {
            if max_len < contents.len() {
                contents = &contents[0..max_len];
            }
        }
        if !config.colors {
            return Line::new(Some(contents.to_vec()), Some(line_num));
        }

        Match::remove_overlapping(&mut matches);

        let mut styled_line = contents.to_vec();
        let mut shift = 0;
        for mut m in matches {
            if m.start >= contents.len() {
                break;
            }
            if m.end >= contents.len() {
                m.end = contents.len();
            }
            if cut > m.start || cut > m.end || m.start == m.end {
                continue;
            }
            m.start -= cut;
            m.end -= cut;
            let styler = get_color(m.pattern_id).to_string().into_bytes();
            let mut start = m.start + shift;
            shift += styler.len();
            styled_line.splice(start..start, styler.into_iter());
            start = m.start + shift;
            let bold = bold();
            shift += bold.len();
            styled_line.splice(start..start, bold.into_iter());
            let end = m.end + shift;
            let reset = reset_bold_and_fg();
            shift += reset.len();
            styled_line.splice(end..end, reset.into_iter());
        }

        Line::new(Some(styled_line), Some(line_num))
    }
}

trait SliceExt {
    fn trim_left(&self) -> (&Self, usize);
}

impl SliceExt for [u8] {
    fn trim_left(&self) -> (&[u8], usize) {
        fn is_space(b: u8) -> bool {
            match b {
                b'\t' | b'\n' | b'\x0B' | b'\x0C' | b'\r' | b' ' => true,
                _ => false,
            }
        }

        let start = self
            .iter()
            .take_while(|&&b| -> bool { is_space(b) })
            .count();

        (&self[start..], start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{Debug, Error, Formatter};

    impl PartialEq for Match {
        fn eq(&self, other: &Self) -> bool {
            self.pattern_id == other.pattern_id
                && self.start == other.start
                && self.end == other.end
        }
    }
    impl Debug for Match {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            f.debug_struct("Match")
                .field("pattern_id", &self.pattern_id)
                .field("start", &self.start)
                .field("end", &self.end)
                .finish()
        }
    }

    #[test]
    fn test_remove_overlapping() {
        let pid = 1;
        let mut input = vec![
            Match::new(pid, 0, 5),
            Match::new(pid, 6, 10),
            Match::new(pid, 12, 15),
        ];
        Match::remove_overlapping(&mut input);
        assert_eq!(
            input,
            vec![
                Match::new(pid, 0, 5),
                Match::new(pid, 6, 10),
                Match::new(pid, 12, 15),
            ]
        );

        input = vec![
            Match::new(pid, 0, 5),
            Match::new(pid, 4, 8),
            Match::new(pid, 7, 12),
        ];
        Match::remove_overlapping(&mut input);
        assert_eq!(
            input,
            vec![
                Match::new(pid, 0, 5),
                Match::new(pid, 5, 8),
                Match::new(pid, 8, 12),
            ]
        );

        input = vec![
            Match::new(pid, 0, 10),
            Match::new(pid, 0, 3),
            Match::new(pid, 5, 10),
            Match::new(pid, 11, 12),
        ];
        Match::remove_overlapping(&mut input);
        assert_eq!(
            input,
            vec![
                Match::new(1, 0, 10),
                Match::new(1, 10, 10),
                Match::new(1, 10, 10),
                Match::new(1, 11, 12),
            ]
        );

        input = vec![Match::new(1, 0, 5)];
        Match::remove_overlapping(&mut input);

        assert_eq!(input, vec![Match::new(1, 0, 5),]);
    }

    #[test]
    fn test_path_name() {
        let mut path = Path::new("/path/to/file.txt");
        assert_eq!(path_name(&path).ok(), Some("file.txt".to_string()));

        path = Path::new("/path/to/unicode_åß∂ƒ.txt");
        assert_eq!(path_name(&path).ok(), Some("unicode_åß∂ƒ.txt".to_string()));

        path = Path::new("/path/to/directory/");
        assert_eq!(path_name(&path).ok(), Some("directory".to_string()));

        path = Path::new("/");
        assert_eq!(path_name(&path).ok(), None);
    }

    #[test]
    fn test_trim_left() {
        let bytes: &[u8] = b"    \t  Hello, World!";

        let (trimmed, count) = bytes.trim_left();

        assert_eq!(trimmed, b"Hello, World!");
        assert_eq!(count, 7);
    }
}
