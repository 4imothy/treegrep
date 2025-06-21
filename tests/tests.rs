// SPDX-License-Identifier: MIT

mod file_system;
mod utils;
use file_system::Dir;
use std::{env, path::PathBuf};
use utils::*;

#[test]
fn setup() {
    assert!(cfg!(feature = "test"));
}

#[test]
fn wide_directory() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("wide");
    dir.create_file_fill(&PathBuf::from("top_file"), b"text in top level file");

    let first = PathBuf::from("first");
    dir.add_child(&first);
    dir.create_file_fill(&first.join("file_in_first"), b"this is some nice text");
    let first_inner = first.join("inner");
    dir.add_child(&first_inner);
    dir.create_file_fill(
        &first_inner.join("file in first inner"),
        b"text in something else",
    );

    let second = PathBuf::from("second");
    dir.add_child(&second);
    dir.create_file_fill(&second.join("file_in_second"), b"text in second directory");

    let (rg_results, tg_results) = get_outputs(&dir.path, "text");
    let tar_12 = tar_dir.join("wide_1");
    let tar_21 = tar_dir.join("wide_2");
    let tars = [tar_12.as_path(), tar_21.as_path()];
    assert_pass_pool(&tars, rg_results, tg_results);
}

#[test]
fn deep_directory() {
    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");

    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("deep");
    dir.create_file_fill(&PathBuf::from("top_file"), pool);

    let inner_name = PathBuf::from("first");
    dir.add_child(&inner_name);
    dir.create_file_fill(&inner_name.join("first_file"), b"this is some nice text");

    let second = inner_name.join("second");
    dir.add_child(&second);
    dir.create_file_fill(&second.join("2_file"), b"nice text in the second file");

    let third = second.join("third");
    dir.add_child(&third);
    dir.create_file_fill(&third.join("file_3"), b"some nice text in the third file");

    let fourth = third.join("4ourth");
    dir.add_child(&fourth);
    dir.create_file_fill(
        &fourth.join("4ourth_nice_file"),
        b"some not so nice text in the 4th file <0-0> \n this text won't be matched",
    );

    let tar_path = tar_dir.join("deep");
    let (rg_results, tg_results) = get_outputs(&dir.path, "nice");
    assert_pass(&tar_path, rg_results, tg_results);
}

#[test]
fn line_numbers() {
    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");

    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("line_number");
    let inner_name = PathBuf::from("inside");
    dir.add_child(&inner_name);
    dir.create_file_fill(&inner_name.join("alice_two"), pool);

    let tar_path = tar_dir.join("line_number");
    let (rg_results, tg_results) = get_outputs(&dir.path, "Alice --line-number");
    assert_pass(&tar_path, rg_results, tg_results);
}

#[test]
fn max_depth() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("max_depth");
    let inner_name = PathBuf::from("one");
    dir.add_child(&inner_name);
    dir.create_file_fill(&inner_name.join("one_file"), b"shouldn't show");
    dir.create_file_fill(&PathBuf::from("valid_file"), b"should show");

    let tar_path = tar_dir.join("max_depth");
    let (rg_results, tg_results) = get_outputs(&dir.path, ". --max-depth=1");
    assert_pass(&tar_path, rg_results, tg_results);
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
    dir.create_file_fill(&excluded.join("one_file"), b"shouldn't show");
    dir.create_file_fill(&included.join("one_file"), b"should show");

    let (rg_results, tg_results) = get_outputs(
        &dir.path,
        &format!(". --glob=!{}", excluded.to_string_lossy()),
    );
    assert_pass(&tar_path, rg_results, tg_results);
}

#[test]
fn file() {
    let tar_dir: PathBuf = target_dir();

    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");
    let dir = Dir::new("file");
    let file = dir.path.join("actual_file");
    dir.create_file_fill(&file, pool);

    let tar_path = tar_dir.join("file");
    let (rg_results, tg_results) = get_outputs(&file, "hat --line-number");
    assert_pass(&tar_path, rg_results, tg_results);
}

