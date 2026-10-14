## treegrep

treegrep is a regex pattern matcher that displays results in a tree structure with an interface to jump to matched text.

[![test](https://github.com/4imothy/treegrep/actions/workflows/test.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)
[![release](https://github.com/4imothy/treegrep/actions/workflows/cr.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)

[demo video](https://youtu.be/fPFmQr08B_I)

[examples](#examples), [integrations](#integrations), and [help](#--help).

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

### integrations
<details>
<summary><em>neovim</em></summary>

- setup using [lazy.nvim](https://github.com/folke/lazy.nvim)
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
    vim.keymap.set('n', '<leader>tt', function() require('treegrep').tgrep_with('--menu --live') end)
    vim.keymap.set('n', '<leader>tr', function() require('treegrep').tgrep_with('--repeat --select --live') end)
    vim.keymap.set('n', '<leader>tf', function() require('treegrep').tgrep_with('--files --select --live') end)
  end,
}
```
</details>
<details>
<summary><em>helix</em></summary>

- keybind to run treegrep and open selection
```toml
space.t = [
  ':sh rm -f /tmp/tgrep-select',
  ':insert-output tgrep --no-alternate-screen --menu --live --selection-file=/tmp/tgrep-select --repeat-file=/tmp/tgrep-repeat > /dev/tty',
  ':open %sh{ f=$(sed -n 1p /tmp/tgrep-select); l=$(sed -n 2p /tmp/tgrep-select); [ -n "$l" ] && echo "$f:$l" || echo "$f"; }',
  ':redraw',
  ':set-option mouse false',
  ':set-option mouse true',
]
```
</details>
<details>
<summary><em>vim</em></summary>

- setup using [vim-plug](https://github.com/junegunn/vim-plug)
```vim
Plug '4imothy/treegrep', {'do': {-> TgrepBuild()}}

let g:tgrep_selection_file = '/tmp/tgrep-select'
let g:tgrep_repeat_file = '/tmp/tgrep-repeat'

nnoremap <leader>tt :call TgrepWith('--menu')<cr>
nnoremap <leader>tr :call TgrepWith('--repeat --select')<cr>
nnoremap <leader>tf :call TgrepWith('--files --select')<cr>
```
</details>
<details>
<summary><em>emacs</em></summary>

- sample setup using [use-package](https://cgit.git.savannah.gnu.org/cgit/emacs.git/tree/lisp/use-package) with the built-in `term`
```elisp
(use-package treegrep
  :ensure nil
  :load-path "path/to/treegrep/plugin"
  :preface
  (let ((src (expand-file-name "path/to/treegrep/plugin/treegrep.el")))
    (when (file-newer-than-file-p src (concat src "c"))
      (byte-compile-file src)))
  :custom
  (tgrep-selection-file "/tmp/tgrep-select")
  (tgrep-repeat-file    "/tmp/tgrep-repeat")
  (tgrep-terminal
   (lambda (cmd)
     (when-let ((old (get-buffer "*tgrep*")))
       (when-let ((p (get-buffer-process old)))
         (set-process-query-on-exit-flag p nil)
         (delete-process p))
       (kill-buffer old))
     (let ((buf (get-buffer-create "*tgrep*")))
       (switch-to-buffer buf)
       (term-mode)
       (setq-local scroll-margin 0)
       (setq-local scroll-conservatively 0)
       (setq-local term-suppress-hard-newline t)
       (setq-local show-trailing-whitespace nil)
       (setq-local display-line-numbers nil)
       (setq-local truncate-lines t)
       (let ((term-exec-hook nil))
         (term-exec buf "tgrep" shell-file-name nil (list shell-command-switch cmd)))
       (term-char-mode)
       (let ((proc (get-buffer-process buf)))
         (set-process-query-on-exit-flag proc nil)
         proc))))
  :config
  (require 'term)
  (tgrep-build)
  :bind
  ("C-c t t" . (lambda () (interactive) (delete-other-windows) (tgrep-with "--menu --path=/some/path")))
  ("C-c t r" . (lambda () (interactive) (delete-other-windows) (tgrep-with "--repeat --select")))
  ("C-c t f" . (lambda () (interactive) (delete-other-windows) (tgrep-with "--files --select --path=/some/path"))))
```
</details>

### examples
<details>
<summary><code>tgrep --regexp \bstruct\s+\w+ --regexp \bimpl\s+\w+ --path src --line-number --context=1 --count</code></summary>

```
src: 9
├──args.rs: 11
│  ├──-1: 
│  ├──25: impl ValueEnum for OpenStrategy {
│  ├──+1:     fn value_variants<'a>() -> &'a [Self] {
│  ├──-1: 
│  ├──64: impl ValueEnum for CompletionShell {
│  ├──+1:     fn value_variants<'a>() -> &'a [Self] {
│  ├──-1: 
│  ├──90: impl CompletionShell {
│  ├──+1:     pub fn generate(&self, cmd: &mut Command, bin: &str, out: &mut dyn std::io::Write) {
│  ├──-1: #[derive(Clone)]
│  ├──142: pub struct ColorParser;
│  ├──+1: 
│  ├──144: impl clap::builder::TypedValueParser for ColorParser {
│  ├──+1:     type Value = Color;
│  ├──-1: #[derive(Clone)]
│  ├──205: pub struct KeyCodeParser;
│  ├──+1: 
│  ├──207: impl clap::builder::TypedValueParser for KeyCodeParser {
│  ├──+1:     type Value = KeyCode;
│  ├──-1: #[cfg_attr(test, derive(PartialEq, Debug))]
│  ├──290: pub struct ScopedRegexp {
│  ├──+1:     pub glob: String,
│  ├──-1: #[derive(Clone)]
│  ├──296: pub struct RegexpGlobParser;
│  ├──+1: 
│  ├──298: impl clap::builder::TypedValueParser for RegexpGlobParser {
│  ├──+1:     type Value = ScopedRegexp;
│  ├──-1: )]
│  ├──450: pub struct Args {
│  ╰──+1:     #[arg(
├──config.rs: 8
│  ├──-1: #[derive(Clone)]
│  ├──33: pub struct KeyBindings {
│  ├──+1:     pub down: Vec<KeyCode>,
│  ├──-1: #[derive(Clone)]
│  ├──58: pub struct Characters {
│  ├──+1:     pub bl: char,
│  ├──-1: #[derive(Clone)]
│  ├──79: pub struct Colors {
│  ├──+1:     pub file: Color,
│  ├──-1: #[derive(Clone)]
│  ├──92: pub struct SearchParams {
│  ├──+1:     pub regexps: Vec<String>,
│  ├──-1: #[derive(Clone)]
│  ├──117: pub struct Config {
│  ├──+1:     pub search: SearchParams,
│  ├──-1: 
│  ├──394: impl Config {
│  ├──+1:     pub fn get_styling(matches: &ArgMatches) -> (bool, bool) {
│  ├──-1: 
│  ├──717: fn flag_arg(id: &str, value: impl AsRef<OsStr>) -> OsString {
│  ├──+1:     let mut arg = OsString::from(id_to_flag(id));
│  ├──-1: 
│  ├──724: impl SearchParams {
│  ╰──+1:     pub fn has_expr(&self) -> bool {
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
├──match_system.rs: 8
│  ├──-1: 
│  ├──25: pub struct Directory {
│  ├──+1:     pub path: PathBuf,
│  ├──-1: 
│  ├──32: impl Directory {
│  ├──+1:     pub fn new(path: &Path, links: bool) -> Result<Self, Message> {
│  ├──-1: 
│  ├──43: pub struct File {
│  ├──+1:     pub path: PathBuf,
│  ├──-1: 
│  ├──49: impl File {
│  ├──+1:     pub fn from_pathbuf(path: PathBuf, links: bool) -> Result<Self, Message> {
│  ├──-1: #[cfg_attr(test, derive(PartialEq, Debug))]
│  ├──75: pub struct Match {
│  ├──+1:     pub regexp_id: usize,
│  ├──-1: 
│  ├──81: impl Match {
│  ├──+1:     pub fn new(regexp_id: usize, start: usize, end: usize) -> Self {
│  ├──-1: 
│  ├──106: pub struct Line {
│  ├──+1:     pub content: Vec<u8>,
│  ├──-1: 
│  ├──113: impl Line {
│  ╰──+1:     pub fn new(content: Vec<u8>, mut matches: Vec<Match>, line_num: usize) -> Self {
├──matcher.rs: 5
│  ├──-1: 
│  ├──32: struct Matcher {
│  ├──+1:     patterns: Vec<String>,
│  ├──-1: 
│  ├──84: impl Combined {
│  ├──+1:     fn new(patterns: &[&str], use_pcre2: bool) -> Result<Self, Message> {
│  ├──-1: 
│  ├──114: impl Individual {
│  ├──+1:     fn new(patterns: &[String], use_pcre2: bool) -> Result<Self, Message> {
│  ├──-1: 
│  ├──144: impl Matcher {
│  ├──+1:     fn new(search: &SearchParams) -> Result<Self, Message> {
│  ├──-1: 
│  ├──220: struct MatchSink<'a> {
│  ╰──+1:     lines: Vec<Line>,
├──menu.rs: 8
│  ├──-1: 
│  ├──58: impl ViewAnchor {
│  ├──+1:     fn next(&mut self) {
│  ├──-1: 
│  ├──75: impl FoldCycle {
│  ├──+1:     fn next(&mut self) {
│  ├──-1: 
│  ├──85: struct DoubleClick {
│  ├──+1:     down_row: u16,
│  ├──-1: 
│  ├──90: impl DoubleClick {
│  ├──+1:     fn new() -> Self {
│  ├──-1: 
│  ├──117: struct Window {
│  ├──+1:     first: isize,
│  ├──-1: 
│  ├──122: impl Window {
│  ├──+1:     fn new() -> Self {
│  ├──-1: 
│  ├──528: pub struct Menu<'a, 'b> {
│  ├──+1:     in_menu: bool,
│  ├──-1: 
│  ├──1756: impl OpenStrategy {
│  ╰──+1:     fn from(editor: &str) -> Self {
├──style.rs: 1
│  ├──-1: 
│  ├──23: pub struct DisplayRepeater<T>(T, usize);
│  ╰──+1: impl<T: Display> Display for DisplayRepeater<T> {
├──term.rs: 1
│  ├──-1: 
│  ├──15: pub struct Term<'a> {
│  ╰──+1:     pub height: u16,
╰──writer.rs: 22
   ├──-1: 
   ├──57: impl HighlightEvent<'_> {
   ├──+1:     fn priority(&self) -> u8 {
   ├──-1: 
   ├──68: struct RenderStyle<'a> {
   ├──+1:     selected: bool,
   ├──-1: 
   ├──73: impl RenderStyle<'_> {
   ├──+1:     fn bg(&self) -> Option<Color> {
   ├──-1: 
   ├──218: pub struct OpenInfo<'a> {
   ├──+1:     pub path: &'a Path,
   ├──-1: 
   ├──231: pub struct WithFilter<'a> {
   ├──+1:     pub entry: &'a dyn Entry,
   ├──-1: 
   ├──237: impl Display for WithFilter<'_> {
   ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
   ├──-1: 
   ├──243: struct PathDisplay {
   ├──+1:     prefix: Option<Vec<PrefixComponent>>,
   ├──-1: 
   ├──254: impl PathDisplay {
   ├──+1:     fn new(
   ├──-1: 
   ├──287: impl Entry for PathDisplay {
   ├──+1:     fn render(&self, f: &mut fmt::Formatter, filter: &str, selected: bool) -> fmt::Result {
   ├──-1: 
   ├──330: impl Display for PathDisplay {
   ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
   ├──-1: 
   ├──394: struct LineDisplay {
   ├──+1:     prefix: Vec<PrefixComponent>,
   ├──-1: 
   ├──405: impl Entry for LineDisplay {
   ├──+1:     fn render(&self, f: &mut fmt::Formatter, filter: &str, selected: bool) -> fmt::Result {
   ├──-1: 
   ├──551: impl Display for LineDisplay {
   ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
   ├──-1: 
   ├──557: struct LongBranchDisplay {
   ├──+1:     prefix: Vec<PrefixComponent>,
   ├──-1: 
   ├──564: impl Entry for LongBranchDisplay {
   ├──+1:     fn render(&self, f: &mut fmt::Formatter, filter: &str, selected: bool) -> fmt::Result {
   ├──-1: 
   ├──614: impl Display for LongBranchDisplay {
   ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
   ├──-1: 
   ├──620: struct OverviewDisplay {
   ├──+1:     dirs: usize,
   ├──-1: 
   ├──630: impl OverviewDisplay {
   ├──+1:     fn add_dir(&mut self, dir: &Directory) {
   ├──-1: 
   ├──654: impl Entry for OverviewDisplay {
   ├──+1:     fn render(&self, f: &mut fmt::Formatter, filter: &str, selected: bool) -> fmt::Result {
   ├──-1: 
   ├──699: impl Display for OverviewDisplay {
   ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
   ├──-1: 
   ├──721: impl Directory {
   ├──+1:     fn to_lines(
   ├──-1: 
   ├──825: impl File {
   ╰──+1:     fn to_lines(
```
</details>

<details>
<summary><code>tgrep Print src/menu.rs --trim --line-number --char-vertical=| --char-horizontal=- --char-top-left=+ --char-top-right=+ --char-bottom-left=+ --char-bottom-right=+ --char-tee=+</code></summary>

```
menu.rs
+--19: style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
+--402: Print(format!(
+--413: Print(format!(
+--425: Print(format!(
+--736: Print(WithFilter {
+--744: queue!(self.term, Print(&cfg.chars.ellipsis))?;
+--760: Print(style::style_with(indicator, c, &cfg).on(b))
+--762: (Some(b), None) => queue!(self.term, Print(indicator.on(b)))?,
+--763: (None, _) => queue!(self.term, Print(indicator))?,
+--783: Print(cfg.chars.selected_indicator_clear.as_str()),
+--1122: Print(&top),
+--1137: Print(hint),
+--1164: Print(display),
+--1260: Print(line)
+--1280: queue!(self.term, cursor::MoveTo(0, y), Print(msg))?;
+--1497: Print(cfg.chars.selected_indicator_clear.as_str())
```
</details>

<details>
<summary><code>tgrep --files --hidden --glob=!.git</code></summary>

```
treegrep
├──.github
│  ╰──workflows
│     ├──cr.yml
│     ├──test.yml
│     ├──update_readme
│     ╰──update_readme.yml
├──benchmarks
│  ├──runner
│  ╰──times
├──doc
│  ├──treegrep.nvim.txt
│  ╰──treegrep.vim9.txt
├──lua
│  ╰──treegrep.lua
├──plugin
│  ├──treegrep.el
│  ╰──treegrep.vim
├──src
│  ├──args.rs
│  ├──config.rs
│  ├──errors.rs
│  ├──log.rs
│  ├──main.rs
│  ├──match_system.rs
│  ├──matcher.rs
│  ├──menu.rs
│  ├──style.rs
│  ├──term.rs
│  ╰──writer.rs
├──tests
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──context_a1
│  │  ├──context_b1
│  │  ├──context_c1
│  │  ├──deep
│  │  ├──file
│  │  ├──files
│  │  ├──files_long_branch
│  │  ├──files_long_branch_expr
│  │  ├──files_long_branch_expr_count
│  │  ├──files_with_expr
│  │  ├──glob_exclusion
│  │  ├──input_cmd
│  │  ├──input_stdin
│  │  ├──links
│  │  ├──max_depth
│  │  ├──menu_custom_keys
│  │  ├──menu_filter
│  │  ├──menu_fold_cycle
│  │  ├──menu_input_cmd
│  │  ├──menu_jump
│  │  ├──menu_navigate
│  │  ├──menu_search_args
│  │  ├──menu_search_mode
│  │  ├──multiple_paths
│  │  ├──no_matches
│  │  ├──overlapping
│  │  ├──overview_dir
│  │  ├──overview_file
│  │  ├──pcre2_lookbehind
│  │  ├──regexp_glob
│  │  ├──summary_dir
│  │  ├──summary_file
│  │  ╰──wide
│  ├──file_system.rs
│  ├──tests.rs
│  ├──tmux.rs
│  ╰──utils.rs
├──.gitignore
├──Cargo.lock
├──Cargo.toml
├──LICENSE
├──README.md
╰──rustfmt.toml
```
</details>

<details>
<summary><code>tgrep --files --branch-each=5 --hidden --glob=!.git</code></summary>

```
treegrep
├──.github
│  ╰──workflows
│     ╰──cr.yml, test.yml, update_readme, update_readme.yml
├──benchmarks
│  ╰──runner, times
├──doc
│  ╰──treegrep.nvim.txt, treegrep.vim9.txt
├──lua
│  ╰──treegrep.lua
├──plugin
│  ╰──treegrep.el, treegrep.vim
├──src
│  ├──args.rs, config.rs, errors.rs, log.rs, main.rs
│  ├──match_system.rs, matcher.rs, menu.rs, style.rs, term.rs
│  ╰──writer.rs
├──tests
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──context_a1, context_b1, context_c1, deep, file
│  │  ├──files, files_long_branch, files_long_branch_expr, files_long_branch_expr_count, files_with_expr
│  │  ├──glob_exclusion, input_cmd, input_stdin, links, max_depth
│  │  ├──menu_custom_keys, menu_filter, menu_fold_cycle, menu_input_cmd, menu_jump
│  │  ├──menu_navigate, menu_search_args, menu_search_mode, multiple_paths, no_matches
│  │  ├──overlapping, overview_dir, overview_file, pcre2_lookbehind, regexp_glob
│  │  ╰──summary_dir, summary_file, wide
│  ╰──file_system.rs, tests.rs, tmux.rs, utils.rs
├──.gitignore, Cargo.lock, Cargo.toml, LICENSE, README.md
╰──rustfmt.toml
```
</details>

<details>
<summary><code>tgrep --regexp-glob *.rs=_mouse --regexp-glob *.lua=path --line-number</code></summary>

```
treegrep
├──lua
│  ╰──treegrep.lua
│     ├──23: local current_file_path = debug.getinfo(1, 'S').source:sub(2)
│     ├──24: local plugin_name = vim.fn.fnamemodify(current_file_path, ':t:r')
│     ├──25: local project_root = vim.fs.abspath(vim.fs.dirname(vim.fs.dirname(current_file_path)))
│     ├──26: local tgrep_bin = vim.fs.joinpath(project_root, 'target', 'release', 'tgrep')
│     ╰──109:         '--manifest-path=' .. vim.fn.fnameescape(vim.fs.joinpath(project_root, 'Cargo.toml'))
╰──src
   ├──args.rs
   │  ╰──693:     pub no_mouse: bool,
   ├──config.rs
   │  ╰──522:             mouse: !args.no_mouse,
   ╰──term.rs
      ├──20:     with_mouse: bool,
      ├──28:         with_mouse: bool,
      ├──38:             with_mouse,
      ├──59:         if self.with_mouse {
      ├──65:             let with_mouse = self.with_mouse;
      ├──68:                 Term::exit(&mut io::stderr(), with_mouse, with_alternate_screen).ok();
      ├──77:     fn exit<W>(w: &mut W, with_mouse: bool, with_alternate_screen: bool) -> io::Result<()>
      ├──92:         if with_mouse {
      ╰──104:         Term::exit(self, self.with_mouse, self.with_alternate_screen)?;
```
</details>

### *--help*
```
tgrep 2.2.0

by Timothy Cronin

home page: https://github.com/4imothy/treegrep

regex pattern matcher that displays results in a tree structure with an interface to jump to matched text

tgrep [OPTIONS] <POSITIONAL_REGEXP|--regexp <>|--regexp-glob <glob=regexp>|--files|--completions <>|--menu|--repeat> [POSITIONAL_PATH]...

arguments:
  [POSITIONAL_REGEXP]
          a regex expression to search for

  [POSITIONAL_PATH]...
          the paths to search, if none provided, search the current directory

options:
  -e, --regexp <>
          a regex expression to search for (saved for repeat)

      --regexp-glob <glob=regexp>
          search for a regex expression only in files matching a glob (saved for repeat)

  -p, --path <>
          the paths to search, if none provided, search the current directory (saved for repeat)

      --input-cmd <>
          command whose output is read in place of stdin

      --input-type <>
          read input as this type (saved for repeat)
          
          [possible values: paths]

  -s, --select
          results are shown in a selection interface for opening

  -m, --menu
          open a search and selection interface

      --live
          trigger search on every keystroke in the menu

      --live-delay <>
          milliseconds to wait after the last keystroke before triggering a live search

  -f, --files
          if an expression is given, hide matched content, otherwise, show the files that would be searched (saved for repeat)

  -., --hidden
          search hidden files (saved for repeat)

  -n, --line-number
          show the line numbers of matches (saved for repeat)

  -c, --count
          display number of files matched in directory and number of lines matched in a file (saved for repeat)

  -g, --glob <>
          rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic (saved for repeat)

  -l, --links
          search linked paths (saved for repeat)

  -o, --overview
          conclude results with an overview (saved for repeat)

  -S, --overview-only
          only show the overview, not the results (saved for repeat)

  -d, --max-depth <>
          the max depth to search (saved for repeat)

  -C, --context <>
          number of lines to show before and after each match

  -B, --before-context <>
          number of lines to show before each match (saved for repeat)

  -A, --after-context <>
          number of lines to show after each match (saved for repeat)

      --max-length <>
          set the max length for a matched line (saved for repeat)

      --no-ignore
          disable ignore files (saved for repeat)

      --pcre2
          use PCRE2 instead of the default regex engine (saved for repeat)

      --trim
          trim whitespace at the beginning of lines (saved for repeat)

      --auto-open
          if there is only one match, open it in the configured editor

      --repeat
          repeats the last saved search

      --branch-len <>
          number of characters to show before a match
          
          [default: 3]

      --branch-each <>
          number of files to print on each branch (saved for repeat)
          
          [default: 1]

      --no-color
          disable colors

      --no-bold
          disable bold

      --threads <>
          set the number of threads to use

      --editor <>
          command used to open selections

      --open-like <>
          command line syntax for opening a file at a line
          
          [possible values: vi, hx, code, jed, default]

      --completions <>
          generate completions for given shell
          
          [possible values: bash, elvish, fish, powershell, zsh, nushell]

      --no-mouse
          disable mouse events

      --no-alternate-screen
          disable the terminal alternate screen

      --file-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --dir-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --text-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --branch-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --line-number-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --match-colors <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --selected-indicator-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --selected-bg-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --filter-highlight-color <>
          black, white, red, green, yellow, blue, magenta, cyan, grey, rgb(_._._), ansi(_)

      --char-vertical <>
          vertical branch character
          
          [default: │]

      --char-horizontal <>
          horizontal branch character
          
          [default: ─]

      --char-top-left <>
          top-left corner character
          
          [default: ╭]

      --char-top-right <>
          top-right corner character
          
          [default: ╮]

      --char-bottom-left <>
          bottom-left corner character
          
          [default: ╰]

      --char-bottom-right <>
          bottom-right corner character
          
          [default: ╯]

      --char-tee <>
          tee branch character
          
          [default: ├]

      --ellipsis <>
          folded indicator
          
          [default: ⤵]

      --search-prompt <>
          search mode prompt
          
          [default: "➜ "]

      --search-prompt-inactive <>
          search prompt when not searching
          
          [default: "- "]

      --filter-prompt <>
          filter mode prompt
          
          [default: /]

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
          move down to the next path
          
          [default: } ]]

      --key-up-path <>
          move up to the previous path
          
          [default: { []

      --key-down-same-depth <>
          move down to the next path at same depth
          
          [default: ) d]

      --key-up-same-depth <>
          move up to the previous path at same depth
          
          [default: ( u]

      --key-top <>
          move to the top
          
          [default: home g <]

      --key-bottom <>
          move to the bottom
          
          [default: end G >]

      --key-page-down <>
          page down
          
          [default: pagedown f]

      --key-page-up <>
          page up
          
          [default: pageup b]

      --key-cycle-view <>
          cycle cursor position (top/center/bottom)
          
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

      --key-fold-cycle <>
          cycle global fold (dirs/files/none)
          
          [default: Z]

      --key-filter <>
          filter within results
          
          [default: / s]

      --key-search <>
          enter search mode
          
          [default: :]

      --key-submit-search <>
          submit search query
          
          [default: enter]

      --selection-file <>
          file to write selection to

      --repeat-file <>
          file to save and replay regexp searches

  -h, --help
          

  -V, --version
          

arguments are prefixed with the contents of the TREEGREP_DEFAULT_OPTS environment variable
```
