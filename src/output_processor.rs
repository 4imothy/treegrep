// SPDX-License-Identifier: CC-BY-4.0

use crate::config::Config;
use crate::errors;
use crate::formats::{remove_reset_style_prefix, remove_reset_style_suffix};
use crate::match_system::{Directory, File, Line, Matches, SliceExt};
use crate::searchers::Searchers;
use errors::Errors;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

impl Directory {
    fn add_new_directories(
        mut dir_path: PathBuf,
        dirs: &mut Vec<Directory>,
        dir_path_to_idx: &mut HashMap<OsString, usize>,
        mut prev_id: Option<usize>,
    ) -> Result<(), Errors> {
        let mut new_id: usize;
        while let Some(p_parent) = dir_path.parent().map(PathBuf::from) {
            if is_start_path(&p_parent) {
                if let Some(pid) = prev_id {
                    dirs.get_mut(0).unwrap().children.push(pid);
                }
                break;
            }
            let m_idx = dir_path_to_idx.get(p_parent.as_os_str());
            let dir: &mut Directory;
            if m_idx.is_none() {
                let mut n_dir = Directory::new(&p_parent)?;
                new_id = dirs.len();
                dir_path_to_idx.insert(n_dir.path.as_os_str().to_owned(), new_id);
                if let Some(pid) = prev_id {
                    n_dir.children.push(pid);
                }
                prev_id = Some(new_id);
                dirs.push(n_dir);
            } else {
                dir = dirs.get_mut(*m_idx.unwrap()).unwrap();
                if let Some(pid) = prev_id {
                    dir.children.push(pid);
                    prev_id = None;
                }
            }
            dir_path = p_parent.to_path_buf();
        }
        Ok(())
    }
}

impl File {
    fn update_file(
        p: &PathBuf,
        id: usize,
        line: Option<&[u8]>,
        num: Option<usize>,
        dirs: &mut Vec<Directory>,
        dir_path_to_idx: &HashMap<OsString, usize>,
    ) -> Result<PathBuf, Errors> {
        p.parent().map_or(
            Err(Errors::FailedToGetParent {
                info: p.as_os_str().to_owned(),
            }),
            |dir_path| {
                let &mut dir;
                if is_start_path(dir_path) {
                    dir = dirs.get_mut(0).unwrap();
                } else {
                    dir = dirs
                        .get_mut(*dir_path_to_idx.get(dir_path.as_os_str()).unwrap())
                        .unwrap();
                }
                dir.files.get_mut(id).unwrap().add_line(num, line);
                Ok(dir_path.to_path_buf())
            },
        )
    }

    pub fn create_update_file(
        p: &PathBuf,
        line: Option<&[u8]>,
        num: Option<usize>,
        file_path_to_idx: &mut HashMap<OsString, usize>,
        dirs: &mut Vec<Directory>,
        dir_path_to_idx: &mut HashMap<OsString, usize>,
        prev_id: &mut Option<usize>,
        config: &Config,
    ) -> Result<PathBuf, Errors> {
        let mut file = File::new(&p, config)?;
        p.parent().map_or(
            Err(Errors::FailedToGetParent {
                info: p.as_os_str().to_owned(),
            }),
            |dir_path| {
                let dir: &mut Directory;

                if is_start_path(&dir_path) {
                    dir = dirs.get_mut(0).unwrap();
                } else if let Some(id) = dir_path_to_idx.get(dir_path.as_os_str()) {
                    dir = dirs.get_mut(*id).unwrap();
                } else {
                    let mut n_dir = Directory::new(&dir_path)?;
                    dir_path_to_idx.insert(n_dir.path.as_os_str().to_owned(), dirs.len());
                    n_dir.to_add = false;
                    dirs.push(n_dir);
                    *prev_id = Some(dirs.len() - 1);
                    dir = dirs.last_mut().unwrap();
                }
                let id = dir.files.len();
                file_path_to_idx.insert(file.path.as_os_str().to_owned(), id);
                file.add_line(num, line);
                dir.files.push(file);
                Ok(dir_path.to_path_buf())
            },
        )
    }

    fn add_line(&mut self, line_num: Option<usize>, contents: Option<&[u8]>) {
        self.lines
            .push(Line::new(contents.map(|v| v.to_vec()), line_num));
    }
}

fn is_start_path(dir_path: &Path) -> bool {
    dir_path.parent().is_none()
}

fn process_rg_results(results: Vec<u8>, config: &Config) -> Result<Matches, Errors> {
    process_lines(results.split(|&byte| byte == b'\n').collect(), config)
}

fn process_grep_results(results: Vec<u8>, config: &Config) -> Result<Matches, Errors> {
    process_lines(results.split(|&byte| byte == b'\n').collect(), config)
}

fn process_lines(lines: Vec<&[u8]>, config: &Config) -> Result<Matches, Errors> {
    if config.is_dir {
        return Ok(Matches::Dir(process_dir_lines(lines, config)?));
    } else {
        return Ok(Matches::File(process_file_lines(lines, config)?));
    }
}

