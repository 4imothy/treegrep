## treegrep

treegrep is a pattern matcher that displays results in a tree structure with an interface to jump to matched text.

[![test](https://github.com/4imothy/treegrep/actions/workflows/test.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)
[![release](https://github.com/4imothy/treegrep/actions/workflows/cr.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)

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
src: 13
├──searchers.rs: 1
│  └──34: impl Searchers {
├──errors.rs: 4
│  ├──17: pub struct Message {
│  ├──37: impl Error for Message {}
│  ├──39: impl fmt::Debug for Message {
│  └──45: impl fmt::Display for Message {
├──formats.rs: 2
│  ├──19: pub struct Chars {
│  └──99: pub struct DisplayRepeater<T>(T, usize);
├──output_processor.rs: 2
│  ├──28: impl File {
│  └──99: impl AsUsize for Value {
├──options.rs: 2
│  ├──42: pub struct Rg;
│  └──44: impl Options for Rg {
├──args.rs: 4
│  ├──18: pub struct ArgInfo {
│  ├──24: impl ArgInfo {
│  ├──38: impl Clone for OpenStrategy {
│  └──50: impl ValueEnum for OpenStrategy {
├──args_menu.rs: 1
│  └──22: pub struct ArgsMenu<'a, 'b> {
├──select_menu.rs: 7
│  ├──34: impl OpenStrategy {
│  ├──46: pub struct SelectMenu<'a, 'b> {
│  ├──61: struct Window {
│  ├──66: impl Window {
│  ├──85: impl Clone for JumpLocation {
│  ├──90: impl Copy for JumpLocation {}
│  └──92: impl JumpLocation {
├──term.rs: 1
│  └──13: pub struct Term<'a> {
├──writer.rs: 10
│  ├──23: impl Clone for PrefixComponent {
│  ├──34: pub struct OpenInfo<'a> {
│  ├──45: struct PathDisplay<'a> {
│  ├──152: struct LineDisplay<'a> {
│  ├──250: struct LongBranchDisplay<'a> {
│  ├──291: struct OverviewDisplay {
│  ├──299: impl Entry for OverviewDisplay {
│  ├──311: impl Display for OverviewDisplay {
│  ├──358: impl Directory {
│  └──443: impl File {
├──config.rs: 3
│  ├──11: pub struct Characters {
│  ├──25: pub struct Config {
│  └──138: impl Config {
├──matcher.rs: 1
│  └──115: impl File {
└──match_system.rs: 10
   ├──24: pub struct Directory {
   ├──32: impl Directory {
   ├──44: pub struct File {
   ├──50: impl File {
   ├──74: pub struct Match {
   ├──80: impl Match {
   ├──105: pub struct Line {
   ├──111: impl Line {
   ├──127:     impl PartialEq for Match {
   └──134:     impl Debug for Match {
```
</details>

<details>
<summary><code>tgrep Print src/select_menu.rs --trim --line-number --char-style=ascii</code></summary>

```
select_menu.rs
+--11: style::{Print, SetBackgroundColor},
+--350: queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
+--371: queue!(self.term, scroll, cursor::MoveTo(START_X, y), Print(line))?;
+--602: Print(config().c.selected_indicator),
+--604: Print(&self.lines[self.selected_id])
+--612: Print(formats::SELECTED_INDICATOR_CLEAR),
+--614: Print(&self.lines[self.selected_id])
+--628: Print(format!(
+--640: Print(format!(
+--653: Print(format!(
```
</details>

<details>
<summary><code>tgrep --files --hidden --glob=!.git</code></summary>

```
treegrep
├──doc
│  ├──treegrep.vim9.txt
│  └──treegrep.nvim.txt
├──benchmarks
│  ├──times
│  └──runner
├──.github
│  └──workflows
│     ├──test.yml
│     ├──cr.yml
│     ├──update_readme
│     └──update_readme.yml
├──lua
│  └──treegrep.lua
├──tests
│  ├──targets
│  │  ├──wide_1
│  │  ├──files_long_branch_expr_1
│  │  ├──line_number
│  │  ├──file
│  │  ├──no_matches
│  │  ├──files_2
│  │  ├──links_1
│  │  ├──links_2
│  │  ├──count
│  │  ├──links_4
│  │  ├──files_long_branch_expr_count_1
│  │  ├──deep
│  │  ├──files_with_expr
│  │  ├──overlapping_tgrep
│  │  ├──overlapping_rg
│  │  ├──glob_inclusion
│  │  ├──files_long_branch_1
│  │  ├──files_long_branch_expr_2
│  │  ├──files_1
│  │  ├──max_depth
│  │  ├──files_long_branch_expr_count_2
│  │  ├──colon
│  │  ├──wide_2
│  │  ├──glob_exclusion
│  │  ├──files_long_branch_2
│  │  └──links_3
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──tests.rs
│  ├──utils.rs
│  └──file_system.rs
├──plugin
│  └──treegrep.vim
├──src
│  ├──searchers.rs
│  ├──errors.rs
│  ├──formats.rs
│  ├──log.rs
│  ├──output_processor.rs
│  ├──options.rs
│  ├──args.rs
│  ├──args_menu.rs
│  ├──select_menu.rs
│  ├──term.rs
│  ├──main.rs
│  ├──writer.rs
│  ├──config.rs
│  ├──matcher.rs
│  └──match_system.rs
├──LICENSE
├──Cargo.lock
├──.gitignore
├──README.md
├──todos.md
└──Cargo.toml
```
</details>

<details>
<summary><code>tgrep --files --long-branch --hidden --glob=!.git</code></summary>

```
treegrep
├──doc
│  └──treegrep.vim9.txt, treegrep.nvim.txt
├──benchmarks
│  └──times, runner
├──.github
│  └──workflows
│     └──test.yml, cr.yml, update_readme, update_readme.yml
├──lua
│  └──treegrep.lua
├──tests
│  ├──targets
│  │  ├──wide_1, files_long_branch_expr_1, line_number, file, no_matches
│  │  ├──files_2, links_1, links_2, count, links_4
│  │  ├──files_long_branch_expr_count_1, deep, files_with_expr, overlapping_tgrep, overlapping_rg
│  │  ├──glob_inclusion, files_long_branch_1, files_long_branch_expr_2, files_1, max_depth
│  │  ├──files_long_branch_expr_count_2, colon, wide_2, glob_exclusion, files_long_branch_2
│  │  └──links_3
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  └──tests.rs, utils.rs, file_system.rs
├──plugin
│  └──treegrep.vim
├──src
│  ├──searchers.rs, errors.rs, formats.rs, log.rs, output_processor.rs
│  ├──options.rs, args.rs, args_menu.rs, select_menu.rs, term.rs
│  └──main.rs, writer.rs, config.rs, matcher.rs, match_system.rs
├──LICENSE, Cargo.lock, .gitignore, README.md, todos.md
└──Cargo.toml
```
</details>

### *--help*
```
treegrep 1.2.1

by Timothy Cronin

home page: https://github.com/4imothy/treegrep

regex pattern matcher that displays results in a tree structure with an interface to jump to matched text

tgrep [OPTIONS] [positional regexp] [positional target]
Arguments:
  [positional regexp]  the regex expression
  [positional target]  the path to search, if not provided, search the current directory

Options:
  -e, --regexp <>                    the regex expression
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
      --pcre2                        enable PCRE2
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
      --searcher <>                  executable to do the searching [possible values: rg, tgrep]
      --selection-file <>            file to write selection to (first line: file path, second line: line number if applicable)
      --repeat-file <>               file where arguments are saved
      --repeat                       repeats the last saved search
  -h, --help                         print help
  -V, --version                      print version

arguments are prefixed with the contents of the TREEGREP_DEFAULT_OPTS environment variable
```
