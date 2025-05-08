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
├──options.rs: 2
│  ├──42: pub struct Rg;
│  ╰──44: impl Options for Rg {
├──menu.rs: 5
│  ├──19: struct PathInfo {
│  ├──26: impl PathInfo {
│  ├──83: pub struct Menu<'a> {
│  ├──100: struct Window {
│  ╰──106: impl Window {
├──formats.rs: 2
│  ├──19: pub struct Chars {
│  ╰──99: pub struct DisplayRepeater<T>(T, usize);
├──args.rs: 2
│  ├──17: pub struct ArgInfo {
│  ╰──23: impl ArgInfo {
├──matcher.rs: 1
│  ╰──121: impl File {
├──config.rs: 3
│  ├──12: pub struct Characters {
│  ├──26: pub struct Config {
│  ╰──79: impl Config {
├──output_processor.rs: 2
│  ├──29: impl File {
│  ╰──101: impl AsUsize for Value {
├──errors.rs: 4
│  ├──8: pub struct Message {
│  ├──22: impl Error for Message {}
│  ├──24: impl fmt::Debug for Message {
│  ╰──30: impl fmt::Display for Message {
├──writer.rs: 8
│  ├──21: impl Clone for PrefixComponent {
│  ├──32: pub struct OpenInfo<'a> {
│  ├──41: struct FileEntry<'a> {
│  ├──66: struct DirEntry<'a> {
│  ├──142: struct LineEntry<'a> {
│  ├──234: struct LongBranchEntry<'a> {
│  ├──278: impl Directory {
│  ╰──354: impl File {
├──match_system.rs: 10
│  ├──39: pub struct Directory {
│  ├──48: impl Directory {
│  ├──61: pub struct File {
│  ├──68: impl File {
│  ├──107: pub struct Match {
│  ├──113: impl Match {
│  ├──138: pub struct Line {
│  ├──144: impl Line {
│  ├──160:     impl PartialEq for Match {
│  ╰──167:     impl Debug for Match {
├──term.rs: 1
│  ╰──13: pub struct Term<'a> {
╰──searchers.rs: 4
   ├──13: struct ShortName(String);
   ├──15: impl ShortName {
   ├──22: impl Deref for ShortName {
   ╰──85: impl Searchers {
```
</details>

<details>
<summary><code>tgrep Print src/menu.rs --trim --line-number --char-style=ascii</code></summary>

```
menu.rs
+--9: style::{Print, SetBackgroundColor},
+--330: queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
+--360: Print(self.lines.get(id).unwrap())
+--389: Print(
+--506: Print(config().c.selected_indicator),
+--508: Print(self.lines.get(self.selected_id).unwrap())
+--516: Print(formats::SELECTED_INDICATOR_CLEAR),
+--518: Print(self.lines.get(self.selected_id).unwrap())
+--532: Print(format!(
+--544: Print(format!(
+--557: Print(format!(
```
</details>

<details>
<summary><code>tgrep --files --hidden</code></summary>

```
treegrep
├──completions
│  ├──tgrep.bash
│  ├──tgrep.fish
│  ├──tgrep.elv
│  ├──_tgrep
│  ╰──_tgrep.ps1
├──tests
│  ├──targets
│  │  ├──glob_inclusion
│  │  ├──links_12
│  │  ├──max_depth
│  │  ├──file
│  │  ├──links_22
│  │  ├──colon
│  │  ├──wide_21
│  │  ├──deep
│  │  ├──line_number
│  │  ├──links_11
│  │  ├──links_21
│  │  ├──glob_exclusion
│  │  ╰──wide_12
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──file_system.rs
│  ├──tests.rs
│  ╰──utils.rs
├──benchmarks
│  ├──times
│  ╰──runner
├──.github
│  ╰──workflows
│     ├──cr.yml
│     ├──update_readme_and_completions.yml
│     ├──test.yml
│     ╰──update_readme
├──src
│  ├──searchers.rs
│  ├──term.rs
│  ├──matcher.rs
│  ├──config.rs
│  ├──output_processor.rs
│  ├──log.rs
│  ├──options.rs
│  ├──errors.rs
│  ├──main.rs
│  ├──formats.rs
│  ├──writer.rs
│  ├──args.rs
│  ├──menu.rs
│  ╰──match_system.rs
├──Cargo.toml
├──LICENSE
├──Cargo.lock
├──build.rs
├──README.md
├──todos.md
╰──.gitignore
```
</details>

<details>
<summary><code>tgrep --files --long-branch --hidden</code></summary>

```
treegrep
├──completions
│  ╰──tgrep.bash, tgrep.fish, tgrep.elv, _tgrep, _tgrep.ps1
├──tests
│  ├──targets
│  │  ├──glob_inclusion, links_12, max_depth, file, links_22
│  │  ├──colon, wide_21, deep, line_number, links_11
│  │  ╰──links_21, glob_exclusion, wide_12
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ╰──file_system.rs, tests.rs, utils.rs
├──benchmarks
│  ╰──times, runner
├──.github
│  ╰──workflows
│     ╰──cr.yml, update_readme_and_completions.yml, test.yml, update_readme
├──src
│  ├──searchers.rs, term.rs, matcher.rs, config.rs, output_processor.rs
│  ├──log.rs, options.rs, errors.rs, main.rs, formats.rs
│  ╰──writer.rs, args.rs, menu.rs, match_system.rs
├──Cargo.toml, LICENSE, Cargo.lock, build.rs, README.md
╰──todos.md, .gitignore
```
</details>

