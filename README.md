## treegrep

treegrep is a frontend for existing pattern matchers or a standalone pattern matcher which presents results in a tree format

### Currently Suuported Backends
- *[ripgrep](https://github.com/BurntSushi/ripgrep)*
- *[grep](https://en.wikipedia.org/wiki/Grep)*

### *--help* Output
```
treegrep
by Timothy Cronin

treegrep is a pattern matcher frontend or backend
    which displays matches in a tree and can
    present results in a menu to be opened
tgrep [OPTIONS] <regex expression-positional|--regexp <regex expression>> [target-positional]

Arguments:
  [regex expression-positional]  specify the regex expression
  [target-positional]            specify the search target. If none provided, search the current directory.

Options:
  -e, --regexp <regex expression>  specify the regex expression
  -t, --target <target>            specify the search target. If none provided, search the current directory.
  -c, --count                      display number of files matched in directory and number of lines matched in a file if present
  -., --hidden                     search hidden files
  -n, --line-number                show line number of match if present
  -m, --menu                       open results in a menu to be opened with $EDITOR
  -f, --files                      show paths that have matches
      --links                      show linked paths for symbolic links
      --trim-left                  trim whitespace at beginning of lines
      --pcre2                      enable pcre2 if the searcher supports it
      --no-ignore                  don't use ignore files
      --max-depth <max-depth>      the max depth to search
      --threads <threads>          set appropriate number of threads to use
      --colors <colors>            set whether to color output [possible values: always, never]
  -s, --searcher <searcher>        executable to do the searching, currently supports rg, grep and tgrep
  -h, --help                       Print help
```