// SPDX-License-Identifier: MIT

use crate::config;
use crate::errors::{mes, Message};
use crate::formats;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn wrap_dirs(dirs: Vec<Directory>) -> Option<Matches> {
    if dirs.get(0).unwrap().children.is_empty() && dirs.get(0).unwrap().files.is_empty() {
        return None;
    }
    Some(Matches::Dir(dirs))
}

pub fn wrap_file(file: Option<File>, tree: bool) -> Option<Matches> {
    file.filter(|f| !f.lines.is_empty() || tree)
        .map(Matches::File)
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
    pub fn new(path: &Path) -> Result<Self, Message> {
        Ok(Directory {
            name: path_name(path)?,
            path: path.to_path_buf(),
            linked: get_linked(path),
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
    pub fn new(path: &Path) -> Result<Self, Message> {
        Ok(File {
            name: path_name(path)?,
            linked: get_linked(path),
            path: path.to_path_buf(),
            lines: Vec::new(),
        })
    }
}

fn get_linked(path: &Path) -> Option<OsString> {
    if config().links {
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
    pub contents: Vec<u8>,
    pub line_num: usize,
}

impl Line {
    pub fn styled(contents: &[u8], mut matches: Vec<Match>, line_num: usize) -> Self {
        if matches.len() > 0 {
            Match::remove_overlapping(&mut matches);
        }
        formats::style_line(contents, matches, line_num)
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
}