#[test]
fn links() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("links");
    let linked_file = PathBuf::from("linked_file");
    dir.create_file_fill(&linked_file, b"top file content");
    dir.link_file(&linked_file, "link_to_file");

    let linked_dir = PathBuf::from("linked");
    dir.add_child(&linked_dir);
    dir.create_file_fill(&linked_dir.join("file"), b"child file content");
    dir.link_dir(&linked_dir, "link_to_dir");

    let (rg_results, tg_results) = get_outputs(&dir.path, ". --links");
    let tar_1 = tar_dir.join("links_1");
    let tar_2 = tar_dir.join("links_2");
    let tar_3 = tar_dir.join("links_3");
    let tar_4 = tar_dir.join("links_4");
    let tars = [
        tar_1.as_path(),
        tar_2.as_path(),
        tar_3.as_path(),
        tar_4.as_path(),
    ];
    assert_pass_pool(&tars, rg_results, tg_results);
}

#[test]
fn no_matches() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("no_matches");
    let sub = PathBuf::from("sub_dir");
    dir.add_child(&sub);
    dir.create_file_fill(&PathBuf::from("one"), b"some more text");
    dir.create_file_fill(&sub.join("two"), b"some text");

    let tar_path = tar_dir.join("no_matches");
    let (rg_results, tg_results) = get_outputs(&dir.path, "nomatches");
    assert_pass(&tar_path, rg_results, tg_results);

    let (rg_results, tg_results) = get_outputs(&dir.path.join("one"), "nomatches --line-number");
    assert_pass(&tar_path, rg_results, tg_results);
}

#[test]
fn files() {
    let tar_dir: PathBuf = target_dir();
    let text = "some";
    let dir = Dir::new("files");
    dir.create_file_fill(&PathBuf::from("top_file"), text.as_bytes());

    let top = PathBuf::from("top");
    dir.add_child(&top);
    dir.create_file_fill(&top.join("file_in_first"), text.as_bytes());
    let sub = top.join("sub");
    dir.add_child(&sub);
    dir.create_file_fill(&sub.join("file with some text"), text.as_bytes());
    dir.create_file_fill(&sub.join("another file"), b"won't be matched");

    let (rg_results, tg_results) = get_outputs(&dir.path, &format!("{} --files", text));
    let tar = tar_dir.join("files_with_expr");
    assert_pass(&tar, rg_results, tg_results);

    let (rg_results, tg_results) = get_outputs(&dir.path, "--files");
    let tar_1 = tar_dir.join("files_1");
    let tar_2 = tar_dir.join("files_2");
    let tars = [tar_1.as_path(), tar_2.as_path()];
    assert_pass_pool(&tars, rg_results, tg_results);

    let (rg_results, tg_results) = get_outputs(&dir.path, "--files --long-branch");
    assert_pass_pool(
        &[
            tar_dir.join("files_long_branch_1").as_path(),
            tar_dir.join("files_long_branch_2").as_path(),
        ],
        rg_results,
        tg_results,
    );
}

#[test]
fn long_branch_with_expr() {
    let tar_dir: PathBuf = target_dir();
    let text = "some";
    let dir = Dir::new("long_branch_with_expr");
    dir.create_file_fill(&PathBuf::from("top_file"), text.as_bytes());

    let top = PathBuf::from("top");
    dir.add_child(&top);
    dir.create_file_fill(&top.join("file_in_first"), text.as_bytes());
    let sub = top.join("sub");
    dir.add_child(&sub);
    dir.create_file_fill(&sub.join("file with some text"), text.as_bytes());
    dir.create_file_fill(
        &sub.join("another file"),
        format!("{} {}", text, text).as_bytes(),
    );
    dir.create_file_fill(&sub.join("one"), b"ausntha");
    dir.create_file_fill(&sub.join("two"), b"ausntha");

    let (rg_results, tg_results) =
        get_outputs(&dir.path, &format!("{} --files --long-branch", text));
    assert_pass_pool(
        &[
            tar_dir.join("files_long_branch_expr_1").as_path(),
            tar_dir.join("files_long_branch_expr_2").as_path(),
        ],
        rg_results,
        tg_results,
    );

    let (rg_results, tg_results) = get_outputs(
        &dir.path,
        &format!("{} --files --long-branch --count", text),
    );
    assert_pass_pool(
        &[
            tar_dir.join("files_long_branch_expr_count_1").as_path(),
            tar_dir.join("files_long_branch_expr_count_2").as_path(),
        ],
        rg_results,
        tg_results,
    );
}

