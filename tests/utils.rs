// SPDX-License-Identifier: MIT

use core::panic;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const OVERWRITE: bool = false;

fn normalize_newlines(content: &mut Vec<u8>) {
    let mut i = 0;
    if content.is_empty() {
        panic!("empty output can't be normalized");
    }
    while i < content.len() - 1 {
        if content[i] == b'\r' && content[i + 1] == b'\n' {
            content[i] = b'\n';
            content.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

fn get_target_content(path: PathBuf) -> Vec<u8> {
    let mut content: Vec<u8> = fs::read(path).unwrap();

    normalize_newlines(&mut content);

    content
}

fn check_results(
    tar_path: PathBuf,
    rg_results: &Vec<u8>,
    tg_results: &Vec<u8>,
    single_poss_tar: bool,
) -> Option<(bool, bool)> {
    if OVERWRITE {
        let mut file = fs::File::create(tar_path).unwrap();
        file.write_all(rg_results).unwrap();
    } else {
        let content = get_target_content(tar_path);
        let content_str = String::from_utf8_lossy(&content);
        if *tg_results != content {
            let tg_str = String::from_utf8_lossy(tg_results);
            print_diff(&tg_str, "tgrep output", &content_str);
        }
        if *rg_results != content {
            let rg_str = String::from_utf8_lossy(rg_results);
            println!("rg output");
            println!("{}", rg_str);
            print_diff(&rg_str, "rg output", &content_str);
        }

        if single_poss_tar {
            assert!(content == *tg_results);
            assert!(content == *rg_results);
        } else {
            return Some((*tg_results == content, *rg_results == content));
        }
    }
    None
}

pub fn assert_pass(tar_path: PathBuf, rg_results: Vec<u8>, tg_results: Vec<u8>) {
    check_results(tar_path, &rg_results, &tg_results, true);
}

pub fn assert_pass_pool(tar_paths: Vec<PathBuf>, rg_results: Vec<u8>, tg_results: Vec<u8>) {
    let mut tg_has_match = false;
    let mut rg_has_match = false;
    for tar_path in tar_paths {
        if let Some((tg_match, rg_match)) = check_results(tar_path, &rg_results, &tg_results, false)
        {
            if !tg_has_match && tg_match {
                tg_has_match = true;
            }
            if !rg_has_match && rg_match {
                rg_has_match = true;
            }
        }
    }
    assert!(rg_has_match && tg_has_match)
}

pub fn get_outputs(path: &Path, expr: &str, extra_option: Option<&str>) -> (Vec<u8>, Vec<u8>) {
    env::set_var("TREEGREP_DEFAULT_OPTS", "");
    let cmd_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!("tgrep{}", env::consts::EXE_SUFFIX));

    let mut tg_on_rg: Command;
    let mut tg: Command;
    match cross_runner() {
        None => {
            tg_on_rg = Command::new(&cmd_path);
            tg = Command::new(&cmd_path);
        }
        Some(runner) => {
            tg_on_rg = Command::new(&runner);
            tg_on_rg.arg(&cmd_path);
            tg = Command::new(&runner);
            tg.arg(&cmd_path);
        }
    }

    let destlye_args = ["--no-color", "--no-bold"];
    tg.args(destlye_args);
    tg_on_rg.args(destlye_args);
    if let Some(o) = extra_option {
        tg_on_rg.arg(o);
        tg.arg(o);
    }
    tg_on_rg.arg(expr);
    tg.arg(expr);

    tg_on_rg.arg(path);
    tg.arg(path);

    tg.arg("--searcher=tgrep");
    tg_on_rg.arg("--searcher=rg");

    let rg_out = tg_on_rg.output().ok().unwrap();

    if !rg_out.status.success() && !rg_out.stderr.is_empty() {
        panic!("cmd failed {}", String::from_utf8_lossy(&rg_out.stderr));
    }

    let tg_out = tg.output().ok().unwrap();
    if !tg_out.status.success() && !rg_out.stderr.is_empty() {
        panic!("cmd failed {}", String::from_utf8_lossy(&rg_out.stderr));
    }
    let mut rg_stdout: Vec<u8> = rg_out.stdout;
    let rg_stderr: Vec<u8> = rg_out.stderr;
    if !rg_stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&rg_stderr));
    }

    let mut tg_stdout: Vec<u8> = tg_out.stdout;
    let tg_stderr: Vec<u8> = tg_out.stderr;
    if !tg_stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&tg_stderr));
    }

    normalize_newlines(&mut rg_stdout);
    normalize_newlines(&mut tg_stdout);

    (rg_stdout, tg_stdout)
}

pub fn cross_runner() -> Option<String> {
    let runner = std::env::var("CROSS_RUNNER").ok()?;
    if runner.is_empty() {
        return None;
    }
    if cfg!(target_arch = "powerpc64") {
        Some("qemu-ppc64".to_string())
    } else if cfg!(target_arch = "x86") {
        Some("i386".to_string())
    } else {
        Some(format!("qemu-{}", std::env::consts::ARCH))
    }
}

pub fn target_dir() -> PathBuf {
    let p = env::current_dir().unwrap().join("tests").join("targets");
    if !p.exists() {
        fs::create_dir_all(&p).unwrap();
    }
    p
}

fn print_diff(output: &str, output_name: &str, target: &str) {
    println!("target content");
    println!("{}", target);
    println!("{}", output_name);
    println!("{}", output);
    println!("diff:");
    let target_lines: Vec<&str> = target.lines().collect();
    let output_lines: Vec<&str> = output.lines().collect();
    let max_lines = target_lines.len().max(output_lines.len());

    for i in 0..max_lines {
        let target_line: &str = target_lines.get(i).unwrap_or(&"");
        let output_line: &str = output_lines.get(i).unwrap_or(&"");
        if target_line != output_line {
            println!("line {}:", i + 1);
            println!("  target: {}", target_line);
            println!("  output: {}", output_line);
        }
    }
}
