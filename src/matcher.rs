// SPDX-License-Identifier: MIT

use crate::{
    config::Config,
    errors::Message,
    match_system::{Directory, File, Line, Match, Matches, wrap_dirs, wrap_file},
    mes,
};
use crossbeam_channel::{Receiver, Sender};
use grep::{
    matcher::LineTerminator,
    regex::{RegexMatcher, RegexMatcherBuilder},
    searcher::{
        BinaryDetection, MmapChoice, Searcher, SearcherBuilder, Sink, SinkContext, SinkMatch,
    },
};
use ignore::{WalkBuilder, WalkState, overrides::OverrideBuilder};
use std::{
    collections::HashMap,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

enum Matcher {
    Regex {
        combined: RegexMatcher,
        individual: Vec<regex::bytes::Regex>,
    },
    #[cfg(feature = "pcre2")]
    Pcre2 {
        combined: grep_pcre2::RegexMatcher,
        individual: Vec<pcre2::bytes::Regex>,
    },
}

impl Matcher {
    fn new(patterns: &[String], use_pcre2: bool) -> Result<Self, Message> {
        #[cfg(feature = "pcre2")]
        if use_pcre2 {
            let combined = grep_pcre2::RegexMatcherBuilder::new()
                .build_many(patterns)
                .map_err(|e| mes!("pcre2 expression is invalid: {}", e))?;
            let individual = patterns
                .iter()
                .map(|p| {
                    pcre2::bytes::RegexBuilder::new()
                        .jit_if_available(true)
                        .build(p)
                        .map_err(|_| mes!("pcre2 expression `{}` is invalid", p))
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Self::Pcre2 {
                combined,
                individual,
            });
        }
        #[cfg(not(feature = "pcre2"))]
        if use_pcre2 {
            return Err(mes!("PCRE2 is not available in this build"));
        }

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
        Ok(Self::Regex {
            combined,
            individual,
        })
    }

    fn find_all_matches(&self, bytes: &[u8], buf: &mut Vec<Match>) -> Result<(), io::Error> {
        match self {
            Self::Regex { individual, .. } => {
                for (id, regex) in individual.iter().enumerate() {
                    for m in regex.find_iter(bytes) {
                        buf.push(Match::new(id, m.start(), m.end()));
                    }
                }
            }
            #[cfg(feature = "pcre2")]
            Self::Pcre2 { individual, .. } => {
                for (id, regex) in individual.iter().enumerate() {
                    for m in regex.find_iter(bytes) {
                        let m = m.map_err(io::Error::other)?;
                        buf.push(Match::new(id, m.start(), m.end()));
                    }
                }
            }
        }
        Ok(())
    }

    fn search_into(&self, searcher: &mut Searcher, pb: &Path, sink: &mut MatchSink<'_>) -> bool {
        match self {
            Self::Regex { combined, .. } => searcher.search_path(combined, pb, sink).is_err(),
            #[cfg(feature = "pcre2")]
            Self::Pcre2 { combined, .. } => searcher.search_path(combined, pb, sink).is_err(),
        }
    }
}

struct MatchSink<'a> {
    lines: Vec<Line>,
    matcher: &'a Matcher,
    match_buf: Vec<Match>,
    match_error: Option<String>,
}

fn strip_line_ending(bytes: &[u8]) -> &[u8] {
    bytes
        .strip_suffix(b"\r\n")
        .or_else(|| bytes.strip_suffix(b"\n"))
        .unwrap_or(bytes)
}

impl<'a> Sink for MatchSink<'a> {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, io::Error> {
        let line_bytes = mat.bytes();
        let line_num = mat.line_number().unwrap_or(0) as usize;
        let trimmed_bytes = strip_line_ending(line_bytes);

        self.match_buf.clear();
        if let Err(e) = self
            .matcher
            .find_all_matches(trimmed_bytes, &mut self.match_buf)
        {
            self.match_error = Some(e.to_string());
            return Ok(false);
        }

        if !self.match_buf.is_empty() {
            let content = trimmed_bytes.to_vec();
            let n = self.match_buf.len();
            let matches = std::mem::replace(&mut self.match_buf, Vec::with_capacity(n));
            self.lines.push(Line::new(content, matches, line_num));
        }

        Ok(true)
    }

    fn context(&mut self, _searcher: &Searcher, ctx: &SinkContext<'_>) -> Result<bool, io::Error> {
        let line_bytes = ctx.bytes();
        let trimmed_bytes = strip_line_ending(line_bytes);
        let content = trimmed_bytes.to_vec();
        let line_num = ctx.line_number().unwrap_or(0) as usize;
        self.lines.push(Line::new_context(content, line_num));
        Ok(true)
    }
}

pub fn search(abort: Arc<AtomicBool>, config: Arc<Config>) -> Result<Option<Matches>, Message> {
    let matchers = Matcher::new(&config.search.regexps, config.search.pcre2)?;
    let mut searcher = SearcherBuilder::new()
        .line_number(true)
        .line_terminator(LineTerminator::byte(b'\n'))
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .memory_map(unsafe { MmapChoice::auto() })
        .before_context(config.search.before_context)
        .after_context(config.search.after_context)
        .build();

    if config.is_dir {
        let matchers = Arc::new(matchers);
        Ok(wrap_dirs(search_dir(
            &matchers,
            &mut searcher,
            abort,
            config,
        )?))
    } else {
        Ok(wrap_file(
            search_file(
                config.search.path.to_path_buf(),
                &matchers,
                &mut searcher,
                &config,
            )?,
            config.search.files && config.search.regexps.is_empty(),
        ))
    }
}

fn search_dir(
    matchers: &Arc<Matcher>,
    searcher: &mut Searcher,
    abort: Arc<AtomicBool>,
    config: Arc<Config>,
) -> Result<Vec<Directory>, Message> {
    let path = config.search.path.clone();
    let globs = config.search.globs.clone();
    search_dir_impl(&path, &globs, matchers, searcher, abort, config)
}

fn search_dir_impl(
    path: &PathBuf,
    globs: &[String],
    matchers: &Arc<Matcher>,
    searcher: &mut Searcher,
    abort: Arc<AtomicBool>,
    config: Arc<Config>,
) -> Result<Vec<Directory>, Message> {
    let mut override_builder = OverrideBuilder::new(path);
    for glob in globs {
        override_builder
            .add(glob)
            .map_err(|_| mes!("glob {} is invalid", glob))?;
    }

    let walker = WalkBuilder::new(path)
        .hidden(!config.search.hidden)
        .max_depth(config.search.max_depth)
        .follow_links(config.search.links)
        .ignore(config.search.ignore)
        .git_global(config.search.ignore)
        .git_ignore(config.search.ignore)
        .git_exclude(config.search.ignore)
        .require_git(false)
        .threads(config.threads)
        .overrides(
            override_builder
                .build()
                .map_err(|_| mes!("failed to build override builder with given globs"))?,
        )
        .build_parallel();

    let (tx, rx): (Sender<File>, Receiver<File>) = crossbeam_channel::unbounded();
    let first_err: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    walker.run(|| {
        let tx = tx.clone();
        let matchers = Arc::clone(matchers);
        let mut searcher = searcher.clone();
        let abort = Arc::clone(&abort);
        let config = Arc::clone(&config);
        let first_err = Arc::clone(&first_err);

        Box::new(move |entry_result| {
            if abort.load(Ordering::Relaxed) {
                return WalkState::Quit;
            }

            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };

            let is_file = entry.file_type().is_some_and(|ft| ft.is_file());
            if !is_file {
                return WalkState::Continue;
            }

            let path = entry.into_path();
            match search_file(path, &matchers, &mut searcher, &config) {
                Ok(Some(file)) => {
                    let _ = tx.send(file);
                }
                Ok(None) => {}
                Err(e) => {
                    if let Ok(mut guard) = first_err.lock()
                        && guard.is_none()
                    {
                        *guard = Some(e.mes);
                    }
                }
            }

            WalkState::Continue
        })
    });

    drop(tx);
    let dirs = build_directory_tree(rx, path, config.search.links)?;
    if let Some(mes) = first_err.lock().ok().and_then(|mut g| g.take()) {
        return Err(Message { mes });
    }
    Ok(dirs)
}

