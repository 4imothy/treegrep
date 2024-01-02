// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::formats;
use crate::match_system::{Directory, File, Line, Match, Matches, SliceExt};
use crate::Errors;
use bstr::ByteSlice;
use ignore::WalkBuilder;
use regex::bytes::Regex;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub fn search(config: &Config) -> Result<Matches, Errors> {
    let mut patterns: Vec<Regex> = Vec::new();
    for expr in &config.patterns {
        patterns.push(Regex::new(expr).map_err(|_| {
            return Errors::InvalidRegex {
                regex: expr.to_string(),
            };
        })?);
    }
    if config.is_dir {
        Ok(Matches::Dir(search_dir(patterns, config)?))
    } else {
        Ok(Matches::File(search_file(&config.path, &patterns, config)?))
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
                        let dir = Directory::new(&path)?;
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
                                let t = *path_to_index.get(par_dir_path.as_os_str()).unwrap();
                                dir = directories.get_mut(t).unwrap();
                                dir.children.push(prev_id);
                                prev_id = t;
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
        let lines = contents.split(|&byte| byte == b'\n');

        let resetter = formats::reset_bold_and_fg().into_bytes();
        let bolder = formats::BOLD.to_string().into_bytes();
        for (line_num, line) in lines
            .map(|v| if config.trim_left { v.trim_left() } else { v })
            .enumerate()
        {
            let mut shift = 0;
            let mut matches: Vec<Match> = Vec::new();
            let mut was_match = false;
            for (j, pattern) in patterns.iter().enumerate() {
                let mut it = pattern.find_iter(&line).peekable();
                if it.peek().is_none() {
                    continue;
                }
                for m in it {
                    if !config.colors {
                        was_match = true;
                        break;
                    }
                    matches.push(Match {
                        pattern_id: j,
                        start: m.start(),
                        end: m.end(),
                    });
                }
            }
            if was_match && !config.colors {
                self.lines
                    .push(Line::new(Some(line.to_vec()), Some(line_num)));
                continue;
            }

            if matches.len() > 0 {
                matches.sort_by_key(|m| m.end);

                let mut m_id = 1;
                while m_id < matches.len() {
                    if matches[m_id].start < matches[m_id - 1].end {
                        matches[m_id].start = matches[m_id - 1].end;
                    }
                    m_id += 1;
                }

                let mut styled_line = line.to_vec();
                for m in matches {
                    let styler = formats::get_color(m.pattern_id).to_string().into_bytes();
                    let mut start = m.start + shift;
                    shift += styler.len();
                    styled_line.splice(start..start, styler.into_iter());
                    start = m.start + shift;
                    styled_line.splice(start..start, bolder.clone().into_iter());
                    shift += bolder.len();
                    let end = m.end + shift;
                    shift += resetter.len();
                    styled_line.splice(end..end, resetter.clone().into_iter());
                }

                self.lines
                    .push(Line::new(Some(styled_line), Some(line_num + 1)));
            }
        }
    }
}
