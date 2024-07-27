// SPDX-License-Identifier: CC-BY-4.0

use core::panic;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const OVERWRITE: bool = false;

fn normalize_newlines(contents: &mut Vec<u8>) {
    let mut i = 0;
    if contents.len() == 0 {
        panic!("empty output can't be normalized");
    }
    while i < contents.len() - 1 {
        if contents[i] == b'\r' && contents[i + 1] == b'\n' {
            contents[i] = b'\n';
            contents.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

fn get_target_contents(path: PathBuf) -> Vec<u8> {
    let mut contents: Vec<u8> = fs::read(&path).unwrap();

    normalize_newlines(&mut contents);

    contents
}

fn check_results(
    tar_path: PathBuf,
    rg_results: &Vec<u8>,
    tg_results: &Vec<u8>,
    single_poss_tar: bool,
) -> Option<(bool, bool)> {
    if OVERWRITE {
        let mut file = fs::File::create(tar_path).unwrap();
        file.write_all(&rg_results).unwrap();
    } else {
        let contents = get_target_contents(tar_path);
        let rg_str = String::from_utf8_lossy(&rg_results);
        let tg_str = String::from_utf8_lossy(&tg_results);
        let contents_str = String::from_utf8_lossy(&contents);
        println!("file contents");
        println!("{}", contents_str);
        println!("rg output");
        println!("{}", rg_str);
        println!("tg output");
        println!("{}", tg_str);

        if single_poss_tar {
            let pass = contents == *tg_results && contents == *rg_results;
            assert!(pass);
        } else {
            return Some((*tg_results == contents, *rg_results == contents));
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

    tg_on_rg.arg("--searcher=rg");

    tg.arg("--searcher=tgrep");
    let rg_out = tg_on_rg.output().ok().unwrap();

    if !rg_out.status.success() && rg_out.stderr.len() > 0 {
        panic!("cmd failed {}", String::from_utf8_lossy(&rg_out.stderr));
    }

    let tg_out = tg.output().ok().unwrap();
    if !tg_out.status.success() && rg_out.stderr.len() > 0 {
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
