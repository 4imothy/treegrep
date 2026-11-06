// SPDX-License-Identifier: MIT

use crate::match_system::{Diff, bytes_to_path};
use std::path::PathBuf;

pub struct DiffLine {
    pub content: Vec<u8>,
    pub line_num: usize,
    pub diff: Diff,
}

pub struct DiffFile {
    pub path: PathBuf,
    pub hunks: Vec<Vec<DiffLine>>,
}

fn unquote(quoted: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(quoted.len());
    let mut bytes = quoted.iter().copied();
    while let Some(b) = bytes.next() {
        let escaped = if b == b'\\' { bytes.next() } else { None };
        out.push(match escaped {
            Some(d @ b'0'..=b'7') => {
                let mut n = d - b'0';
                for c in bytes.by_ref().take(2) {
                    n = n.wrapping_mul(8).wrapping_add(c.wrapping_sub(b'0'));
                }
                n
            }
            Some(b'a') => 7,
            Some(b'b') => 8,
            Some(b't') => 9,
            Some(b'n') => 10,
            Some(b'v') => 11,
            Some(b'f') => 12,
            Some(b'r') => 13,
            Some(c) => c,
            None => b,
        });
    }
    out
}

fn diff_path(header: &[u8]) -> Option<PathBuf> {
    let header = header.split(|&b| b == b'\t').next().unwrap_or_default();
    let raw = match header
        .strip_prefix(b"\"")
        .and_then(|h| h.strip_suffix(b"\""))
    {
        Some(quoted) => unquote(quoted),
        None => header.to_vec(),
    };
    let path = match raw.as_slice() {
        b"/dev/null" => return None,
        [
            b'a' | b'b' | b'c' | b'i' | b'o' | b'w' | b'1' | b'2',
            b'/',
            rest @ ..,
        ] => rest,
        p => p,
    };
    Some(bytes_to_path(path.to_vec()))
}

fn hunk_range(range: Option<&str>, sign: char) -> Option<(usize, usize)> {
    let range = range?.strip_prefix(sign)?;
    let (start, len) = range.split_once(',').unwrap_or((range, "1"));
    Some((start.parse().ok()?, len.parse().ok()?))
}

pub fn parse_diff(bytes: &[u8]) -> Vec<DiffFile> {
    let mut files: Vec<DiffFile> = Vec::new();
    let mut new = 0;
    let mut old_left = 0;
    let mut new_left = 0;
    let mut from = None;
    for l in bytes
        .strip_suffix(b"\n")
        .unwrap_or(bytes)
        .split(|&b| b == b'\n')
    {
        let l = l.strip_suffix(b"\r").unwrap_or(l);
        let diff = match l.first() {
            Some(b'+') if new_left > 0 => {
                new_left -= 1;
                Diff::Added
            }
            Some(b'-') if old_left > 0 => {
                old_left -= 1;
                Diff::Removed
            }
            None | Some(b' ') if old_left > 0 && new_left > 0 => {
                old_left -= 1;
                new_left -= 1;
                Diff::Unchanged
            }
            Some(b'\\') => continue,
            _ => {
                old_left = 0;
                new_left = 0;
                if let Some(h) = l.strip_prefix(b"@@ ") {
                    let h = String::from_utf8_lossy(h);
                    let mut ranges = h.split(' ');
                    if let (Some((_, old_len)), Some((start, new_len))) = (
                        hunk_range(ranges.next(), '-'),
                        hunk_range(ranges.next(), '+'),
                    ) {
                        new = if new_len == 0 { start + 1 } else { start };
                        old_left = old_len;
                        new_left = new_len;
                        if let Some(f) = files.last_mut() {
                            f.hunks.push(Vec::new());
                        }
                    }
                } else if let Some(p) = l.strip_prefix(b"--- ") {
                    from = diff_path(p);
                } else if let Some(p) = l.strip_prefix(b"+++ ")
                    && let Some(path) = diff_path(p).or(from.take())
                {
                    files.push(DiffFile {
                        path,
                        hunks: Vec::new(),
                    });
                }
                continue;
            }
        };
        if let Some(hunk) = files.last_mut().and_then(|f| f.hunks.last_mut()) {
            hunk.push(DiffLine {
                content: l.get(1..).unwrap_or_default().to_vec(),
                line_num: new,
                diff,
            });
        }
        if !matches!(diff, Diff::Removed) {
            new += 1;
        }
    }
    files
}
