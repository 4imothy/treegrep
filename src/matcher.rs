// SPDX-License-Identifier: MIT

use crate::{
    config,
    errors::Message,
    match_system::{Directory, File, Line, Match, Matches, wrap_dirs, wrap_file},
    mes,
};
use crossbeam_channel::{Receiver, Sender};
use grep::{
    matcher::LineTerminator,
    regex::{RegexMatcher, RegexMatcherBuilder},
    searcher::{BinaryDetection, MmapChoice, Searcher, SearcherBuilder, Sink, SinkMatch},
};
use ignore::{WalkBuilder, WalkState, overrides::OverrideBuilder};
use std::{collections::HashMap, ffi::OsString, io, path::PathBuf, sync::Arc};

struct Matcher {
    combined: RegexMatcher,
    individual: Vec<regex::bytes::Regex>,
}

impl Matcher {
    fn new(patterns: &[String]) -> Result<Self, Message> {
        let combined = RegexMatcherBuilder::new()
            .line_terminator(Some(b'\n'))
            .build_many(patterns)
            .map_err(|e| mes!("regex expression is invalid: {}", e))?;
        let individual = patterns
            .iter()
            .map(|p| {
                regex::bytes::Regex::new(p).map_err(|_| mes!("regex expression `{}` is invalid", p))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Matcher {
            combined,
            individual,
        })
    }
}

struct MatchSink<'a> {
    lines: Vec<Line>,
    matcher: &'a Matcher,
    match_buf: Vec<Match>,
}

impl<'a> Sink for MatchSink<'a> {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, io::Error> {
        let line_bytes = mat.bytes();
        let line_num = mat.line_number().unwrap_or(0) as usize;
        let trimmed_bytes = line_bytes
            .strip_suffix(b"\r\n")
            .or_else(|| line_bytes.strip_suffix(b"\n"))
            .unwrap_or(line_bytes);

        self.match_buf.clear();
        for (pattern_id, regex) in self.matcher.individual.iter().enumerate() {
            for m in regex.find_iter(trimmed_bytes) {
                self.match_buf
                    .push(Match::new(pattern_id, m.start(), m.end()));
            }
        }

        if !self.match_buf.is_empty() {
            let content = String::from_utf8_lossy(trimmed_bytes).into_owned();
            let matches = std::mem::replace(&mut self.match_buf, Vec::with_capacity(8));
            self.lines.push(Line::new(content, matches, line_num));
        }

        Ok(true)
    }
}

pub fn search() -> Result<Option<Matches>, Message> {
    let matchers = Matcher::new(&config().regexps)?;
    let mut searcher = SearcherBuilder::new()
        .line_number(config().line_number)
        .line_terminator(LineTerminator::byte(b'\n'))
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .memory_map(unsafe { MmapChoice::auto() })
        .build();

    if config().is_dir {
        let matchers = Arc::new(matchers);
        Ok(wrap_dirs(search_dir(&matchers, &mut searcher)?))
    } else {
        Ok(wrap_file(
            search_file(config().path.to_path_buf(), &matchers, &mut searcher)?,
            config().just_files,
        ))
    }
}

fn search_dir(matchers: &Arc<Matcher>, searcher: &mut Searcher) -> Result<Vec<Directory>, Message> {
    let mut override_builder = OverrideBuilder::new(&config().path);
    for glob in &config().globs {
        override_builder
            .add(glob)
            .map_err(|_| mes!("glob {} is invalid", glob))?;
    }

    let walker = WalkBuilder::new(&config().path)
        .hidden(!config().hidden)
        .max_depth(config().max_depth)
        .follow_links(config().links)
        .ignore(config().ignore)
        .git_global(config().ignore)
        .git_ignore(config().ignore)
        .git_exclude(config().ignore)
        .require_git(false)
        .threads(config().threads)
        .overrides(
            override_builder
                .build()
                .map_err(|_| mes!("failed to build override builder with given globs"))?,
        )
        .build_parallel();

    let (tx, rx): (Sender<File>, Receiver<File>) = crossbeam_channel::unbounded();

    walker.run(|| {
        let tx = tx.clone();
        let matchers = Arc::clone(matchers);
        let mut searcher = searcher.clone();

        Box::new(move |entry_result| {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };

            let is_file = entry.file_type().is_some_and(|ft| ft.is_file());
            if !is_file {
                return WalkState::Continue;
            }

            let path = entry.into_path();
            if let Ok(Some(file)) = search_file(path, &matchers, &mut searcher) {
                let _ = tx.send(file);
            }

            WalkState::Continue
        })
    });

    drop(tx);
    build_directory_tree(rx)
}

fn search_file(
    pb: PathBuf,
    matcher: &Matcher,
    searcher: &mut Searcher,
) -> Result<Option<File>, Message> {
    if config().just_files {
        return Ok(Some(File::from_pathbuf(pb)?));
    }

    let mut sink = MatchSink {
        lines: Vec::with_capacity(32),
        match_buf: Vec::with_capacity(8),
        matcher,
    };

    if searcher
        .search_path(&matcher.combined, &pb, &mut sink)
        .is_err()
    {
        return Ok(None);
    }

    if sink.lines.is_empty() {
        return Ok(None);
    }

    let mut file = File::from_pathbuf(pb)?;
    file.lines = sink.lines;
    Ok(Some(file))
}

fn build_directory_tree(rx: Receiver<File>) -> Result<Vec<Directory>, Message> {
    let mut path_to_index: HashMap<OsString, usize> = HashMap::new();
    let mut directories: Vec<Directory> = Vec::new();

    let root_path = config().path.clone();
    path_to_index.insert(root_path.as_os_str().to_owned(), 0);
    directories.push(Directory::new(&root_path)?);

    for file in rx {
        if let Some(dir_path) = file.path.parent() {
            let dir_idx = get_or_create_directory(&mut path_to_index, &mut directories, dir_path)?;
            directories[dir_idx].files.push(file);
        }
    }

    Ok(directories)
}

fn get_or_create_directory(
    path_to_index: &mut HashMap<OsString, usize>,
    directories: &mut Vec<Directory>,
    dir_path: &std::path::Path,
) -> Result<usize, Message> {
    if let Some(&idx) = path_to_index.get(dir_path.as_os_str()) {
        return Ok(idx);
    }

    let parent_idx = if let Some(parent) = dir_path.parent() {
        if parent == config().path || path_to_index.contains_key(parent.as_os_str()) {
            path_to_index.get(parent.as_os_str()).copied()
        } else {
            Some(get_or_create_directory(path_to_index, directories, parent)?)
        }
    } else {
        None
    };

    let new_idx = directories.len();
    path_to_index.insert(dir_path.as_os_str().to_owned(), new_idx);
    directories.push(Directory::new(dir_path)?);

    if let Some(p_idx) = parent_idx {
        directories[p_idx].children.push(new_idx);
    }

    Ok(new_idx)
}
