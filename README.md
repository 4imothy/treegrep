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
src: 13
├──args.rs: 6
│  ├──21: pub struct ArgInfo {
│  ├──27: impl ArgInfo {
│  ├──42: impl ValueEnum for OpenStrategy {
│  ├──91: impl ValueEnum for CharacterStyle {
│  ├──121: struct ColorParser;
│  └──123: impl clap::builder::TypedValueParser for ColorParser {
├──errors.rs: 4
│  ├──14: pub struct Message {
│  ├──17: impl Error for Message {}
│  ├──34: impl fmt::Debug for Message {
│  └──40: impl fmt::Display for Message {
├──match_system.rs: 8
│  ├──23: pub struct Directory {
│  ├──31: impl Directory {
│  ├──43: pub struct File {
│  ├──49: impl File {
│  ├──74: pub struct Match {
│  ├──80: impl Match {
│  ├──105: pub struct Line {
│  └──111: impl Line {
├──options.rs: 2
│  ├──40: pub struct Rg;
│  └──42: impl Options for Rg {
├──matcher.rs: 1
│  └──114: impl File {
├──output_processor.rs: 1
│  └──31: impl File {
├──searchers.rs: 1
│  └──39: impl Searchers {
├──args_menu.rs: 1
│  └──21: pub struct ArgsMenu<'a, 'b> {
├──style.rs: 2
│  ├──22: pub struct Chars {
│  └──102: pub struct DisplayRepeater<T>(T, usize);
├──term.rs: 1
│  └──15: pub struct Term<'a> {
├──config.rs: 5
│  ├──17: pub struct Characters {
│  ├──31: pub struct Colors {
│  ├──42: impl args::Color {
│  ├──60: pub struct Config {
│  └──174: impl Config {
├──select_menu.rs: 5
│  ├──33: impl OpenStrategy {
│  ├──45: pub struct SelectMenu<'a, 'b> {
│  ├──60: struct Window {
│  ├──65: impl Window {
│  └──86: impl JumpLocation {
└──writer.rs: 9
   ├──26: pub struct OpenInfo<'a> {
   ├──37: struct PathDisplay<'a> {
   ├──150: struct LineDisplay<'a> {
   ├──262: struct LongBranchDisplay<'a> {
   ├──303: struct OverviewDisplay {
   ├──311: impl Entry for OverviewDisplay {
   ├──323: impl Display for OverviewDisplay {
   ├──370: impl Directory {
   └──455: impl File {
```
</details>

<details>
<summary><code>tgrep Print src/select_menu.rs --trim --line-number --char-style=ascii</code></summary>

```
select_menu.rs
+--8: style::{Print, SetBackgroundColor},
+--344: queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
+--365: queue!(self.term, scroll, cursor::MoveTo(START_X, y), Print(line))?;
+--599: Print(style::style_with(config().chars.selected_indicator, c)),
+--603: queue!(self.term, Print(config().chars.selected_indicator),)?;
+--608: Print(&self.lines[self.selected_id])
+--616: Print(style::SELECTED_INDICATOR_CLEAR),
+--618: Print(&self.lines[self.selected_id])
+--632: Print(format!(
+--644: Print(format!(
+--657: Print(format!(
```
</details>

<details>
<summary><code>tgrep --files --hidden --glob=!.git</code></summary>

```
treegrep
├──tests
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──wide_2
│  │  ├──files_long_branch_expr_1
│  │  ├──deep
│  │  ├──overlapping_tgrep
│  │  ├──line_number
│  │  ├──no_matches
│  │  ├──files_1
│  │  ├──links_1
│  │  ├──files_with_expr
│  │  ├──max_depth
│  │  ├──files_long_branch_expr_count_2
│  │  ├──overlapping_rg
│  │  ├──files_2
│  │  ├──files_long_branch_expr_2
│  │  ├──links_3
│  │  ├──files_long_branch_1
│  │  ├──links_4
│  │  ├──count
│  │  ├──file
│  │  ├──wide_1
│  │  ├──links_2
│  │  ├──files_long_branch_expr_count_1
│  │  ├──glob_inclusion
│  │  ├──files_long_branch_2
│  │  ├──colon
│  │  └──glob_exclusion
│  ├──utils.rs
│  ├──file_system.rs
│  └──tests.rs
├──benchmarks
│  ├──times
│  └──runner
├──doc
│  ├──treegrep.vim9.txt
│  └──treegrep.nvim.txt
├──src
│  ├──args.rs
│  ├──errors.rs
│  ├──match_system.rs
│  ├──options.rs
│  ├──matcher.rs
│  ├──output_processor.rs
│  ├──searchers.rs
│  ├──args_menu.rs
│  ├──style.rs
│  ├──term.rs
│  ├──config.rs
│  ├──select_menu.rs
│  ├──log.rs
│  ├──main.rs
│  └──writer.rs
├──.github
│  └──workflows
│     ├──update_readme.yml
│     ├──update_readme
│     ├──test.yml
│     └──cr.yml
├──plugin
│  └──treegrep.vim
├──lua
│  └──treegrep.lua
├──README.md
├──Cargo.lock
├──rustfmt.toml
├──todos.md
├──LICENSE
├──Cargo.toml
└──.gitignore
```
</details>

<details>
<summary><code>tgrep --files --long-branch --hidden --glob=!.git</code></summary>

```
treegrep
├──tests
│  ├──pool
│  │  └──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──wide_2, files_long_branch_expr_1, deep, overlapping_tgrep, line_number
│  │  ├──no_matches, files_1, links_1, files_with_expr, max_depth
│  │  ├──files_long_branch_expr_count_2, overlapping_rg, files_2, files_long_branch_expr_2, links_3
│  │  ├──files_long_branch_1, links_4, count, file, wide_1
│  │  ├──links_2, files_long_branch_expr_count_1, glob_inclusion, files_long_branch_2, colon
│  │  └──glob_exclusion
│  └──utils.rs, file_system.rs, tests.rs
├──benchmarks
│  └──times, runner
├──doc
│  └──treegrep.vim9.txt, treegrep.nvim.txt
├──src
│  ├──args.rs, errors.rs, match_system.rs, options.rs, matcher.rs
│  ├──output_processor.rs, searchers.rs, args_menu.rs, style.rs, term.rs
│  └──config.rs, select_menu.rs, log.rs, main.rs, writer.rs
├──.github
│  └──workflows
│     └──update_readme.yml, update_readme, test.yml, cr.yml
├──plugin
│  └──treegrep.vim
├──lua
│  └──treegrep.lua
├──README.md, Cargo.lock, rustfmt.toml, todos.md, LICENSE
└──Cargo.toml, .gitignore
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
