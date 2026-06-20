// SPDX-License-Identifier: MIT

mod file_system;
#[cfg(unix)]
mod tmux;
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

    let result = get_output(&dir.path, "text -e this");
    let tar_12 = tar_dir.join("wide_1");
    let tar_21 = tar_dir.join("wide_2");
    let tars = [tar_12.as_path(), tar_21.as_path()];
    assert_pass_pool(&tars, result);
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
    let result = get_output(&dir.path, "nice");
    assert_pass(&tar_path, result);
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
    let result = get_output(&dir.path, ". --max-depth=1");
    assert_pass(&tar_path, result);
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

    let result = get_output(
        &dir.path,
        &format!(". --glob=!{}", excluded.to_string_lossy()),
    );
    assert_pass(&tar_path, result);
}

#[test]
fn file() {
    let tar_dir: PathBuf = target_dir();

    let pool: &[u8] = include_bytes!("pool/alice_adventures_in_wonderland_by_lewis_carroll.txt");
    let dir = Dir::new("file");
    let file = PathBuf::from("actual_file");
    dir.create_file_fill(&file, pool);

    let tar_path = tar_dir.join("file");
    let result = get_output(&dir.path.join(file), "hat --line-number");
    assert_pass(&tar_path, result);
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

    let result = get_output(&dir.path, ". --links");
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
    assert_pass_pool(&tars, result);
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
    let result = get_output(&dir.path, "nomatches");
    assert_pass(&tar_path, result);

    let result = get_output(&dir.path.join("one"), "nomatches --line-number");
    assert_pass(&tar_path, result);
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

    let results = get_output(&dir.path, &format!("{} --files", text));
    let tar = tar_dir.join("files_with_expr");
    assert_pass(&tar, results);

    let result = get_output(&dir.path, "--files");
    let tar_1 = tar_dir.join("files_1");
    let tar_2 = tar_dir.join("files_2");
    let tars = [tar_1.as_path(), tar_2.as_path()];
    assert_pass_pool(&tars, result);

    let result = get_output(&dir.path, "--files --branch-each 2");
    assert_pass_pool(
        &[
            tar_dir.join("files_long_branch_1").as_path(),
            tar_dir.join("files_long_branch_2").as_path(),
        ],
        result,
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

    let result = get_output(&dir.path, &format!("{} --files --branch-each 5", text));
    assert_pass_pool(
        &[
            tar_dir.join("files_long_branch_expr_1").as_path(),
            tar_dir.join("files_long_branch_expr_2").as_path(),
        ],
        result,
    );

    let result = get_output(
        &dir.path,
        &format!("{} --files --branch-each 5 --count", text),
    );
    assert_pass_pool(
        &[
            tar_dir.join("files_long_branch_expr_count_1").as_path(),
            tar_dir.join("files_long_branch_expr_count_2").as_path(),
        ],
        result,
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

    let result = get_output(&dir.path, "--regexp overlapping --regexp over --count");
    assert_pass(&tar_dir.join("overlapping"), result);
}

#[test]
fn overview() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("overview");
    dir.create_file_fill(
        &PathBuf::from("top_file"),
        b"some text\nmore text\nother line",
    );

    let sub = PathBuf::from("sub");
    dir.add_child(&sub);
    dir.create_file_fill(&sub.join("sub_file"), b"some text here too");

    let result = get_output(&dir.path, "text --overview");
    assert_pass(&tar_dir.join("overview_dir"), result);

    let result = get_output(&dir.path.join("top_file"), "text --overview");
    assert_pass(&tar_dir.join("overview_file"), result);
}

#[test]
fn overview_only() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("summary");
    dir.create_file_fill(
        &PathBuf::from("top_file"),
        b"some text\nmore text\nother line",
    );

    let sub = PathBuf::from("sub");
    dir.add_child(&sub);
    dir.create_file_fill(&sub.join("sub_file"), b"some text here too");

    let result = get_output(&dir.path, "text --overview-only");
    assert_pass(&tar_dir.join("summary_dir"), result);

    let result = get_output(&dir.path.join("top_file"), "text --overview-only");
    assert_pass(&tar_dir.join("summary_file"), result);
}

