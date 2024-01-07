// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::formats;
use crate::match_system::{wrap_dirs, wrap_file, Directory, File, Line, Match, Matches};
use crate::Errors;
use bstr::ByteSlice;
use ignore::WalkBuilder;
use regex::bytes::Regex;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub fn search(config: &Config) -> Result<Option<Matches>, Errors> {
    let mut patterns: Vec<Regex> = Vec::new();
    for expr in &config.patterns {
        patterns.push(Regex::new(expr).map_err(|_| {
            return Errors::InvalidRegex {
                regex: expr.to_string(),
            };
        })?);
    }
    if config.is_dir {
        Ok(wrap_dirs(search_dir(patterns, config)?))
    } else {
        Ok(wrap_file(Some(search_file(
            &config.path,
            &patterns,
            config,
        )?)))
    }
}

fn search_dir(patterns: Vec<Regex>, config: &Config) -> Result<Vec<Directory>, Errors> {
    let walker = WalkBuilder::new(&config.path)
        .hidden(!config.hidden)
        .max_depth(config.max_depth)
        .follow_links(config.links)
        .ignore(config.ignore)
        .git_global(config.ignore)
        .git_ignore(config.ignore)
        .git_exclude(config.ignore)
        .build();

    let mut path_to_index: HashMap<OsString, usize> = HashMap::new();
    let mut directories: Vec<Directory> = Vec::new();
    for res in walker {
        match res {
            Ok(entry) => {
                let path = entry.into_path();
                if path.is_dir() {
                    if path_to_index.get(path.as_os_str()).is_none() {
                        path_to_index.insert(path.clone().into_os_string(), directories.len());
                        let dir = Directory::new(&path, config)?;
                        directories.push(dir);
                    }
                } else if path.is_file() {
                    let file = search_file(&path, &patterns, config)?;
                    if file.lines.len() > 0 {
                        if let Some(mut dir_path) = file.path.parent().map(|v| v.to_path_buf()) {
                            let mut prev_id: usize =
                                *path_to_index.get(dir_path.as_os_str()).unwrap();
                            let mut dir: &mut Directory = directories.get_mut(prev_id).unwrap();
                            dir.files.push(file);
                            let mut to_add = dir.to_add;
                            while let Some(par_dir_path) = dir_path.parent() {
                                if !to_add || dir_path == config.path {
                                    break;
                                }
                                dir.to_add = false;
                                let p_id = *path_to_index.get(par_dir_path.as_os_str()).unwrap();
                                dir = directories.get_mut(p_id).unwrap();
                                dir.children.push(prev_id);
                                prev_id = p_id;
                                to_add = dir.to_add;
                                dir_path = par_dir_path.to_path_buf();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(directories)
}

fn search_file(pb: &PathBuf, patterns: &Vec<Regex>, config: &Config) -> Result<File, Errors> {
    let m_content_bytes: Option<Vec<u8>> = fs::read(pb).ok();

    let mut file = File::new(pb, config)?;

    let content_bytes: Vec<u8>;
    match m_content_bytes {
        None => return Ok(file),
        Some(b) => content_bytes = b,
    }

    if content_bytes.find_byte(0).is_some() {
        return Ok(file);
    }

    file.add_matches(content_bytes, patterns, config);

    return Ok(file);
}

impl File {
    fn add_matches(&mut self, contents: Vec<u8>, patterns: &Vec<Regex>, config: &Config) {
        let lines = contents.split_inclusive(|&byte| byte == formats::NEW_LINE as u8);

        for (line_num, line) in lines.enumerate() {
            let mut matches: Vec<Match> = Vec::new();
            let mut was_match = false;
            for (j, pattern) in patterns.iter().enumerate() {
                let mut it = pattern.find_iter(&line).peekable();
                if it.peek().is_none() {
                    continue;
                }
                for m in it {
                    was_match = true;
                    if !config.colors {
                        break;
                    };
                    matches.push(Match::new(j, m.start(), m.end()));
                }
            }
            if was_match {
                self.lines
                    .push(Line::style_line(line, matches, line_num + 1, config));
            }
        }
    }
}
