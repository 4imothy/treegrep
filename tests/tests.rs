// SPDX-License-Identifier: CC-BY-4.0

mod file_system;
mod utils;
use file_system::Dir;
use std::path::PathBuf;
use utils::*;

#[test]
fn wide_directory_tree() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("wide");
    dir.create_file_str(&PathBuf::from("top_file"), "text in top level file");

    let first = PathBuf::from("first");
    dir.add_child(&first);
    dir.create_file_str(&first.join("file_in_first"), "this is some nice text");
    let first_inner = first.join("inner");
    dir.add_child(&first_inner);
    dir.create_file_str(
        &first_inner.join("file in first inner"),
        "text in something else",
    );

    let second = PathBuf::from("second");
    dir.add_child(&second);
    dir.create_file_str(&second.join("file_in_second"), "text in second directory");

    let (rg_results, tg_results) = get_outputs(&dir.path, "text", None);
    let tar_12 = tar_dir.join(format!("wide_12"));
    let tar_21 = tar_dir.join(format!("wide_21"));
    let tars = vec![tar_12, tar_21];
    assert_pass_pool(tars, rg_results, tg_results);
}

#[test]
fn deep_directory_tree() {
    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");

    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("deep");
    dir.create_file_bytes(&PathBuf::from("top_file"), pool);

    let inner_name = PathBuf::from("first");
    dir.add_child(&inner_name);
    dir.create_file_str(&inner_name.join("first_file"), "this is some nice text");

    let second = inner_name.join("second");
    dir.add_child(&second);
    dir.create_file_str(&second.join("2_file"), "nice text in the second file");

    let third = second.join("third");
    dir.add_child(&third);
    dir.create_file_str(&third.join("file_3"), "some nice text in the third file");

    let fourth = third.join("4ourth");
    dir.add_child(&fourth);
    dir.create_file_str(
        &fourth.join("4ourth_nice_file"),
        "some not so nice text in the 4th file <0-0> \n this text won't be matched",
    );

    let tar_path = tar_dir.join("deep");
    let (rg_results, tg_results) = get_outputs(&dir.path, "nice", None);
    assert_pass(tar_path, rg_results, tg_results);
}

#[test]
fn line_numbers() {
    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");

    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("line_number");
    let inner_name = PathBuf::from("inside");
    dir.add_child(&inner_name);
    dir.create_file_bytes(&inner_name.join("alice_two"), pool);

    let tar_path = tar_dir.join("line_number");
    let (rg_results, tg_results) = get_outputs(&dir.path, "Alice", Some("--line-number"));
    assert_pass(tar_path, rg_results, tg_results);
}

#[test]
fn max_depth() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("max_depth");
    let inner_name = PathBuf::from("one");
    dir.add_child(&inner_name);
    dir.create_file_str(&inner_name.join("one_file"), "shouldn't show");
    dir.create_file_str(&PathBuf::from("valid_file"), "should show");

    let tar_path = tar_dir.join("max_depth");
    let (rg_results, tg_results) = get_outputs(&dir.path, ".", Some("--max-depth=1"));
    assert_pass(tar_path, rg_results, tg_results);
}

#[test]
fn glob_exclusion() {
    let tar_dir: PathBuf = target_dir();
    let tar_path = tar_dir.join("glob_exclusion");
    let dir = Dir::new("glob_exclusion");
    let excluded = PathBuf::from("excluded");
    dir.add_child(&excluded);
    let included = PathBuf::from("included");
    dir.add_child(&included);
    dir.create_file_str(&excluded.join("one_file"), "shouldn't show");
    dir.create_file_str(&included.join("one_file"), "should show");

    let (rg_results, tg_results) = get_outputs(
        &dir.path,
        ".",
        Some(&("--glob=!".to_string() + &excluded.to_string_lossy())),
    );
    assert_pass(tar_path, rg_results, tg_results);
}

#[test]
fn file() {
    let tar_dir: PathBuf = target_dir();

    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");
    let dir = Dir::new("file");
    let file = dir.path.join("actual_file");
    dir.create_file_bytes(&file, pool);

    let tar_path = tar_dir.join("file");
    let (rg_results, tg_results) = get_outputs(&file, "hat", Some("--line-number"));
    assert_pass(tar_path, rg_results, tg_results);
}

#[test]
fn links() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("links");
    let linked_file = PathBuf::from("linked_file");
    dir.create_file_str(&linked_file, "top file contents");
    dir.link_file(&linked_file, "link_to_file");

    let linked_dir = PathBuf::from("linked");
    dir.add_child(&linked_dir);
    dir.create_file_str(&linked_dir.join("file"), "child file contents");
    dir.link_dir(&linked_dir, "link_to_dir");

    let (rg_results, tg_results) = get_outputs(&dir.path, ".", Some("--links"));
    let tar_11 = tar_dir.join(format!("links_11"));
    let tar_12 = tar_dir.join(format!("links_12"));
    let tar_21 = tar_dir.join(format!("links_21"));
    let tar_22 = tar_dir.join(format!("links_22"));
    let tars = vec![tar_11, tar_12, tar_21, tar_22];
    assert_pass_pool(tars, rg_results, tg_results);
}