fn split_line(line: &[u8], num_parts: usize) -> Vec<&[u8]> {
    let mut slices = Vec::new();

    let mut colon_count = 0;
    let mut start = 0;
    let mut skip_first_colon = cfg!(target_os = "windows");

    for (i, &c) in line.iter().enumerate() {
        if c == b':' {
            if skip_first_colon {
                skip_first_colon = false;
            } else {
                if colon_count == num_parts - 1 {
                    break;
                }
                slices.push(&line[start..i]);
                colon_count += 1;
                start = i + 1;
            }
        }
    }

    slices.push(&line[start..]);

    slices
}

fn num_parts() -> usize {
    3
}

fn get_contents<'a>(parts: &'a Vec<&[u8]>, num_parts: usize, config: &Config) -> Option<&'a [u8]> {
    if config.just_files {
        return None;
    }
    parts
        .get(num_parts - 1)
        .map(|v| if config.trim_left { v.trim_left() } else { v })
}

fn get_line_number<'a>(
    parts: &'a Vec<&[u8]>,
    num_parts: usize,
    config: &Config,
) -> Option<Result<usize, std::num::ParseIntError>> {
    if !config.just_files {
        let raw = parts.get(num_parts - 2).unwrap();
        let mut num_string = String::new();
        let mut found_digit = false;
        for c in remove_reset_style_prefix(&String::from_utf8_lossy(raw), config.need_reset_style)
            .chars()
        {
            if c.is_digit(10) {
                num_string.push(c);
                found_digit = true;
            } else if found_digit {
                break;
            }
        }
        Some(num_string.parse())
    } else {
        return None;
    }
}

fn process_file_lines(lines: Vec<&[u8]>, config: &Config) -> Result<File, Errors> {
    let num_paths = 1;
    let num_parts = num_parts() - num_paths;
    let mut f = File::new(&config.path, config)?;

    for line in lines {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<_> = split_line(line, num_parts);
        let contents =
            get_contents(&parts, num_parts, config).ok_or(Errors::BadlyFormattedLine {
                line: String::from_utf8_lossy(line).to_string(),
            })?;
        let line_number: Option<usize> = get_line_number(&parts, num_parts, config)
            .map(|v| {
                v.or(Err(Errors::BadlyFormattedLine {
                    line: String::from_utf8_lossy(line).to_string(),
                }))
            })
            .transpose()?;

        f.add_line(line_number, Some(contents));
    }
    Ok(f)
}

fn process_dir_lines(lines: Vec<&[u8]>, config: &Config) -> Result<Vec<Directory>, Errors> {
    let num_parts = num_parts();
    let mut start_path_str = config.path.as_os_str().to_str().unwrap_or("").to_string();
    if config.is_dir {
        start_path_str.push(std::path::MAIN_SEPARATOR);
    }
    let mut file_path_to_idx: HashMap<OsString, usize> = HashMap::new();
    let mut dirs: Vec<Directory> = Vec::new();
    let mut dir_path_to_idx: HashMap<OsString, usize> = HashMap::new();
    dirs.push(Directory::new(&config.path)?);
    dir_path_to_idx.insert(config.path.as_os_str().to_owned(), 0);
    for line in lines.iter().filter(|&line| line.len() > 0) {
        let parts: Vec<_> = split_line(line, num_parts);
        let p: PathBuf = PathBuf::from(
            remove_reset_style_suffix(
                remove_reset_style_prefix(
                    &String::from_utf8_lossy(parts.get(0).unwrap()),
                    config.need_reset_style,
                ),
                config.need_reset_style,
            )
            .to_string()
            .trim_start_matches(&start_path_str),
        );

        let mut prev_id: Option<usize> = None;
        let contents: Option<&[u8]> = if !config.just_files {
            Some(
                get_contents(&parts, num_parts, config).ok_or(Errors::BadlyFormattedLine {
                    line: String::from_utf8_lossy(line).to_string(),
                })?,
            )
        } else {
            None
        };
        let line_number = get_line_number(&parts, num_parts, config)
            .map(|v| {
                v.or(Err(Errors::BadlyFormattedLine {
                    line: String::from_utf8_lossy(line).to_string(),
                }))
            })
            .transpose()?;

        let dir_path: PathBuf;
        if let Some(f_id) = file_path_to_idx.get(p.as_os_str()) {
            dir_path = File::update_file(
                &p,
                *f_id,
                contents,
                line_number,
                &mut dirs,
                &dir_path_to_idx,
            )?;
        } else {
            dir_path = File::create_update_file(
                &p,
                contents,
                line_number,
                &mut file_path_to_idx,
                &mut dirs,
                &mut dir_path_to_idx,
                &mut prev_id,
                config,
            )?;
        }
        Directory::add_new_directories(dir_path, &mut dirs, &mut dir_path_to_idx, prev_id)?;
    }

    Ok(dirs)
}

pub fn process_results(results: Vec<u8>, config: &Config) -> Result<Matches, Errors> {
    match config.exec {
        Searchers::RipGrep => process_rg_results(results, config),
        Searchers::Grep => process_grep_results(results, config),
        Searchers::TreeGrep => {
            return Err(Errors::ProcessingInternalSearcherAsExternal);
        }
    }
}
