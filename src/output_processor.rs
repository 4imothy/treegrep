// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::errors::Errors;
use crate::formats;
use crate::match_system::{wrap_dirs, wrap_file, Directory, File, Line, Match, Matches};
use crate::Searchers;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn is_start_path(dir_path: &Path, root_path: &Path) -> bool {
    dir_path.parent().is_none() || dir_path == root_path
}

pub fn process_results(results: Vec<u8>, config: &Config) -> Result<Option<Matches>, Errors> {
    let lines = results
        .split(|&byte| byte == formats::NEW_LINE as u8)
        .collect();
    match config.exec {
        Searchers::RipGrep => process_json_lines(lines, config),
        Searchers::TreeGrep => {
            return Err(Errors::ProcessingInternalSearcherAsExternal);
        }
    }
}

impl File {
    fn include(
        path: &str,
        dirs: &mut Vec<Directory>,
        path_to_index: &mut HashMap<OsString, usize>,
        config: &Config,
    ) -> Result<(usize, usize), Errors> {
        let f_path = PathBuf::from(path);
        let file = File::new(&f_path, config)?;

        f_path.parent().map_or(
            Err(Errors::FailedToGetParent {
                info: f_path.as_os_str().to_owned(),
            }),
            |mut dir_path| {
                let d_id: usize;
                let mut to_add_id: Option<usize> = None;

                if is_start_path(&dir_path, &config.path) {
                    d_id = 0;
                } else if let Some(id) = path_to_index.get(dir_path.as_os_str()) {
                    d_id = *id;
                } else {
                    let n_dir = Directory::new(&dir_path)?;
                    path_to_index.insert(n_dir.path.as_os_str().to_owned(), dirs.len());
                    d_id = dirs.len();
                    to_add_id = Some(d_id);
                    dirs.push(n_dir);
                }
                let dir: &mut Directory;
                dir = dirs.get_mut(d_id).unwrap();
                let f_id = dir.files.len();
                dir.files.push(file);

                if !is_start_path(&dir_path, &config.path) {
                    while let Some(p_parent) = dir_path.parent() {
                        if is_start_path(&p_parent, &config.path) {
                            if let Some(id) = to_add_id {
                                dirs.get_mut(0).unwrap().children.push(id);
                            }
                            break;
                        }
                        let m_id = path_to_index.get(p_parent.as_os_str());
                        if m_id.is_none() {
                            let mut n_dir = Directory::new(&p_parent)?;
                            let n_id = dirs.len();
                            path_to_index.insert(n_dir.path.as_os_str().to_owned(), n_id);
                            if let Some(id) = to_add_id {
                                n_dir.children.push(id);
                            }
                            to_add_id = Some(n_id);
                            dirs.push(n_dir);
                        } else {
                            if let Some(id) = to_add_id {
                                dirs.get_mut(*m_id.unwrap()).unwrap().children.push(id);
                            }
                            break;
                        }
                        dir_path = p_parent;
                    }
                }

                Ok((d_id, f_id))
            },
        )
    }
}

trait AsUsize {
    fn as_usize(&self) -> Option<usize>;
}

impl AsUsize for Value {
    fn as_usize(&self) -> Option<usize> {
        match self {
            Value::Number(n) => n.as_u64().map(|v| v as usize),
            _ => None,
        }
    }
}

pub fn process_json_lines(lines: Vec<&[u8]>, config: &Config) -> Result<Option<Matches>, Errors> {
    let mut path_to_index: HashMap<OsString, usize> = HashMap::new();
    let mut dirs: Vec<Directory> = Vec::new();

    dirs.push(Directory::new(&config.path)?);

    let mut cur_file: Option<&mut File> = None;
    let mut d_id = 0;
    let mut f_id;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let res: Value = serde_json::from_slice(line).map_err(|e| Errors::InvalidJson {
            serde_json_mes: e.to_string(),
        })?;
        match res["type"].as_str().unwrap() {
            "begin" => {
                (d_id, f_id) = File::include(
                    res["data"]["path"]["text"].as_str().unwrap(),
                    &mut dirs,
                    &mut path_to_index,
                    config,
                )?;

                cur_file = Some(dirs.get_mut(d_id).unwrap().files.get_mut(f_id).unwrap());
            }
            "match" => {
                let mut matches = Vec::new();
                for m in res["data"]["submatches"].as_array().unwrap() {
                    matches.push(Match::new(
                        0,
                        m["start"].as_usize().unwrap(),
                        m["end"].as_usize().unwrap(),
                    ));
                }

                cur_file.as_mut().unwrap().lines.push(Line::style_line(
                    match res["data"]["lines"]["text"].as_str() {
                        Some(text) => text.as_bytes(),
                        None => res["data"]["lines"]["bytes"].as_str().unwrap().as_bytes(),
                    },
                    matches,
                    res["data"]["line_number"].as_usize().unwrap(),
                    config,
                ));
            }
            "end" => cur_file = None,
            _ => {}
        }
    }
    if config.is_dir {
        Ok(wrap_dirs(dirs))
    } else {
        Ok(wrap_file(dirs.get_mut(d_id).unwrap().files.pop()))
    }
}