#[test]
fn context() {
    let tar_dir: PathBuf = target_dir();
    let dir = Dir::new("context");
    let file = PathBuf::from("ctx_file");
    dir.create_file_fill(
        &file,
        b"line one\nline two\nline three match\nline four\nline five\nline six match\nline seven\nline eight",
    );

    let result = get_output(&dir.path.join(&file), "match --line-number --context 1");
    assert_pass(&tar_dir.join("context_c1"), result);

    let result = get_output(
        &dir.path.join(&file),
        "match --line-number --before-context 1",
    );
    assert_pass(&tar_dir.join("context_b1"), result);

    let result = get_output(
        &dir.path.join(&file),
        "match --line-number --after-context 1",
    );
    assert_pass(&tar_dir.join("context_a1"), result);
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

    let mut orig_result = get_output(
        &dir.path,
        &format!(
            "some --line-number --count --glob=!4ourth --repeat-file={} --overview --trim",
            repeat_file
        ),
    );
    let mut rep_result = get_output(
        &dir.path,
        &format!("--repeat --repeat-file={}", repeat_file),
    );
    assert!(orig_result == rep_result);

    (orig_result) = get_output(
        &dir.path,
        &format!(
            "in --files --glob=!4ourth_nice_file --repeat-file={} --overview",
            repeat_file
        ),
    );
    (rep_result) = get_output(
        &dir.path,
        &format!("--repeat --repeat-file={}", repeat_file),
    );
    assert!(orig_result == rep_result);
}

#[cfg(unix)]
#[test]
fn menu_navigate() {
    let tar_dir = target_dir();
    let dir = Dir::new("menu_navigate");

    let sub = PathBuf::from("sub_dir");
    dir.add_child(&sub);
    dir.create_file_fill(&sub.join("inner"), b"hit here\nhit again\nthird hit");
    dir.create_file_fill(&PathBuf::from("sibling"), b"hit also\nanother hit");

    let result = tmux::get_menu_output(
        &dir.path,
        "hit",
        &["}", "d", "j", "{", "Tab", "g", "j", "z"],
    );
    assert_pass_pool(
        &[
            &tar_dir.join("menu_navigate_1"),
            &tar_dir.join("menu_navigate_2"),
        ],
        result,
    );
}

#[cfg(unix)]
#[test]
fn menu_jump() {
    let tar_dir = target_dir();
    let dir = Dir::new("menu_jump");

    let dir_a = PathBuf::from("dir_a");
    dir.add_child(&dir_a);
    dir.create_file_fill(&dir_a.join("file_a"), b"word found\nword again\nword three");

    let dir_b = PathBuf::from("dir_b");
    dir.add_child(&dir_b);
    dir.create_file_fill(&dir_b.join("file_b"), b"word here");

    let result = tmux::get_menu_output(&dir.path, "word", &["G", "K", "}", "u", "Tab", "b"]);
    assert_pass_pool(
        &[&tar_dir.join("menu_jump_1"), &tar_dir.join("menu_jump_2")],
        result,
    );
}

#[cfg(unix)]
#[test]
fn menu_filter() {
    let tar_dir = target_dir();
    let dir = Dir::new("menu_filter");

    let sub = PathBuf::from("only_match");
    dir.add_child(&sub);
    dir.create_file_fill(&sub.join("content"), b"term one\nterm two\nterm three");
    dir.create_file_fill(&PathBuf::from("no_match_file"), b"term here");

    let result = tmux::get_menu_output(
        &dir.path,
        "term",
        &[
            "s", "o", "n", "l", "y", "_", "m", "a", "t", "c", "h", "Enter", "j", "j", "Tab", "Tab",
            "k", "G",
        ],
    );
    assert_pass(&tar_dir.join("menu_filter"), result);
}

#[cfg(unix)]
#[test]
fn menu_custom_keys() {
    let tar_dir = target_dir();
    let dir = Dir::new("menu_custom_keys");

    let sub = PathBuf::from("sub_dir");
    dir.add_child(&sub);
    dir.create_file_fill(&sub.join("inner"), b"search_me\nanother_match");

    let result = tmux::get_menu_output(
        &dir.path,
        "search_me --key-down=c --key-fold=x --key-filter=y",
        &["c", "c", "y", "i", "n", "n", "e", "r", "Enter", "x"],
    );
    assert_pass(&tar_dir.join("menu_custom_keys"), result);
}

#[cfg(unix)]
#[test]
fn menu_search_mode() {
    let tar_dir = target_dir();
    let dir = Dir::new("menu_search_mode");

    let sub = PathBuf::from("sub_dir");
    dir.add_child(&sub);
    dir.create_file_fill(&sub.join("inner"), b"hit here\nhit again\nthird hit");
    dir.create_file_fill(&PathBuf::from("sibling"), b"hit also\nanother hit");

    let result = tmux::get_menu_output_from_search(&dir.path, &["h", "i", "t", "Enter"]);
    assert_pass(&tar_dir.join("menu_search_mode"), result);
}
