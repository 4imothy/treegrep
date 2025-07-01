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
│  ╰──134:     impl Debug for Match {
├──args.rs: 4
│  ├──18: pub struct ArgInfo {
│  ├──24: impl ArgInfo {
│  ├──38: impl Clone for OpenStrategy {
│  ╰──50: impl ValueEnum for OpenStrategy {
├──writer.rs: 10
│  ├──23: impl Clone for PrefixComponent {
│  ├──34: pub struct OpenInfo<'a> {
│  ├──43: struct PathDisplay<'a> {
│  ├──144: struct LineDisplay<'a> {
│  ├──236: struct LongBranchDisplay<'a> {
│  ├──271: struct OverviewDisplay {
│  ├──279: impl Entry for OverviewDisplay {
│  ├──285: impl Display for OverviewDisplay {
│  ├──332: impl Directory {
│  ╰──438: impl File {
├──formats.rs: 2
│  ├──19: pub struct Chars {
│  ╰──99: pub struct DisplayRepeater<T>(T, usize);
├──errors.rs: 4
│  ├──8: pub struct Message {
│  ├──28: impl Error for Message {}
│  ├──30: impl fmt::Debug for Message {
│  ╰──36: impl fmt::Display for Message {
├──select_menu.rs: 6
│  ├──21: struct PathInfo {
│  ├──40: impl OpenStrategy {
│  ├──52: impl PathInfo {
│  ├──96: pub struct PickerMenu<'a, 'b> {
│  ├──111: struct Window {
│  ╰──116: impl Window {
├──options.rs: 2
│  ├──42: pub struct Rg;
│  ╰──44: impl Options for Rg {
├──output_processor.rs: 2
│  ├──28: impl File {
│  ╰──99: impl AsUsize for Value {
├──term.rs: 1
│  ╰──13: pub struct Term<'a> {
├──matcher.rs: 1
│  ╰──115: impl File {
├──searchers.rs: 1
│  ╰──34: impl Searchers {
├──config.rs: 3
│  ├──11: pub struct Characters {
│  ├──25: pub struct Config {
│  ╰──139: impl Config {
╰──args_menu.rs: 1
   ╰──19: pub struct ArgsMenu<'a, 'b> {
```
</details>

<details>
<summary><code>tgrep Print src/select_menu.rs --trim --line-number --char-style=ascii</code></summary>

```
select_menu.rs
+--11: style::{Print, SetBackgroundColor},
+--369: queue!(self.term, cursor::MoveTo(START_X, cursor), Print(line))?;
+--390: queue!(self.term, scroll, cursor::MoveTo(START_X, y), Print(line))?;
+--579: Print(config().c.selected_indicator),
+--581: Print(self.lines.get(self.selected_id).unwrap())
+--589: Print(formats::SELECTED_INDICATOR_CLEAR),
+--591: Print(self.lines.get(self.selected_id).unwrap())
+--605: Print(format!(
+--617: Print(format!(
+--630: Print(format!(
```
</details>

<details>
<summary><code>tgrep --files --hidden --glob=!.git</code></summary>

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
│  │  ├──files_with_expr
│  │  ├──files_long_branch_expr_count_1
│  │  ├──glob_inclusion
│  │  ├──files_long_branch_2
│  │  ├──max_depth
│  │  ├──file
│  │  ├──files_long_branch_expr_1
│  │  ├──overlapping_tgrep
│  │  ├──colon
│  │  ├──count
│  │  ├──links_1
│  │  ├──deep
│  │  ├──line_number
│  │  ├──files_2
│  │  ├──wide_1
│  │  ├──files_long_branch_expr_count_2
│  │  ├──overlapping_rg
│  │  ├──files_long_branch_1
│  │  ├──no_matches
│  │  ├──glob_exclusion
│  │  ├──files_long_branch_expr_2
│  │  ├──links_2
│  │  ├──links_3
│  │  ├──links_4
│  │  ├──wide_2
│  │  ╰──files_1
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
│  ├──args_menu.rs
│  ├──term.rs
│  ├──matcher.rs
│  ├──config.rs
│  ├──output_processor.rs
│  ├──log.rs
│  ├──options.rs
│  ├──select_menu.rs
│  ├──errors.rs
│  ├──main.rs
│  ├──formats.rs
│  ├──writer.rs
│  ├──args.rs
│  ╰──match_system.rs
├──Cargo.toml
├──LICENSE
├──Cargo.lock
├──build.rs
├──README.md
├──todos.md
├──.gitignore
╰──release_notes.md
```
</details>

<details>
<summary><code>tgrep --files --long-branch --hidden --glob=!.git</code></summary>

```
treegrep
├──completions
│  ╰──tgrep.bash, tgrep.fish, tgrep.elv, _tgrep, _tgrep.ps1
├──tests
│  ├──targets
│  │  ├──files_with_expr, files_long_branch_expr_count_1, glob_inclusion, files_long_branch_2, max_depth
│  │  ├──file, files_long_branch_expr_1, overlapping_tgrep, colon, count
│  │  ├──links_1, deep, line_number, files_2, wide_1
│  │  ├──files_long_branch_expr_count_2, overlapping_rg, files_long_branch_1, no_matches, glob_exclusion
│  │  ├──files_long_branch_expr_2, links_2, links_3, links_4, wide_2
│  │  ╰──files_1
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ╰──file_system.rs, tests.rs, utils.rs
├──benchmarks
│  ╰──times, runner
├──.github
│  ╰──workflows
│     ╰──cr.yml, update_readme_and_completions.yml, test.yml, update_readme
├──src
│  ├──searchers.rs, args_menu.rs, term.rs, matcher.rs, config.rs
│  ├──output_processor.rs, log.rs, options.rs, select_menu.rs, errors.rs
│  ╰──main.rs, formats.rs, writer.rs, args.rs, match_system.rs
├──Cargo.toml, LICENSE, Cargo.lock, build.rs, README.md
╰──todos.md, .gitignore, release_notes.md
```
</details>

### *--help*
```
treegrep 1.0.0

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
      --glob <>              rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic
      --searcher <>          executable to do the searching [possible values: rg, tgrep]
      --char-style <>        style of characters to use [possible values: ascii, single, double, heavy, rounded, none]
      --editor <>            command used to open selections
      --open-like <>         command line syntax for opening a file at a line [possible values: vi, hx, code, jed, default]
      --long-branch          multiple files from the same directory are shown on the same branch
      --completions <shell>  generate completions for given shell [possible values: bash, elvish, fish, powershell, zsh]
      --selection-file <>    file to write selection to (first line: file path, second line: line number if applicable)
      --repeat-file <>       file where arguments are saved
  -., --hidden               search hidden files
      --repeat               repeats the last saved search
  -n, --line-number          show line number of match
  -f, --files                if a pattern is given hide matched content, otherwise show the files that would be searched
      --links                search linked paths
      --no-ignore            don't use ignore files
  -c, --count                display number of files matched in directory and number of lines matched in a file
      --no-color             don't use colors
      --no-bold              don't bold anything
      --overview             conclude results with an overview
  -s, --select               results are shown in a selection interface for opening
      --menu                 provide arguments and select result through an interface
      --trim                 trim whitespace at the beginning of lines
      --pcre2                enable PCRE2
      --threads <>           set the appropriate number of threads to use
      --max-depth <>         the max depth to search
      --prefix-len <>        number of characters to show before a match [default: 3]
      --max-length <>        set the max length for a matched line
      --long-branch-each <>  number of files to print on each branch [default: 5]
  -h, --help                 Print help
  -V, --version              Print version

arguments are prefixed with the contents of the TREEGREP_DEFAULT_OPTS environment variable
```
