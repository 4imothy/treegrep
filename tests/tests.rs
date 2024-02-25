// SPDX-License-Identifier: CC-BY-4.0

// testing methed relies on paths being searched in the same order
// which is not always the case so we
// can't have a directory that has more than one file in it

mod file_system;
mod utils;
use file_system::Dir;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use utils::*;

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
fn file() {
    let tar_dir: PathBuf = target_dir();

    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");
    let dir = Dir::new("file");
    let file = dir.path.join("actual_file");
    dir.create_file_bytes(&file.to_string_lossy(), pool);

    let tar_path = tar_dir.join("file");
    let (rg_results, tg_results) = get_outputs(&file, "hat", Some("--line-number"));
    check_results(tar_path, rg_results, tg_results);
}

#[cfg(windows)]
const PLATFORM_SUFFIX: &str = "windows";
#[cfg(target_os = "macos")]
const PLATFORM_SUFFIX: &str = "macos";
#[cfg(target_os = "linux")]
const PLATFORM_SUFFIX: &str = "linux";

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

    let tar_path = tar_dir.join(format!("links_{}", PLATFORM_SUFFIX));
    let (_, results) = get_outputs(&dir.path, ".", Some("--links"));

    if OVERWRITE {
        let mut file = fs::File::create(tar_path).unwrap();
        file.write_all(&results).unwrap();
    } else {
        let contents = get_target_contents(tar_path);

        println!("file contents");
        println!("{}", String::from_utf8_lossy(&contents));
        println!("tg output");
        println!("{}", String::from_utf8_lossy(&results));

        assert_eq!(contents, results);
    }
}
