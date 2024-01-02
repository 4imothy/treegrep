// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::errors::Errors;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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
    pub children: Vec<usize>,
    pub files: Vec<File>,
    pub to_add: bool,
}

impl Directory {
    pub fn new(path: &Path) -> Result<Self, Errors> {
        Ok(Directory {
            name: path_name(path)?,
            path: path.to_path_buf(),
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
        let linked: Option<OsString>;

        if config.links {
            if let Some(p_str) = path.as_os_str().to_str() {
                linked = PathBuf::from(p_str)
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
                    .map(|v| v.as_os_str().to_owned());
            } else {
                linked = None;
            }
        } else {
            linked = None;
        }

        Ok(File {
            name: path_name(path)?,
            linked,
            path: path.to_path_buf(),
            lines: Vec::new(),
        })
    }
}

pub struct Match {
    pub pattern_id: usize,
    pub start: usize,
    pub end: usize,
}

pub struct Line {
    pub line_num: Option<usize>,
    pub contents: Option<Vec<u8>>,
}

impl Line {
    pub fn new(contents: Option<Vec<u8>>, line_num: Option<usize>) -> Self {
        Line { contents, line_num }
    }
}

pub trait SliceExt {
    fn trim_left(&self) -> &Self;
}

impl SliceExt for [u8] {
    // NOTE this has issues with the spaces in the prefix of a line being matched
    // in that case the first character of the line isn't a ` ` it is a coloring character
    fn trim_left(&self) -> &[u8] {
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

        &self[start..]
    }
}
