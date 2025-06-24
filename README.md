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

- sample function that runs treegrep in a window and opens selection,
  with shortcuts for reusing previous arguments or searching files

```lua
local NORMAL = 0
local REPEAT = 1
local FILES = 2
local function tgrep_float(opt)
    local buf = vim.api.nvim_create_buf(false, true)

    local original_win = vim.api.nvim_get_current_win()

    local win = vim.api.nvim_open_win(buf, true, {
        relative = 'editor',
        style = 'minimal',
        border = 'rounded',
        width = math.floor(vim.o.columns * 0.8),
        height = math.floor(vim.o.lines * 0.8),
        col = math.floor((vim.o.columns - (vim.o.columns * 0.8)) / 2),
        row = math.floor((vim.o.lines - (vim.o.lines * 0.8)) / 2)
    })

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
<details>
<summary><em>vim</em></summary>

- sample function that runs treegrep in a window and opens selection,
  with shortcuts for reusing previous arguments or searching files

```vim
vim9script

const NORMAL: number = 0
const REPEAT: number = 1
const FILES: number = 2

def g:TgrepFloat(opt: number)
  var original_win: number = win_getid()

  var select_file: string = '/tmp/tgrep-select'
  var repeat_file: string = '/tmp/tgrep-repeat'
  var cmd: string = 'tgrep --selection-file=' .. fnameescape(select_file) ..
                    ' --repeat-file=' .. fnameescape(repeat_file)

  if opt == NORMAL
    cmd ..= ' --menu'
  elseif opt == REPEAT
    cmd ..= ' --repeat'
  elseif opt == FILES
    cmd ..= ' --select --files'
  endif

  var buf: number
  var win: number

  def OnTermExit(_job: job, _status: number): void
      popup_close(win)
      if bufexists(buf)
          execute 'bdelete! ' .. buf
      endif
      call win_gotoid(original_win)

      if filereadable(select_file)
          var lines: list<string> = readfile(select_file)
          if len(lines) == 0
              return
          endif
          var edit_cmd: string = 'edit ' .. fnameescape(lines[0])
          if len(lines) > 1
              edit_cmd ..= ' | :' .. lines[1]
          endif
          execute edit_cmd .. ' | redraw'
      endif
  enddef

  buf = term_start(cmd, {
      term_name: 'tgrep',
      hidden: true,
      term_finish: 'close',
      exit_cb: OnTermExit
  })

  var width: number = float2nr(&columns * 0.8)
  var height: number = float2nr(&lines * 0.8)
  var col: number = float2nr((&columns - width) / 2)
  var row: number = float2nr((&lines - height) / 2)

  win = popup_create(buf, {
      line: row,
      col: col,
      minwidth: width,
      minheight: height,
      maxwidth: width,
      maxheight: height,
      border: [1, 1, 1, 1],
      borderchars: ['─', '│', '─', '│', '╭', '╮', '╯', '╰'],
      padding: [0, 0, 0, 0],
      zindex: 1,
      drag: 1,
      pos: 'center',
  })
enddef

nnoremap ,tt :call TgrepFloat(0)<CR>
nnoremap ,tr :call TgrepFloat(1)<CR>
nnoremap ,tf :call TgrepFloat(2)<CR>
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
