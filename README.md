## treegrep

treegrep is a frontend for existing pattern matchers or a standalone pattern matcher which presents results in a tree format

### Currently Suuported Backends
- *[ripgrep](https://github.com/BurntSushi/ripgrep)*
- *[treegrep](https://github.com/4imothy/treegrep)*

### To Install

#### Cargo
```
cargo install treegrep
```

#### Releases
Download from [releases](https://github.com/4imothy/treegrep/releases/)

#### Manual
```
git clone https://github.com/4imothy/treegrep
cargo build
```

### Links
[crates](https://crates.io/crates/treegrep) | [github](https://github.com/4imothy/treegrep) | [aur](https://aur.archlinux.org/packages/treegrep-bin)


https://github.com/4imothy/treegrep/assets/40186632/9c85c309-df78-4996-8127-ee5ad9f91ec3


### *--help* Output
```
treegrep 0.1.1
by Timothy Cronin

A pattern matcher frontend or backend which displays results in a tree

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
  -m, --menu                       open results in a menu to be opened with $EDITOR, move with j/k, n/p, up/down
  -f, --files                      show the paths that have matches
      --links                      show linked paths for symbolic links
      --trim                       trim whitespace at beginning of lines
      --pcre2                      enable pcre2 if the searcher supports it
      --no-ignore                  don't use ignore files
      --max-depth <max-depth>      the max depth to search
      --threads <threads>          set appropriate number of threads to use
      --color <color>              set whether to color output [possible values: always, never]
  -s, --searcher <searcher>        executable to do the searching, currently supports rg  and tgrep
  -h, --help                       Print help
  -V, --version                    Print version
```