fn search_file(
    pb: PathBuf,
    matcher: &Matcher,
    searcher: &mut Searcher,
    config: &Config,
) -> Result<Option<File>, Message> {
    if config.search.files && config.search.regexps.is_empty() {
        return Ok(Some(File::from_pathbuf(pb, config.search.links)?));
    }

    let mut sink = MatchSink {
        lines: Vec::with_capacity(32),
        match_buf: Vec::with_capacity(8),
        matcher,
        match_error: None,
    };

    if matcher.search_into(searcher, &pb, &mut sink) {
        return Ok(None);
    }
    if let Some(e) = sink.match_error {
        return Err(mes!("pcre2 match error on {}: {}", pb.display(), e));
    }

    if sink.lines.is_empty() {
        return Ok(None);
    }

    if config.search.before_context > 0 || config.search.after_context > 0 {
        Line::compute_context_offsets(&mut sink.lines);
    }

    let mut file = File::from_pathbuf(pb, config.search.links)?;
    file.lines = sink.lines;
    Ok(Some(file))
}

fn build_directory_tree(
    rx: Receiver<File>,
    root_path: &Path,
    links: bool,
) -> Result<Vec<Directory>, Message> {
    let mut path_to_index: HashMap<OsString, usize> = HashMap::new();
    let mut directories: Vec<Directory> = Vec::new();

    path_to_index.insert(root_path.as_os_str().to_owned(), 0);
    directories.push(Directory::new(root_path, links)?);

    for file in rx {
        if let Some(dir_path) = file.path.parent() {
            let dir_idx = get_or_create_directory(
                &mut path_to_index,
                &mut directories,
                dir_path,
                root_path,
                links,
            )?;
            directories[dir_idx].files.push(file);
        }
    }

    Ok(directories)
}

fn get_or_create_directory(
    path_to_index: &mut HashMap<OsString, usize>,
    directories: &mut Vec<Directory>,
    dir_path: &std::path::Path,
    root_path: &Path,
    links: bool,
) -> Result<usize, Message> {
    if let Some(&idx) = path_to_index.get(dir_path.as_os_str()) {
        return Ok(idx);
    }

    let parent_idx = if let Some(parent) = dir_path.parent() {
        if parent == root_path || path_to_index.contains_key(parent.as_os_str()) {
            path_to_index.get(parent.as_os_str()).copied()
        } else {
            Some(get_or_create_directory(
                path_to_index,
                directories,
                parent,
                root_path,
                links,
            )?)
        }
    } else {
        None
    };

    let new_idx = directories.len();
    path_to_index.insert(dir_path.as_os_str().to_owned(), new_idx);
    directories.push(Directory::new(dir_path, links)?);

    if let Some(p_idx) = parent_idx {
        directories[p_idx].children.push(new_idx);
    }

    Ok(new_idx)
}
