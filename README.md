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
├──output_processor.rs: 2
│  ├──29: impl File {
│  ╰──101: impl AsUsize for Value {
├──writer.rs: 8
│  ├──21: impl Clone for PrefixComponent {
│  ├──32: pub struct OpenInfo<'a> {
│  ├──41: struct FileEntry<'a> {
│  ├──66: struct DirEntry<'a> {
│  ├──142: struct LineEntry<'a> {
│  ├──234: struct LongBranchEntry<'a> {
│  ├──278: impl Directory {
│  ╰──354: impl File {
├──menu.rs: 5
│  ├──19: struct PathInfo {
│  ├──26: impl PathInfo {
│  ├──83: pub struct Menu<'a> {
│  ├──100: struct Window {
│  ╰──106: impl Window {
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
├──errors.rs: 4
│  ├──8: pub struct Message {
│  ├──22: impl Error for Message {}
│  ├──24: impl fmt::Debug for Message {
│  ╰──30: impl fmt::Display for Message {
├──options.rs: 2
│  ├──42: pub struct Rg;
│  ╰──44: impl Options for Rg {
├──args.rs: 2
│  ├──17: pub struct ArgInfo {
│  ╰──23: impl ArgInfo {
├──term.rs: 1
│  ╰──13: pub struct Term<'a> {
├──matcher.rs: 1
│  ╰──121: impl File {
├──searchers.rs: 4
│  ├──13: struct ShortName(String);
│  ├──15: impl ShortName {
│  ├──22: impl Deref for ShortName {
│  ╰──85: impl Searchers {
├──formats.rs: 2
│  ├──19: pub struct Chars {
│  ╰──99: pub struct DisplayRepeater<T>(T, usize);
╰──config.rs: 3
   ├──12: pub struct Characters {
   ├──26: pub struct Config {
   ╰──79: impl Config {
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

### *--help*
```
treegrep 0.1.4

by Timothy Cronin

home page: https://github.com/4imothy/treegrep

pattern matcher that displays results in a tree structure with an interface to jump to matched text

tgrep [OPTIONS] [positional regexp] [positional target]
Arguments:
  [positional regexp]  the regex expression
  [positional target]  the path to search, if not provided, search the current directory

Options:
  -e, --regexp <>            the regex expression
  -p, --path <>              the path to search, if not provided, search the current directory
      --completions <shell>  generate completions for given shell [possible values: bash, elvish, fish, powershell, zsh]
      --glob <>              rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic
  -s, --searcher <>          executable to do the searching [possible values: rg, tgrep]
      --threads <>           set the appropriate number of threads to use
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
      --max-depth <>         the max depth to search
      --prefix-len <>        number of characters to show before a match [default: 3]
      --max-length <>        set the max length for a matched line
      --long-branch-each <>  number of files to print on each branch [default: 5]
      --char-style <>        style of characters to use [possible values: ascii, single, double, heavy, rounded, none]
      --long-branch          multiple files from the same directory are shown on the same branch
  -m, --menu                 open results in a menu to be edited with $EDITOR
                             navigate through the menu using the following commands:
                              - move up/down: k/j, p/n, up arrow/down arrow
                              - move up/down with a bigger jump: K/J, P/N
                              - move up/down paths: {/}, [/]
                              - move to the start/end: g/G, </>, home/end
                              - move up/down a page: b/f, pageup/pagedown
                              - center cursor: z/l
                              - quit: q, ctrl + c
  -h, --help                 Print help
  -V, --version              Print version

any of the above can be set using the TREEGREP_DEFAULT_OPTS environment variable
```
