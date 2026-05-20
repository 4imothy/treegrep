## treegrep

treegrep is a regex pattern matcher that displays results in a tree structure with an interface to jump to matched text.

[![test](https://github.com/4imothy/treegrep/actions/workflows/test.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)
[![release](https://github.com/4imothy/treegrep/actions/workflows/cr.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)

[demo video](https://youtu.be/fPFmQr08B_I)

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
  ("C-c t t" . (lambda () (interactive) (delete-other-windows) (tgrep-with "--menu --live --no-mouse --path=/some/path")))
  ("C-c t r" . (lambda () (interactive) (delete-other-windows) (tgrep-with "--repeat --select --live --no-mouse")))
  ("C-c t f" . (lambda () (interactive) (delete-other-windows) (tgrep-with "--files --select --live --no-mouse --path=/some/path"))))
```
</details>

### examples
<details>
<summary><code>tgrep --regexp \bstruct\s+\w+ --regexp \bimpl\s+\w+ --path src --line-number --context=1 --count</code></summary>

```
src: 9
├──style.rs: 1
│  ├──-1: 
│  ├──23: pub struct DisplayRepeater<T>(T, usize);
│  ╰──+1: impl<T: Display> Display for DisplayRepeater<T> {
├──term.rs: 1
│  ├──-1: 
│  ├──15: pub struct Term<'a> {
│  ╰──+1:     pub height: u16,
├──match_system.rs: 8
│  ├──-1: 
│  ├──23: pub struct Directory {
│  ├──+1:     pub path: PathBuf,
│  ├──-1: 
│  ├──30: impl Directory {
│  ├──+1:     pub fn new(path: &Path, links: bool) -> Result<Self, Message> {
│  ├──-1: 
│  ├──41: pub struct File {
│  ├──+1:     pub path: PathBuf,
│  ├──-1: 
│  ├──47: impl File {
│  ├──+1:     pub fn from_pathbuf(path: PathBuf, links: bool) -> Result<Self, Message> {
│  ├──-1: #[cfg_attr(test, derive(PartialEq, Debug))]
│  ├──73: pub struct Match {
│  ├──+1:     pub regexp_id: usize,
│  ├──-1: 
│  ├──79: impl Match {
│  ├──+1:     pub fn new(regexp_id: usize, start: usize, end: usize) -> Self {
│  ├──-1: 
│  ├──104: pub struct Line {
│  ├──+1:     pub content: String,
│  ├──-1: 
│  ├──111: impl Line {
│  ╰──+1:     pub fn new(content: String, mut matches: Vec<Match>, line_num: usize) -> Self {
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
├──matcher.rs: 3
│  ├──-1: 
│  ├──29: struct Matcher {
│  ├──+1:     combined: RegexMatcher,
│  ├──-1: 
│  ├──34: impl Matcher {
│  ├──+1:     fn new(patterns: &[String]) -> Result<Self, Message> {
│  ├──-1: 
│  ├──53: struct MatchSink<'a> {
│  ╰──+1:     lines: Vec<Line>,
├──writer.rs: 19
│  ├──-1: 
│  ├──57: impl HighlightEvent<'_> {
│  ├──+1:     fn priority(&self) -> u8 {
│  ├──-1: 
│  ├──158: pub struct OpenInfo<'a> {
│  ├──+1:     pub path: &'a Path,
│  ├──-1: 
│  ├──171: pub struct WithFilter<'a> {
│  ├──+1:     pub entry: &'a dyn Entry,
│  ├──-1: 
│  ├──176: impl Display for WithFilter<'_> {
│  ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
│  ├──-1: 
│  ├──182: struct PathDisplay {
│  ├──+1:     prefix: Option<Vec<PrefixComponent>>,
│  ├──-1: 
│  ├──193: impl PathDisplay {
│  ├──+1:     fn new(
│  ├──-1: 
│  ├──226: impl Entry for PathDisplay {
│  ├──+1:     fn render(&self, f: &mut fmt::Formatter, filter: &str) -> fmt::Result {
│  ├──-1: 
│  ├──265: impl Display for PathDisplay {
│  ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
│  ├──-1: 
│  ├──323: struct LineDisplay {
│  ├──+1:     prefix: Vec<PrefixComponent>,
│  ├──-1: 
│  ├──334: impl Entry for LineDisplay {
│  ├──+1:     fn render(&self, f: &mut fmt::Formatter, filter: &str) -> fmt::Result {
│  ├──-1: 
│  ├──463: impl Display for LineDisplay {
│  ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
│  ├──-1: 
│  ├──469: struct LongBranchDisplay {
│  ├──+1:     prefix: Vec<PrefixComponent>,
│  ├──-1: 
│  ├──476: impl Entry for LongBranchDisplay {
│  ├──+1:     fn render(&self, f: &mut fmt::Formatter, filter: &str) -> fmt::Result {
│  ├──-1: 
│  ├──527: impl Display for LongBranchDisplay {
│  ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
│  ├──-1: 
│  ├──533: struct OverviewDisplay {
│  ├──+1:     dirs: usize,
│  ├──-1: 
│  ├──542: impl Entry for OverviewDisplay {
│  ├──+1:     fn render(&self, f: &mut fmt::Formatter, _filter: &str) -> fmt::Result {
│  ├──-1: 
│  ├──582: impl Display for OverviewDisplay {
│  ├──+1:     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
│  ├──-1: 
│  ├──604: impl Directory {
│  ├──+1:     fn to_lines(
│  ├──-1: 
│  ├──709: impl File {
│  ╰──+1:     fn to_lines(
├──args.rs: 6
│  ├──-1: 
│  ├──25: impl ValueEnum for OpenStrategy {
│  ├──+1:     fn value_variants<'a>() -> &'a [Self] {
│  ├──-1: #[derive(Clone)]
│  ├──83: pub struct ColorParser;
│  ├──+1: 
│  ├──85: impl clap::builder::TypedValueParser for ColorParser {
│  ├──+1:     type Value = Color;
│  ├──-1: #[derive(Clone)]
│  ├──142: pub struct KeyCodeParser;
│  ├──+1: 
│  ├──144: impl clap::builder::TypedValueParser for KeyCodeParser {
│  ├──+1:     type Value = KeyCode;
│  ├──-1: )]
│  ├──328: pub struct Args {
│  ╰──+1:     #[arg(
├──config.rs: 9
│  ├──-1: #[derive(Clone)]
│  ├──28: pub struct KeyBindings {
│  ├──+1:     pub down: Vec<KeyCode>,
│  ├──-1: #[derive(Clone)]
│  ├──52: pub struct Characters {
│  ├──+1:     pub bl: char,
│  ├──-1: #[derive(Clone)]
│  ├──73: pub struct Colors {
│  ├──+1:     pub file: Color,
│  ├──-1: 
│  ├──85: impl args::Color {
│  ├──+1:     fn get(&self) -> Color {
│  ├──-1: #[derive(Clone)]
│  ├──104: pub struct SearchParams {
│  ├──+1:     pub regexps: Vec<String>,
│  ├──-1: #[derive(Clone)]
│  ├──124: pub struct Config {
│  ├──+1:     pub search: SearchParams,
│  ├──-1: 
│  ├──436: struct NonSearchFields {
│  ├──+1:     selection_file: Option<PathBuf>,
│  ├──-1: 
│  ├──453: impl Config {
│  ├──+1:     pub fn get_styling(matches: &ArgMatches) -> (bool, bool) {
│  ├──-1: 
│  ├──754: impl SearchParams {
│  ╰──+1:     fn to_args(&self) -> Vec<OsString> {
╰──menu.rs: 9
   ├──-1: 
   ├──59: impl ViewAnchor {
   ├──+1:     fn next(&mut self) {
   ├──-1: 
   ├──69: struct DoubleClick {
   ├──+1:     down_row: u16,
   ├──-1: 
   ├──74: impl DoubleClick {
   ├──+1:     fn new() -> Self {
   ├──-1: 
   ├──101: struct Window {
   ├──+1:     first: isize,
   ├──-1: 
   ├──106: impl Window {
   ├──+1:     fn new() -> Self {
   ├──-1: 
   ├──451: struct CurrentResults {
   ├──+1:     lines: Vec<Box<dyn Entry>>,
   ├──-1: 
   ├──455: impl CurrentResults {
   ├──+1:     fn new(matches: Matches, config: Arc<Config>) -> io::Result<Self> {
   ├──-1: 
   ├──463: pub struct Menu<'a, 'b> {
   ├──+1:     in_menu: bool,
   ├──-1: 
   ├──1633: impl OpenStrategy {
   ╰──+1:     fn from(editor: &str) -> Self {
```
</details>

<details>
<summary><code>tgrep Print src/menu.rs --trim --line-number --char-vertical=| --char-horizontal=- --char-top-left=+ --char-top-right=+ --char-bottom-left=+ --char-bottom-right=+ --char-tee=+</code></summary>

```
menu.rs
+--19: style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
+--328: Print(format!(
+--339: Print(format!(
+--351: Print(format!(
+--640: queue!(self.term, Print(WithFilter { entry, filter }))?;
+--643: queue!(self.term, Print(&cfg.chars.ellipsis))?;
+--662: Print(style::style_with(
+--670: queue!(self.term, Print(cfg.chars.selected_indicator.as_str()))?;
+--691: Print(cfg.chars.selected_indicator_clear.as_str()),
+--1045: Print(top),
+--1067: Print(display),
+--1163: Print(line)
+--1183: queue!(self.term, cursor::MoveTo(0, y), Print(msg))?;
+--1398: Print(cfg.chars.selected_indicator_clear.as_str())
```
</details>

<details>
<summary><code>tgrep --files --hidden --glob=!.git</code></summary>

```
treegrep
├──src
│  ├──match_system.rs
│  ├──menu.rs
│  ├──args.rs
│  ├──writer.rs
│  ├──main.rs
│  ├──errors.rs
│  ├──style.rs
│  ├──log.rs
│  ├──config.rs
│  ├──matcher.rs
│  ╰──term.rs
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
│  ├──treegrep.el
│  ╰──treegrep.vim
├──tests
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──context_c1
│  │  ├──links_1
│  │  ├──count
│  │  ├──overview_file
│  │  ├──files_long_branch_expr_1
│  │  ├──overlapping
│  │  ├──file
│  │  ├──max_depth
│  │  ├──files_long_branch_2
│  │  ├──glob_inclusion
│  │  ├──files_long_branch_expr_count_1
│  │  ├──files_with_expr
│  │  ├──files_1
│  │  ├──context_b1
│  │  ├──wide_2
│  │  ├──links_4
│  │  ├──context_a1
│  │  ├──links_3
│  │  ├──files_long_branch_expr_count_2
│  │  ├──links_2
│  │  ├──files_2
│  │  ├──overview_dir
│  │  ├──files_long_branch_expr_2
│  │  ├──line_number
│  │  ├──wide_1
│  │  ├──glob_exclusion
│  │  ├──deep
│  │  ├──no_matches
│  │  ╰──files_long_branch_1
│  ├──utils.rs
│  ├──tests.rs
│  ╰──file_system.rs
├──.gitignore
├──README.md
├──Cargo.lock
├──LICENSE
├──rustfmt.toml
╰──Cargo.toml
```
</details>

<details>
<summary><code>tgrep --files --branch-each=5 --hidden --glob=!.git</code></summary>

```
treegrep
├──src
│  ├──match_system.rs, menu.rs, args.rs, writer.rs, main.rs
│  ├──errors.rs, style.rs, log.rs, config.rs, matcher.rs
│  ╰──term.rs
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
│  ╰──treegrep.el, treegrep.vim
├──tests
│  ├──pool
│  │  ╰──alice_adventures_in_wonderland_by_lewis_carroll.txt
│  ├──targets
│  │  ├──files_1, wide_2, links_4, links_3, links_2
│  │  ├──files_long_branch_expr_2, glob_exclusion, no_matches, files_long_branch_1, context_b1
│  │  ├──context_a1, files_long_branch_expr_count_2, overview_dir, wide_1, files_2
│  │  ├──line_number, deep, context_c1, links_1, count
│  │  ├──overview_file, files_long_branch_expr_1, overlapping, file, max_depth
│  │  ╰──files_long_branch_2, glob_inclusion, files_long_branch_expr_count_1, files_with_expr
│  ╰──utils.rs, tests.rs, file_system.rs
├──.gitignore, README.md, Cargo.lock, LICENSE, rustfmt.toml
╰──Cargo.toml
```
</details>

### *--help*
```
tgrep 2.0.0

by Timothy Cronin

home page: https://github.com/4imothy/treegrep

regex pattern matcher that displays results in a tree structure with an interface to jump to matched text

tgrep [OPTIONS] <POSITIONAL_REGEXP|--regexp <>|--files|--completions <>|--menu|--repeat> [POSITIONAL_PATH]

arguments:
  [POSITIONAL_REGEXP]
          a regex expression to search for

  [POSITIONAL_PATH]
          the path to search, if not provided, search the current directory

options:
  -e, --regexp <>
          a regex expression to search for (saved for repeat)

  -p, --path <>
          the path to search, if not provided, search the current directory (saved for repeat)

  -s, --select
          results are shown in a selection interface for opening

  -m, --menu
          open a search and selection interface

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

  -d, --max-depth <>
          the max depth to search (saved for repeat)

  -C, --context <>
          number of lines to show before and after each match

  -B, --before-context <>
          number of lines to show before each match (saved for repeat)

  -A, --after-context <>
          number of lines to show after each match (saved for repeat)

      --live
          trigger search on every keystroke in the menu

      --max-length <>
          set the max length for a matched line (saved for repeat)

      --no-ignore
          disable ignore files (saved for repeat)

      --trim
          trim whitespace at the beginning of lines (saved for repeat)

      --threads <>
          set the number of threads to use

      --editor <>
          command used to open selections

      --auto-open
          if there is only one match, open it in the configured editor

      --open-like <>
          command line syntax for opening a file at a line
          
          [possible values: vi, hx, code, jed, default]

      --completions <>
          generate completions for given shell
          
          [possible values: bash, elvish, fish, powershell, zsh]

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
