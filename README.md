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

- sample function that runs treegrep in a window and open selection,
  with shortcuts for reusing previous arguments or searching files

```lua
local NORMAL = 0
local REPEAT = 1
local FILES = 2
local function tgrep_float(opt)
    local buf = vim.api.nvim_create_buf(false, true)

    local original_win = vim.api.nvim_get_current_win()

    local opts = {
        relative = 'editor',
        width = math.floor(vim.o.columns * 0.8),
        height = math.floor(vim.o.lines * 0.8),
        col = math.floor((vim.o.columns - (vim.o.columns * 0.8)) / 2),
        row = math.floor((vim.o.lines - (vim.o.lines * 0.8)) / 2),
        anchor = 'NW',
        style = 'minimal',
        border = 'rounded'
    }

    local win = vim.api.nvim_open_win(buf, true, opts)

    local select_file = '/tmp/tgrep-select'
    local repeat_file = '/tmp/tgrep-repeat'
    local cmd = string.format(
        'tgrep --selection-file=%s --repeat-file=%s %s',
        vim.fn.fnameescape(select_file),
        vim.fn.fnameescape(repeat_file),
        opt == NORMAL and '--menu' or
        opt == REPEAT and '--repeat' or
        opt == FILES and '--select --files'
    )

    vim.fn.termopen(cmd, {
        on_exit = function()
            vim.api.nvim_win_close(win, true)
            vim.api.nvim_buf_delete(buf, { force = true })
            vim.api.nvim_set_current_win(original_win)
            if vim.fn.filereadable(select_file) == 1 then
                local lines = vim.fn.readfile(select_file)
                if #lines == 0 then
                    return
                end
                local edit_cmd = 'edit ' .. vim.fn.fnameescape(lines[1])
                if #lines > 1 then
                    edit_cmd = edit_cmd .. ' | ' .. lines[2]
                end
                vim.cmd(edit_cmd .. ' | redraw')
            end
        end
    })

    vim.cmd('startinsert')
end

vim.keymap.set('n', '<leader>tt', function() tgrep_float(NORMAL) end, { noremap = true, silent = true })
vim.keymap.set('n', '<leader>tr', function() tgrep_float(REPEAT) end, { noremap = true, silent = true })
vim.keymap.set('n', '<leader>tf', function() tgrep_float(FILES) end, { noremap = true, silent = true })
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
