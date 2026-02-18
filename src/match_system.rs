// SPDX-License-Identifier: MIT

use crate::{config, errors::Message};
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
}

impl Directory {
    pub fn new(path: &Path) -> Result<Self, Message> {
        Ok(Self {
            path: path.to_path_buf(),
            linked: get_linked(path),
            children: Vec::new(),
            files: Vec::new(),
        })
    }
}

pub struct File {
    pub path: PathBuf,
    pub lines: Vec<Line>,
    pub linked: Option<PathBuf>,
}

impl File {
    pub fn from_pathbuf(path: PathBuf) -> Result<Self, Message> {
        Ok(Self {
            linked: get_linked(&path),
            path,
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

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Match {
    pub regexp_id: usize,
    pub start: usize,
    pub end: usize,
}

impl Match {
    pub fn new(regexp_id: usize, start: usize, end: usize) -> Self {
        Self {
            regexp_id,
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
    pub context_offset: Option<isize>,
}

impl Line {
    pub fn new(content: String, mut matches: Vec<Match>, line_num: usize) -> Self {
        Match::remove_overlapping(&mut matches);
        Self {
            content,
            matches,
            line_num,
            context_offset: None,
        }
    }

    pub fn new_context(content: String, line_num: usize) -> Self {
        Self {
            content,
            matches: Vec::new(),
            line_num,
            context_offset: None,
        }
    }

    pub fn compute_context_offsets(lines: &mut [Line]) {
        let mut anchor: Option<usize> = None;
        for line in lines.iter_mut() {
            if !line.matches.is_empty() {
                anchor = Some(line.line_num);
            } else if let Some(a) = anchor {
                line.context_offset = Some(line.line_num as isize - a as isize);
            }
        }

        anchor = None;
        for line in lines.iter_mut().rev() {
            if !line.matches.is_empty() {
                anchor = Some(line.line_num);
            } else if let Some(a) = anchor {
                let offset = line.line_num as isize - a as isize;
                if line
                    .context_offset
                    .is_none_or(|e| offset.unsigned_abs() < e.unsigned_abs())
                {
                    line.context_offset = Some(offset);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
