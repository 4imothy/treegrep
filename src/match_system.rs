// SPDX-License-Identifier: MIT

use crate::config;
use crate::errors::Message;
use std::path::{Path, PathBuf};

pub fn wrap_dirs(dirs: Vec<Directory>) -> Option<Matches> {
    if dirs.first().unwrap().children.is_empty() && dirs.first().unwrap().files.is_empty() {
        return None;
    }
    Some(Matches::Dir(dirs))
}

pub fn wrap_file(file: Option<File>, just_files: bool) -> Option<Matches> {
    file.filter(|f| !f.lines.is_empty() || just_files)
        .map(Matches::File)
}

pub enum Matches {
    Dir(Vec<Directory>),
    File(File),
}

pub struct Directory {
    pub path: PathBuf,
    pub linked: Option<PathBuf>,
    pub children: Vec<usize>,
    pub files: Vec<File>,
    pub to_add: bool,
}

impl Directory {
    pub fn new(path: &Path) -> Result<Self, Message> {
        Ok(Directory {
            path: path.to_path_buf(),
            linked: get_linked(path),
            children: Vec::new(),
            files: Vec::new(),
            to_add: true,
        })
    }
}

pub struct File {
    pub path: PathBuf,
    pub lines: Vec<Line>,
    pub linked: Option<PathBuf>,
}

impl File {
    pub fn new(path: &Path) -> Result<Self, Message> {
        Ok(File {
            linked: get_linked(path),
            path: path.to_path_buf(),
            lines: Vec::new(),
        })
    }
}

fn get_linked(path: &Path) -> Option<PathBuf> {
    if !config().links {
        return None;
    }
    path.read_link().ok().map(|link| {
        std::env::var("HOME")
            .ok()
            .filter(|home| link.starts_with(home))
            .and_then(|home| link.strip_prefix(&home).ok())
            .map(|clean| PathBuf::from("~").join(clean))
            .unwrap_or(link)
    })
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

    fn remove_overlapping(matches: &mut [Match]) {
        if matches.is_empty() {
            return;
        }
        matches.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));
        let mut current_max_end = matches[0].end;
        for m in matches.iter_mut().skip(1) {
            if m.start <= current_max_end {
                m.start = current_max_end;
                m.end = current_max_end.max(m.end);
            }
            current_max_end = current_max_end.max(m.end);
        }
    }
}

pub struct Line {
    pub content: String,
    pub matches: Vec<Match>,
    pub line_num: usize,
}

impl Line {
    pub fn new(content: String, mut matches: Vec<Match>, line_num: usize) -> Self {
        Match::remove_overlapping(&mut matches);
        Line {
            content,
            matches,
            line_num,
        }
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
}
