## treegrep

treegrep is a regex pattern matcher that displays results in a tree structure with an interface to jump to matched text.

[![test](https://github.com/4imothy/treegrep/actions/workflows/test.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)
[![release](https://github.com/4imothy/treegrep/actions/workflows/cr.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)

[introduction video](https://youtu.be/lRMwCE6Zwuw?si=m9SRypN6_NxgW6K4)

[examples](#examples), [editor integrations](#editor-integrations), and [help](#--help).

### links
[crates.io](https://crates.io/crates/treegrep) | [GitHub](https://github.com/4imothy/treegrep) | [AUR](https://aur.archlinux.org/packages/treegrep-bin) | [NetBSD](https://pkgsrc.se/sysutils/treegrep)

### installation
- **cargo:** ```cargo install treegrep```
- **releases:** Download from [releases](https://github.com/4imothy/treegrep/releases/)
- **manual:**
  ```
  git clone https://github.com/4imothy/treegrep
  cd treegrep
  cargo build --release
  ```

### editor integrations
<details>
<summary><em>neovim</em></summary>

- sample installation using [lazy.nvim](https://github.com/folke/lazy.nvim)
```lua
return {
    '4imothy/treegrep',
    build = function()
        require('treegrep').build_tgrep()
    end,
    config = function()
        require('treegrep').setup({
            selection_file = '/tmp/tgrep-select',
            repeat_file = '/tmp/tgrep-repeat',
        })
        vim.keymap.set('n', '<leader>tt', function() require('treegrep').tgrep_with('--menu') end)
        vim.keymap.set('n', '<leader>tr', function() require('treegrep').tgrep_with('--repeat') end)
        vim.keymap.set('n', '<leader>tm', function() require('treegrep').tgrep_with('--menu --repeat') end)
        vim.keymap.set('n', '<leader>tf', function() require('treegrep').tgrep_with('--files --select') end)
    end,
}
```
</details>
<details>
<summary><em>helix</em></summary>

- sample keybind to run treegrep and open selection
```toml
C-t = [
    ':sh rm -f /tmp/tgrep-select',
    ':insert-output tgrep --menu --selection-file=/tmp/tgrep-select --repeat-file=/tmp/tgrep-repeat > /dev/tty',
    ':open %sh{ f=$(sed -n 1p /tmp/tgrep-select); l=$(sed -n 2p /tmp/tgrep-select); [ -n "$l" ] && echo "$f:$l" || echo "$f"; }',
    ':redraw',
    ':set mouse false',
    ':set mouse true',
]
```
</details>
<details>
<summary><em>vim</em></summary>

- sample installation using [vim-plug](https://github.com/junegunn/vim-plug)
```vim
Plug '4imothy/treegrep', {'do': {-> TgrepBuild()}}

let g:tgrep_selection_file = '/tmp/tgrep-select'
let g:tgrep_repeat_file = '/tmp/tgrep-repeat'

nnoremap <leader>tt :call TgrepWith('--menu')<cr>
nnoremap <leader>tr :call TgrepWith('--repeat')<cr>
nnoremap <leader>tm :call TgrepWith('--menu --repeat')<cr>
nnoremap <leader>tf :call TgrepWith('--files --select')<cr>
```
</details>

### examples
<details>
<summary><code>tgrep --regexp \bstruct\s+\w+ --regexp \bimpl\s+\w+ --path src --line-number --context=1 --count</code></summary>

```
src: 10
├──term.rs: 1
│  ├──-1: 
│  ├──15: pub struct Term<'a> {
│  ╰──+1:     pub height: u16,
├──matcher.rs: 3
│  ├──-1: 
│  ├──20: struct Matcher {
│  ├──+1:     combined: RegexMatcher,
│  ├──-1: 
│  ├──25: impl Matcher {
│  ├──+1:     fn new(patterns: &[String]) -> Result<Self, Message> {
│  ├──-1: 
│  ├──44: struct MatchSink<'a> {
│  ╰──+1:     lines: Vec<Line>,
├──args_menu.rs: 1
│  ├──-1: 
│  ├──21: pub struct ArgsMenu<'a, 'b> {
│  ╰──+1:     term: &'a mut term::Term<'b>,
├──errors.rs: 4
│  ├──-1: 
│  ├──14: pub struct Message {
│  ├──+1:     pub mes: String,
│  ├──-1: }
│  ├──17: impl Error for Message {}
│  ├──+1: 
│  ├──-1: 
│  ├──34: impl fmt::Debug for Message {
│  ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
│  ├──-1: 
│  ├──40: impl fmt::Display for Message {
│  ╰──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
├──style.rs: 1
│  ├──-1: 
│  ├──31: pub struct DisplayRepeater<T>(T, usize);
│  ╰──+1: impl<T: Display> Display for DisplayRepeater<T> {
├──match_system.rs: 8
│  ├──-1: 
│  ├──23: pub struct Directory {
│  ├──+1:     pub path: PathBuf,
│  ├──-1: 
│  ├──30: impl Directory {
│  ├──+1:     pub fn new(path: &Path) -> Result<Self, Message> {
│  ├──-1: 
│  ├──41: pub struct File {
│  ├──+1:     pub path: PathBuf,
│  ├──-1: 
│  ├──47: impl File {
│  ├──+1:     pub fn from_pathbuf(path: PathBuf) -> Result<Self, Message> {
│  ├──-1: #[cfg_attr(test, derive(PartialEq, Debug))]
│  ├──72: pub struct Match {
│  ├──+1:     pub regexp_id: usize,
│  ├──-1: 
│  ├──78: impl Match {
│  ├──+1:     pub fn new(regexp_id: usize, start: usize, end: usize) -> Self {
│  ├──-1: 
│  ├──103: pub struct Line {
│  ├──+1:     pub content: String,
│  ├──-1: 
│  ├──110: impl Line {
│  ╰──+1:     pub fn new(content: String, mut matches: Vec<Match>, line_num: usize) -> Self {
├──select_menu.rs: 5
│  ├──-1: 
│  ├──81: impl OpenStrategy {
│  ├──+1:     fn from(editor: &str) -> Self {
│  ├──-1: 
│  ├──93: pub struct SelectMenu<'a, 'b> {
│  ├──+1:     jump: JumpLocation,
│  ├──-1: 
│  ├──110: struct Window {
│  ├──+1:     first: isize,
│  ├──-1: 
│  ├──115: impl Window {
│  ├──+1:     pub fn shift_up(&mut self) {
│  ├──-1: 
│  ├──136: impl JumpLocation {
│  ╰──+1:     fn default() -> Self {
├──config.rs: 6
│  ├──-1: 
│  ├──15: pub struct KeyBindings {
│  ├──+1:     pub down: Vec<KeyCode>,
│  ├──-1: 
│  ├──35: pub struct Characters {
│  ├──+1:     pub bl: char,
│  ├──-1: 
│  ├──51: pub struct Colors {
│  ├──+1:     pub file: Color,
│  ├──-1: 
│  ├──62: impl args::Color {
│  ├──+1:     fn get(&self) -> Color {
│  ├──-1: 
│  ├──80: pub struct Config {
│  ├──+1:     pub path: PathBuf,
│  ├──-1: 
│  ├──186: impl Config {
│  ╰──+1:     pub fn get_styling(matches: &ArgMatches) -> (bool, bool) {
├──args.rs: 7
│  ├──-1: 
│  ├──19: pub struct ArgInfo {
│  ├──+1:     pub id: &'static str,
│  ├──-1: 
│  ├──25: impl ArgInfo {
│  ├──+1:     const fn new(id: &'static str, h: &'static str, s: Option<char>) -> Self {
│  ├──-1: 
│  ├──40: impl ValueEnum for OpenStrategy {
│  ├──+1:     fn value_variants<'a>() -> &'a [Self] {
│  ├──-1: #[derive(Clone)]
│  ├──85: struct ColorParser;
│  ├──+1: 
│  ├──87: impl clap::builder::TypedValueParser for ColorParser {
│  ├──+1:     type Value = Color;
│  ├──-1: #[derive(Clone)]
│  ├──173: struct KeyCodeParser;
│  ├──+1: 
│  ├──175: impl clap::builder::TypedValueParser for KeyCodeParser {
│  ╰──+1:     type Value = KeyCode;
╰──writer.rs: 9
   ├──-1: 
   ├──26: pub struct OpenInfo<'a> {
   ├──+1:     pub path: &'a Path,
   ├──-1: 
   ├──37: struct PathDisplay<'a> {
   ├──+1:     prefix: Option<Vec<PrefixComponent>>,
   ├──-1: 
   ├──150: struct LineDisplay<'a> {
   ├──+1:     prefix: Vec<PrefixComponent>,
   ├──-1: 
   ├──279: struct LongBranchDisplay<'a> {
   ├──+1:     prefix: Vec<PrefixComponent>,
   ├──-1: 
   ├──320: struct OverviewDisplay {
   ├──+1:     dirs: usize,
   ├──-1: 
   ├──328: impl Entry for OverviewDisplay {
   ├──+1:     fn open_info(&self) -> Result<OpenInfo<'_>, Message> {
   ├──-1: 
   ├──340: impl Display for OverviewDisplay {
   ├──+1:     fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
   ├──-1: 
   ├──388: impl Directory {
   ├──+1:     fn to_lines<'a>(
   ├──-1: 
   ├──473: impl File {
   ╰──+1:     fn to_lines<'a>(
```
</details>

<details>
<summary><code>tgrep Print src/select_menu.rs --trim --line-number --char-vertical=| --char-horizontal=- --char-top-left=+ --char-top-right=+ --char-bottom-left=+ --char-bottom-right=+ --char-tee=+ --char-ellipsis=|</code></summary>

```
select_menu.rs
+--15: style::{Print, SetBackgroundColor},
+--214: queue!(self.term, Print(&self.lines[orig]))?;
+--216: queue!(self.term, Print(config().chars.ellipsis))?;
+--711: Print(style::style_with(config().chars.selected_indicator.as_str(), c)),
+--715: queue!(self.term, Print(config().chars.selected_indicator.as_str()),)?;
+--725: Print(config().chars.selected_indicator_clear.as_str()),
+--741: Print(format!(
+--753: Print(format!(
+--766: Print(format!(
```
</details>

<details>
<summary><code>tgrep --files --hidden --glob=!.git</code></summary>

```
treegrep
├──src
│  ├──match_system.rs
│  ├──args.rs
│  ├──writer.rs
│  ├──main.rs
│  ├──errors.rs
│  ├──select_menu.rs
│  ├──style.rs
│  ├──log.rs
│  ├──config.rs
│  ├──matcher.rs
│  ├──term.rs
│  ╰──args_menu.rs
├──doc
│  ├──treegrep.vim9.txt
│  ╰──treegrep.nvim.txt
├──.github
│  ╰──workflows
│     ├──update_readme
│     ├──test.yml
│     ├──update_readme.yml
│     ╰──cr.yml
├──benchmarks
│  ├──runner
│  ╰──times
├──lua
│  ╰──treegrep.lua
├──plugin
│  ╰──treegrep.vim
├──tests
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──files_1
│  │  ├──wide_2
│  │  ├──links_4
│  │  ├──links_3
│  │  ├──links_2
│  │  ├──files_long_branch_expr_2
│  │  ├──glob_exclusion
│  │  ├──no_matches
│  │  ├──files_long_branch_1
│  │  ├──context_b1
│  │  ├──context_a1
│  │  ├──files_long_branch_expr_count_2
│  │  ├──overview_dir
│  │  ├──wide_1
│  │  ├──files_2
│  │  ├──line_number
│  │  ├──deep
│  │  ├──context_c1
│  │  ├──links_1
│  │  ├──count
│  │  ├──colon
│  │  ├──overview_file
│  │  ├──files_long_branch_expr_1
│  │  ├──overlapping
│  │  ├──file
│  │  ├──max_depth
│  │  ├──files_long_branch_2
│  │  ├──glob_inclusion
│  │  ├──files_long_branch_expr_count_1
│  │  ╰──files_with_expr
│  ├──utils.rs
│  ├──tests.rs
│  ╰──file_system.rs
├──.gitignore
├──README.md
├──Cargo.lock
├──rustfmt.toml
├──Cargo.toml
╰──LICENSE
```
</details>

<details>
<summary><code>tgrep --files --long-branch --hidden --glob=!.git</code></summary>

```
treegrep
├──src
│  ├──match_system.rs, args.rs, writer.rs, main.rs, errors.rs
│  ├──select_menu.rs, style.rs, log.rs, config.rs, matcher.rs
│  ╰──term.rs, args_menu.rs
├──doc
│  ╰──treegrep.vim9.txt, treegrep.nvim.txt
├──.github
│  ╰──workflows
│     ╰──update_readme, test.yml, update_readme.yml, cr.yml
├──benchmarks
│  ╰──runner, times
├──lua
│  ╰──treegrep.lua
├──plugin
│  ╰──treegrep.vim
├──tests
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──max_depth, files_long_branch_2, glob_inclusion, files_long_branch_expr_count_1, overview_file
│  │  ├──files_with_expr, files_long_branch_expr_1, overlapping, file, links_1
│  │  ├──colon, count, context_c1, deep, line_number
│  │  ├──files_2, wide_1, overview_dir, files_long_branch_expr_count_2, files_long_branch_1
│  │  ├──context_b1, context_a1, glob_exclusion, no_matches, files_long_branch_expr_2
│  │  ╰──files_1, wide_2, links_4, links_3, links_2
│  ╰──utils.rs, tests.rs, file_system.rs
├──.gitignore, README.md, Cargo.lock, LICENSE, rustfmt.toml
╰──Cargo.toml
```
</details>

### *--help*
```
treegrep 1.3.0

by Timothy Cronin

home page: https://github.com/4imothy/treegrep

regex pattern matcher that displays results in a tree structure with an interface to jump to matched text

tgrep [OPTIONS] [positional regexp] [positional target]

arguments:
  [positional regexp]
          a regex expression to search for

  [positional target]
          the path to search, if not provided, search the current directory

options:
  -e, --regexp <>
          a regex expression to search for

  -p, --path <>
          the path to search, if not provided, search the current directory

  -s, --select
          results are shown in a selection interface for opening

  -f, --files
          if an expression is given, hide matched content, otherwise, show the files that would be searched

  -., --hidden
          search hidden files

  -n, --line-number
          show line number of match

  -c, --count
          display number of files matched in directory and number of lines matched in a file

  -g, --glob <>
          rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic

  -l, --links
          search linked paths

  -o, --overview
          conclude results with an overview

  -d, --max-depth <>
          the max depth to search

  -C, --context <>
          number of lines to show before and after each match

  -B, --before-context <>
          number of lines to show before each match

  -A, --after-context <>
          number of lines to show after each match

      --max-length <>
          set the max length for a matched line

      --menu
          provide arguments and select results through an interface

      --no-ignore
          don't use ignore files

      --trim
          trim whitespace at the beginning of lines

      --threads <>
          set the appropriate number of threads to use

      --long-branch
          multiple files from the same directory are shown on the same branch

      --editor <>
          command used to open selections

      --open-like <>
          command line syntax for opening a file at a line
          
          [possible values: vi, hx, code, jed, default]

      --completions <>
          generate completions for given shell
          
          [possible values: bash, elvish, fish, powershell, zsh]

      --selection-file <>
          file to write selection to (first line: file path, second line: line number if applicable)

      --repeat-file <>
          file where arguments are saved

      --repeat
          repeats the last saved search

      --no-color
          don't use colors

      --no-bold
          don't bold anything

      --file-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --dir-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --text-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --line-number-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --branch-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --match-colors <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --selected-indicator-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --selected-bg-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --prefix-len <>
          number of characters to show before a match
          
          [default: 3]

      --long-branch-each <>
          number of files to print on each branch
          
          [default: 5]

      --char-vertical <>
          vertical branch character

      --char-horizontal <>
          horizontal branch character

      --char-top-left <>
          top-left corner character

      --char-top-right <>
          top-right corner character

      --char-bottom-left <>
          bottom-left corner character

      --char-bottom-right <>
          bottom-right corner character

      --char-tee <>
          tee branch character

      --char-ellipsis <>
          folding indicator character

      --selected-indicator <>
          selected indicator characters
          
          [default: "─❱ "]

      --key-down <>
          move down
          
          [default: down j n]

      --key-up <>
          move up
          
          [default: up k p]

      --key-big-down <>
          big jump down
          
          [default: J N]

      --key-big-up <>
          big jump up
          
          [default: K P]

      --key-down-path <>
          next path
          
          [default: } ]]

      --key-up-path <>
          previous path
          
          [default: { []

      --key-down-same-depth <>
          next path at same depth
          
          [default: ) d]

      --key-up-same-depth <>
          previous path at same depth
          
          [default: ( u]

      --key-top <>
          go to top
          
          [default: home g <]

      --key-bottom <>
          go to bottom
          
          [default: end G >]

      --key-page-down <>
          page down
          
          [default: pagedown f]

      --key-page-up <>
          page up
          
          [default: pageup b]

      --key-center <>
          center cursor
          
          [default: z l]

      --key-help <>
          show help
          
          [default: h]

      --key-quit <>
          quit
          
          [default: q]

      --key-open <>
          open selection
          
          [default: enter]

      --key-fold <>
          fold/unfold path
          
          [default: tab]

  -h, --help
          print help

  -V, --version
          print version

arguments are prefixed with the contents of the TREEGREP_DEFAULT_OPTS environment variable
```
