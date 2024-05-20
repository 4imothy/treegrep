// SPDX-License-Identifier: CC-BY-4.0

use core::panic;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const OVERWRITE: bool = false;

pub fn normalize_newlines(contents: &mut Vec<u8>) {
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

pub fn get_target_contents(path: PathBuf) -> Vec<u8> {
    let mut contents: Vec<u8> = fs::read(&path).unwrap();

    normalize_newlines(&mut contents);

    contents
}

pub fn check_results(tar_path: PathBuf, rg_results: Vec<u8>, tg_results: Vec<u8>) {
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

        assert_eq!(contents, rg_results);
        assert_eq!(contents, tg_results);
    }
}

pub fn get_outputs(path: &Path, expr: &str, extra_option: Option<&str>) -> (Vec<u8>, Vec<u8>) {
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

    tg.arg("--no-color");
    tg_on_rg.arg("--no-color");
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

    let mut rg_results: Vec<u8> = rg_out.stdout;
    let mut tg_results: Vec<u8> = tg_out.stdout;

    normalize_newlines(&mut rg_results);
    normalize_newlines(&mut tg_results);

    (rg_results, tg_results)
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
