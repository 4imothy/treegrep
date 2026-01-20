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
nnoremap <leader>tf :call TgrepWith('--files --select')<cr>
```
</details>

### examples
<details>
<summary><code>tgrep --regexp \bstruct\s+\w+ --regexp \bimpl\s+\w+ --path src --line-number --count</code></summary>

```
src: 10
├──term.rs: 1
│  └──15: pub struct Term<'a> {
├──matcher.rs: 3
│  ├──18: struct Matcher {
│  ├──23: impl Matcher {
│  └──42: struct MatchSink<'a> {
├──match_system.rs: 8
│  ├──23: pub struct Directory {
│  ├──30: impl Directory {
│  ├──41: pub struct File {
│  ├──47: impl File {
│  ├──73: pub struct Match {
│  ├──79: impl Match {
│  ├──104: pub struct Line {
│  └──110: impl Line {
├──style.rs: 2
│  ├──22: pub struct Chars {
│  └──99: pub struct DisplayRepeater<T>(T, usize);
├──config.rs: 5
│  ├──15: pub struct Characters {
│  ├──29: pub struct Colors {
│  ├──40: impl args::Color {
│  ├──58: pub struct Config {
│  └──169: impl Config {
├──errors.rs: 4
│  ├──14: pub struct Message {
│  ├──17: impl Error for Message {}
│  ├──34: impl fmt::Debug for Message {
│  └──40: impl fmt::Display for Message {
├──select_menu.rs: 5
│  ├──33: impl OpenStrategy {
│  ├──45: pub struct SelectMenu<'a, 'b> {
│  ├──60: struct Window {
│  ├──65: impl Window {
│  └──86: impl JumpLocation {
├──args_menu.rs: 1
│  └──21: pub struct ArgsMenu<'a, 'b> {
├──args.rs: 6
│  ├──18: pub struct ArgInfo {
│  ├──24: impl ArgInfo {
│  ├──39: impl ValueEnum for OpenStrategy {
│  ├──88: impl ValueEnum for CharacterStyle {
│  ├──120: struct ColorParser;
│  └──122: impl clap::builder::TypedValueParser for ColorParser {
└──writer.rs: 9
   ├──26: pub struct OpenInfo<'a> {
   ├──37: struct PathDisplay<'a> {
   ├──150: struct LineDisplay<'a> {
   ├──262: struct LongBranchDisplay<'a> {
   ├──303: struct OverviewDisplay {
   ├──311: impl Entry for OverviewDisplay {
   ├──323: impl Display for OverviewDisplay {
   ├──371: impl Directory {
   └──456: impl File {
```
</details>

<details>
<summary><code>tgrep Print src/select_menu.rs --trim --line-number --char-style=ascii</code></summary>

```
select_menu.rs
+--8: style::{Print, SetBackgroundColor},
+--344: queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
+--365: queue!(self.term, scroll, cursor::MoveTo(START_X, y), Print(line))?;
+--590: Print(style::style_with(config().chars.selected_indicator, c)),
+--594: queue!(self.term, Print(config().chars.selected_indicator),)?;
+--599: Print(&self.lines[self.selected_id])
+--607: Print(style::SELECTED_INDICATOR_CLEAR),
+--609: Print(&self.lines[self.selected_id])
+--623: Print(format!(
+--635: Print(format!(
+--648: Print(format!(
```
</details>

<details>
<summary><code>tgrep --files --hidden --glob=!.git</code></summary>

```
treegrep
├──doc
│  ├──treegrep.nvim.txt
│  └──treegrep.vim9.txt
├──src
│  ├──match_system.rs
│  ├──style.rs
│  ├──matcher.rs
│  ├──config.rs
│  ├──writer.rs
│  ├──main.rs
│  ├──term.rs
│  ├──select_menu.rs
│  ├──args_menu.rs
│  ├──args.rs
│  ├──log.rs
│  └──errors.rs
├──.github
│  └──workflows
│     ├──update_readme.yml
│     ├──update_readme
│     ├──cr.yml
│     └──test.yml
├──plugin
│  └──treegrep.vim
├──benchmarks
│  ├──runner
│  └──times
├──lua
│  └──treegrep.lua
├──tests
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──wide_1
│  │  ├──links_3
│  │  ├──files_long_branch_2
│  │  ├──glob_exclusion
│  │  ├──wide_2
│  │  ├──colon
│  │  ├──overlapping
│  │  ├──files_long_branch_expr_count_2
│  │  ├──max_depth
│  │  ├──files_1
│  │  ├──files_long_branch_expr_2
│  │  ├──files_long_branch_1
│  │  ├──glob_inclusion
│  │  ├──files_with_expr
│  │  ├──deep
│  │  ├──files_long_branch_expr_count_1
│  │  ├──links_4
│  │  ├──count
│  │  ├──links_2
│  │  ├──links_1
│  │  ├──files_2
│  │  ├──no_matches
│  │  ├──file
│  │  ├──line_number
│  │  └──files_long_branch_expr_1
│  ├──utils.rs
│  ├──tests.rs
│  └──file_system.rs
├──Cargo.toml
├──README.md
├──.gitignore
├──Cargo.lock
├──rustfmt.toml
├──todos.md
└──LICENSE
```
</details>

<details>
<summary><code>tgrep --files --long-branch --hidden --glob=!.git</code></summary>

```
treegrep
├──src
│  ├──match_system.rs, style.rs, matcher.rs, config.rs, writer.rs
│  ├──main.rs, term.rs, select_menu.rs, args_menu.rs, args.rs
│  └──log.rs, errors.rs
├──plugin
│  └──treegrep.vim
├──tests
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──links_3, files_long_branch_2, glob_exclusion, wide_2, colon
│  │  ├──overlapping, files_long_branch_expr_count_2, max_depth, files_1, files_long_branch_expr_2
│  │  ├──files_long_branch_1, glob_inclusion, files_with_expr, deep, files_long_branch_expr_count_1
│  │  ├──links_4, count, links_2, links_1, files_2
│  │  └──no_matches, file, line_number, files_long_branch_expr_1, wide_1
│  └──file_system.rs, utils.rs, tests.rs
├──lua
│  └──treegrep.lua
├──.github
│  └──workflows
│     └──update_readme.yml, update_readme, cr.yml, test.yml
├──benchmarks
│  └──runner, times
├──doc
│  └──treegrep.nvim.txt, treegrep.vim9.txt
├──Cargo.toml, rustfmt.toml, todos.md, README.md, .gitignore
└──Cargo.lock, LICENSE
```
</details>

### *--help*
```
treegrep 1.3.0

by Timothy Cronin

home page: https://github.com/4imothy/treegrep

regex pattern matcher that displays results in a tree structure with an interface to jump to matched text

tgrep [OPTIONS] [positional regexp] [positional target]

Arguments:
  [positional regexp]  a regex expression to search for
  [positional target]  the path to search, if not provided, search the current directory

Options:
  -e, --regexp <>                    a regex expression to search for
  -p, --path <>                      the path to search, if not provided, search the current directory
      --menu                         provide arguments and select results through an interface
  -s, --select                       results are shown in a selection interface for opening
      --glob <>                      rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic
  -f, --files                        if an expression is given, hide matched content, otherwise, show the files that would be searched
  -., --hidden                       search hidden files
  -n, --line-number                  show line number of match
      --links                        search linked paths
      --no-ignore                    don't use ignore files
  -c, --count                        display number of files matched in directory and number of lines matched in a file
      --no-color                     don't use colors
      --no-bold                      don't bold anything
      --overview                     conclude results with an overview
      --max-depth <>                 the max depth to search
      --prefix-len <>                number of characters to show before a match [default: 3]
      --max-length <>                set the max length for a matched line
      --trim                         trim whitespace at the beginning of lines
      --threads <>                   set the appropriate number of threads to use
      --long-branch                  multiple files from the same directory are shown on the same branch
      --long-branch-each <>          number of files to print on each branch [default: 5]
      --editor <>                    command used to open selections
      --open-like <>                 command line syntax for opening a file at a line [possible values: vi, hx, code, jed, default]
      --char-style <>                style of characters to use [possible values: single, rounded, heavy, double, ascii, none]
      --file-color <>                black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)
      --dir-color <>                 black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)
      --text-color <>                black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)
      --line-number-color <>         black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)
      --branch-color <>              black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)
      --match-colors <>              black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)
      --selected-indicator-color <>  black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)
      --selected-bg-color <>         black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)
      --completions <>               generate completions for given shell [possible values: bash, elvish, fish, powershell, zsh]
      --selection-file <>            file to write selection to (first line: file path, second line: line number if applicable)
      --repeat-file <>               file where arguments are saved
      --repeat                       repeats the last saved search
  -h, --help                         print help
  -V, --version                      print version

arguments are prefixed with the contents of the TREEGREP_DEFAULT_OPTS environment variable
```
