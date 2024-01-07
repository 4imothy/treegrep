// testing methed relies on paths being searched in the same order
// which is not always the case so we
// can't have a directory that has more than one file in it

const OVERWRITE: bool = false;

mod file_system;
use file_system::Dir;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn deep_directory_tree() {
    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");

    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("deep");
    dir.create_file_bytes("top_file", pool);

    let inner_name = "first".to_string();
    dir.add_child(&inner_name);
    dir.create_file_str(
        &(inner_name.clone() + "/first_file"),
        "this is some nice text",
    );

    let second = inner_name + "/second";
    dir.add_child(&second);
    dir.create_file_str(
        &(second.clone() + "/2_file"),
        "nice text in the second file",
    );

    let third = second + "/third";
    dir.add_child(&third);
    dir.create_file_str(
        &(third.clone() + "/file_3"),
        "some nice text in the third file",
    );

    let fourth = third + "/4ourth";
    dir.add_child(&fourth);
    dir.create_file_str(
        &(fourth.clone() + "/4ourth_nice_file"),
        "some not so nice text in the 4th file <0-0> \n this text won't be matched",
    );

    let tar_path = tar_dir.join("deep");
    let (rg_results, tg_results) = get_outputs(&dir.path, "nice", None);
    check_results(tar_path, rg_results, tg_results);
}

#[test]
fn line_numbers() {
    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");

    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("line_number");
    let inner_name = "inside";
    dir.add_child(inner_name);
    dir.create_file_bytes(
        &PathBuf::from(inner_name)
            .join("alice_two")
            .to_string_lossy(),
        pool,
    );

    let tar_path = tar_dir.join("line_number");
    let (rg_results, tg_results) = get_outputs(&dir.path, "Alice", Some("--line-number"));
    check_results(tar_path, rg_results, tg_results);
}

#[test]
fn max_depth() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("max_depth");
    let inner_name = "one";
    dir.add_child(inner_name);
    dir.create_file_str(&(inner_name.to_string() + "/one_file"), "shouldn't show");
    dir.create_file_str("valid_file", "should show");

    let tar_path = tar_dir.join("max_depth");
    let (rg_results, tg_results) = get_outputs(&dir.path, ".", Some("--max-depth=1"));
    check_results(tar_path, rg_results, tg_results);
}

#[test]
fn links() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("links");
    let linked_file = PathBuf::from("linked_file");
    dir.create_file_str(&linked_file.to_string_lossy(), "top file contents");
    dir.link_file(&linked_file, "link_to_file");

    let linked_dir = PathBuf::from("linked");
    dir.add_child(&linked_dir.to_string_lossy());
    dir.create_file_str(
        &linked_dir.join("file").to_string_lossy(),
        "child file contents",
    );
    dir.link_dir(&linked_dir, "link_to_dir");

    let tar_path;
    if cfg!(windows) {
        tar_path = tar_dir.join("links_windows");
    } else {
        tar_path = tar_dir.join("links");
    }
    let (_, results) = get_outputs(&dir.path, ".", Some("--links"));

    if OVERWRITE {
        let mut file = fs::File::create(tar_path).unwrap();
        file.write_all(&results).unwrap();
    } else {
        let contents = fs::read(&tar_path).unwrap();

        println!("file contents");
        println!("{}", String::from_utf8_lossy(&contents));
        println!("tg output");
        println!("{}", String::from_utf8_lossy(&results));

        assert_eq!(contents, results);
    }
}

fn check_results(tar_path: PathBuf, rg_results: Vec<u8>, tg_results: Vec<u8>) {
    if OVERWRITE {
        let mut file = fs::File::create(tar_path).unwrap();
        file.write_all(&rg_results).unwrap();
    } else {
        let contents = fs::read(&tar_path).unwrap();
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

fn get_outputs(path: &Path, expr: &str, extra_option: Option<&str>) -> (Vec<u8>, Vec<u8>) {
    let cmd_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!("tgrep{}", env::consts::EXE_SUFFIX));

    let mut rg_cmd;
    let mut tg_cmd;
    match cross_runner() {
        None => {
            rg_cmd = Command::new(&cmd_path);
            tg_cmd = Command::new(&cmd_path);
        }
        Some(runner) => {
            rg_cmd = Command::new(&runner);
            rg_cmd.arg(&cmd_path);
            tg_cmd = Command::new(&runner);
            tg_cmd.arg(&cmd_path);
        }
    }
    tg_cmd.arg("--color=never");
    rg_cmd.arg("--color=never");
    if let Some(o) = extra_option {
        rg_cmd.arg(o);
        tg_cmd.arg(o);
    }
    rg_cmd.arg(expr);
    tg_cmd.arg(expr);
    rg_cmd.arg(path);
    tg_cmd.arg(path);

    rg_cmd.arg("--searcher=rg");
    tg_cmd.arg("--searcher=tgrep");
    let rg_out = rg_cmd.output().ok().unwrap();
    if !rg_out.status.success() && rg_out.stderr.len() > 0 {
        panic!("cmd failed {}", String::from_utf8_lossy(&rg_out.stderr));
    }

    let tg_out = tg_cmd.output().ok().unwrap();
    if !tg_out.status.success() && rg_out.stderr.len() > 0 {
        panic!("cmd failed {}", String::from_utf8_lossy(&rg_out.stderr));
    }

    let rg_results: Vec<u8> = rg_out.stdout;
    let tg_results: Vec<u8> = tg_out.stdout;

    (rg_results, tg_results)
}

fn cross_runner() -> Option<String> {
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

fn target_dir() -> PathBuf {
    let p = env::current_dir().unwrap().join("tests").join("targets");
    if !p.exists() {
        fs::create_dir_all(&p).unwrap();
    }
    p
}
