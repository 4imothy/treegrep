## treegrep

treegrep is a pattern matcher that displays results in a tree structure with an interface to jump to matched text.

[![test](https://github.com/4imothy/treegrep/actions/workflows/test.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)
[![release](https://github.com/4imothy/treegrep/actions/workflows/cr.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)

[examples](#examples) and [help](#--help).

### supported backends
- *[ripgrep](https://github.com/BurntSushi/ripgrep)*
- *[treegrep](https://github.com/4imothy/treegrep)*

### links
[crates.io](https://crates.io/crates/treegrep) | [GitHub](https://github.com/4imothy/treegrep) | [AUR](https://aur.archlinux.org/packages/treegrep-bin) | [NetBSD](https://pkgsrc.se/sysutils/treegrep)

### installation
- **Cargo:** ```cargo install treegrep```
- **Releases:** Download from [releases](https://github.com/4imothy/treegrep/releases/)
- **Manual:**
  ```
  git clone https://github.com/4imothy/treegrep
  cd treegrep
  cargo build --release
  ```

### examples
<details>
<summary><code>tgrep --regexp \bstruct\s+\w+ --regexp \bimpl\s+\w+ --path src --line-number --count</code></summary>

```
src: 12
├──matcher.rs: 1
│  └──115: impl File {
├──match_system.rs: 10
│  ├──24: pub struct Directory {
│  ├──32: impl Directory {
│  ├──44: pub struct File {
│  ├──50: impl File {
│  ├──74: pub struct Match {
│  ├──80: impl Match {
│  ├──105: pub struct Line {
│  ├──111: impl Line {
│  ├──127:     impl PartialEq for Match {
│  └──134:     impl Debug for Match {
├──formats.rs: 2
│  ├──19: pub struct Chars {
│  └──99: pub struct DisplayRepeater<T>(T, usize);
├──options.rs: 2
│  ├──42: pub struct Rg;
│  └──44: impl Options for Rg {
├──errors.rs: 4
│  ├──8: pub struct Message {
│  ├──22: impl Error for Message {}
│  ├──24: impl fmt::Debug for Message {
│  └──30: impl fmt::Display for Message {
├──searchers.rs: 5
│  ├──12: struct ShortName(String);
│  ├──14: impl ShortName {
│  ├──21: impl std::fmt::Display for ShortName {
│  ├──27: impl Deref for ShortName {
│  └──89: impl Searchers {
├──config.rs: 3
│  ├──11: pub struct Characters {
│  ├──25: pub struct Config {
│  └──79: impl Config {
├──term.rs: 1
│  └──13: pub struct Term<'a> {
├──output_processor.rs: 2
│  ├──29: impl File {
│  └──100: impl AsUsize for Value {
├──menu.rs: 5
│  ├──19: struct PathInfo {
│  ├──26: impl PathInfo {
│  ├──83: pub struct Menu<'a> {
│  ├──100: struct Window {
│  └──106: impl Window {
├──writer.rs: 10
│  ├──22: impl Clone for PrefixComponent {
│  ├──33: pub struct OpenInfo<'a> {
│  ├──42: struct PathDisplay<'a> {
│  ├──143: struct LineDisplay<'a> {
│  ├──235: struct LongBranchDisplay<'a> {
│  ├──267: struct OverviewDisplay {
│  ├──275: impl Entry for OverviewDisplay {
│  ├──281: impl Display for OverviewDisplay {
│  ├──328: impl Directory {
│  └──431: impl File {
└──args.rs: 2
   ├──16: pub struct ArgInfo {
   └──22: impl ArgInfo {
```
</details>

<details>
<summary><code>tgrep Print src/menu.rs --trim --line-number --char-style=ascii</code></summary>

```
menu.rs
+--9: style::{Print, SetBackgroundColor},
+--325: queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
+--355: Print(self.lines.get(id).unwrap())
+--384: Print(
+--501: Print(config().c.selected_indicator),
+--503: Print(self.lines.get(self.selected_id).unwrap())
+--511: Print(formats::SELECTED_INDICATOR_CLEAR),
+--513: Print(self.lines.get(self.selected_id).unwrap())
+--527: Print(format!(
+--539: Print(format!(
+--552: Print(format!(
```
</details>

<details>
<summary><code>tgrep --files --hidden --glob=!.git</code></summary>

```
treegrep
├──.github
│  └──workflows
│     ├──update_readme
│     ├──test.yml
│     ├──update_readme_and_completions.yml
│     └──cr.yml
├──completions
│  ├──tgrep.elv
│  ├──tgrep.bash
│  ├──_tgrep
│  ├──_tgrep.ps1
│  └──tgrep.fish
├──src
│  ├──matcher.rs
│  ├──match_system.rs
│  ├──main.rs
│  ├──formats.rs
│  ├──options.rs
│  ├──errors.rs
│  ├──searchers.rs
│  ├──config.rs
│  ├──term.rs
│  ├──output_processor.rs
│  ├──log.rs
│  ├──menu.rs
│  ├──writer.rs
│  └──args.rs
├──tests
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──line_number
│  │  ├──file
│  │  ├──no_matches
│  │  ├──colon
│  │  ├──wide_2
│  │  ├──links_1
│  │  ├──files_long_branch_1
│  │  ├──max_depth
│  │  ├──files_long_branch_expr_2
│  │  ├──links_4
│  │  ├──files_long_branch_expr_1
│  │  ├──links_2
│  │  ├──files_1
│  │  ├──glob_inclusion
│  │  ├──overlapping_tgrep
│  │  ├──wide_1
│  │  ├──overlapping_rg
│  │  ├──files_long_branch_expr_count_2
│  │  ├──files_2
│  │  ├──links_3
│  │  ├──files_with_expr
│  │  ├──files_long_branch_2
│  │  ├──deep
│  │  ├──count
│  │  ├──glob_exclusion
│  │  └──files_long_branch_expr_count_1
│  ├──tests.rs
│  ├──utils.rs
│  └──file_system.rs
├──benchmarks
│  ├──times
│  └──runner
├──Cargo.toml
├──Cargo.lock
├──.gitignore
├──build.rs
├──README.md
├──todos.md
└──LICENSE
```
</details>

<details>
<summary><code>tgrep --files --long-branch --hidden --glob=!.git</code></summary>

```
treegrep
├──.github
│  └──workflows
│     └──update_readme, test.yml, update_readme_and_completions.yml, cr.yml
├──completions
│  └──tgrep.elv, tgrep.bash, _tgrep, _tgrep.ps1, tgrep.fish
├──src
│  ├──matcher.rs, match_system.rs, main.rs, formats.rs, options.rs
│  ├──errors.rs, searchers.rs, config.rs, term.rs, output_processor.rs
│  └──log.rs, menu.rs, writer.rs, args.rs
├──tests
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──line_number, file, no_matches, colon, wide_2
│  │  ├──links_1, files_long_branch_1, max_depth, files_long_branch_expr_2, links_4
│  │  ├──files_long_branch_expr_1, links_2, files_1, glob_inclusion, overlapping_tgrep
│  │  ├──wide_1, overlapping_rg, files_long_branch_expr_count_2, files_2, links_3
│  │  ├──files_with_expr, files_long_branch_2, deep, count, glob_exclusion
│  │  └──files_long_branch_expr_count_1
│  └──tests.rs, utils.rs, file_system.rs
├──benchmarks
│  └──times, runner
├──Cargo.toml, Cargo.lock, .gitignore, build.rs, README.md
└──todos.md, LICENSE
```
</details>

### *--help*
```
treegrep 0.1.4

by Timothy Cronin

home page: https://github.com/4imothy/treegrep

pattern matcher that displays results in a tree structure with an interface to jump to matched text

tgrep [OPTIONS] [positional regexp] [positional target]
       tgrep [OPTIONS] [positional regexp] [positional target] <COMMAND>
Commands:
  completions  generate completions for given shell

Arguments:
  [positional regexp]  the regex expression
  [positional target]  the path to search, if not provided, search the current directory

Options:
  -e, --regexp <>            the regex expression
  -p, --path <>              the path to search, if not provided, search the current directory
      --glob <>              rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic
  -s, --searcher <>          executable to do the searching [possible values: rg, tgrep]
      --char-style <>        style of characters to use [possible values: ascii, single, double, heavy, rounded, none]
      --editor <>            command used to open selections
      --open-like <>         command line syntax for opening a file at a line [possible values: vi, hx, code, jed, default]
      --long-branch          multiple files from the same directory are shown on the same branch
  -., --hidden               search hidden files
  -n, --line-number          show line number of match
  -f, --files                if a pattern is given hide matched content, otherwise show the files that would be searched
      --links                search linked paths
      --trim                 trim whitespace at the beginning of lines
      --pcre2                enable PCRE2
      --no-ignore            don't use ignore files
  -c, --count                display number of files matched in directory and number of lines matched in a file
      --no-color             don't use colors
      --no-bold              don't bold anything
      --overview             conclude results with an overview
  -m, --menu                 show results in a menu to be jumped to, press h in menu for help
      --threads <>           set the appropriate number of threads to use
      --max-depth <>         the max depth to search
      --prefix-len <>        number of characters to show before a match [default: 3]
      --max-length <>        set the max length for a matched line
      --long-branch-each <>  number of files to print on each branch [default: 5]
  -h, --help                 Print help
  -V, --version              Print version

any of the above can be set using the TREEGREP_DEFAULT_OPTS environment variable
```