#[test]
fn overlapping() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("overlapping");
    dir.create_file_fill(&PathBuf::from("top_file"), b"overlapping over overlapping");

    let inner_name = PathBuf::from("first");
    dir.add_child(&inner_name);
    dir.create_file_fill(&inner_name.join("first_file"), b"overlapping");

    let second = inner_name.join("second");
    dir.add_child(&second);
    dir.create_file_fill(&second.join("2_file"), b"overlapping over");

    let (rg_results, tg_results) =
        get_outputs(&dir.path, "--regexp overlapping --regexp over --count");
    assert_pass_single(
        &tar_dir.join("overlapping_rg"),
        "ripgrep output",
        rg_results,
    );
    assert_pass_single(
        &tar_dir.join("overlapping_tgrep"),
        "tgrep output",
        tg_results,
    );
}

#[test]
fn count() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("count");
    dir.create_file_fill(
        &PathBuf::from("file"),
        b"some long text\nwith multiple lines",
    );

    let sub_dir = PathBuf::from("sub");
    dir.add_child(&sub_dir);
    dir.create_file_fill(
        &sub_dir.join("first_file"),
        b"other text\nthat also has multiple lines",
    );

    let sub_sub_dir = sub_dir.join("sub sub");
    dir.add_child(&sub_sub_dir);
    dir.create_file_fill(&sub_sub_dir.join("2_file"), b"even more text but one line");

    let (rg_results, tg_results) = get_outputs(&dir.path, ". --count");
    assert_pass(&tar_dir.join("count"), rg_results, tg_results);
}

#[test]
fn repeat() {
    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");
    let dir = Dir::new("repeat");
    let repeat_file_path = env::temp_dir().join("repeat-file");
    let repeat_file = repeat_file_path.to_string_lossy();

    dir.create_file_fill(&PathBuf::from("top_file"), pool);

    let inner_name = PathBuf::from("first");
    dir.add_child(&inner_name);
    dir.create_file_fill(&inner_name.join("first_file"), b"this is some nice text");

    let second = inner_name.join("second");
    dir.add_child(&second);
    dir.create_file_fill(&second.join("2_file"), b"nice text in the second file");

    let third = second.join("third");
    dir.add_child(&third);
    dir.create_file_fill(&third.join("file_3"), b"some nice text in the third file");

    let fourth = third.join("4ourth");
    dir.add_child(&fourth);
    dir.create_file_fill(
        &fourth.join("4ourth_nice_file"),
        b"    some not so nice text in the 4th file <0-0> \n this text won't be matched",
    );

    let (mut rg_results_orig, mut tg_results_orig) = get_outputs(
        &dir.path,
        &format!(
            "some --line-number --count --glob=!4ourth --repeat-file={} --overview --prefix-len=5 --trim",
            repeat_file
        ),
    );
    let (mut rg_results_rep, mut tg_results_rep) = get_outputs(
        &dir.path,
        &format!("--repeat --repeat-file={}", repeat_file),
    );
    assert!(rg_results_orig == tg_results_orig);
    assert!(rg_results_orig == rg_results_rep);
    assert!(tg_results_orig == tg_results_rep);

    (rg_results_orig, tg_results_orig) = get_outputs(
        &dir.path,
        &format!(
            "--files --glob=!4ourth_nice_file --repeat-file={} --overview --prefix-len=5",
            repeat_file
        ),
    );
    (rg_results_rep, tg_results_rep) = get_outputs(
        &dir.path,
        &format!("--repeat --repeat-file={}", repeat_file),
    );
    assert!(rg_results_orig == tg_results_orig);
    assert!(rg_results_orig == rg_results_rep);
    assert!(tg_results_orig == tg_results_rep);
}
