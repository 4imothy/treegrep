## treegrep

treegrep is a frontend for existing pattern matchers or a standalone pattern matcher which presents results in a tree format

[![tests](https://github.com/4imothy/treegrep/actions/workflows/ci.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)
[![release](https://github.com/4imothy/treegrep/actions/workflows/cr.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)

### Currently Suuported Backends
- *[ripgrep](https://github.com/BurntSushi/ripgrep)*
- *[treegrep](https://github.com/4imothy/treegrep)*

### To Install

#### Cargo
```
cargo install treegrep
```

#### NetBSD
```
pkgin install treegrep
```

#### Releases
Download from [releases](https://github.com/4imothy/treegrep/releases/)

#### Manual
```
git clone https://github.com/4imothy/treegrep
cd treegrep
cargo build --release
```

### Links
[crates.io](https://crates.io/crates/treegrep) | [GitHub](https://github.com/4imothy/treegrep) | [AUR](https://aur.archlinux.org/packages/treegrep-bin) | [NetBSD](https://mail-index.netbsd.org/pkgsrc-changes/2024/01/11/msg290674.html)


https://github.com/4imothy/treegrep/assets/40186632/9c85c309-df78-4996-8127-ee5ad9f91ec3


### *--help* Output
```
treegrep 0.1.3
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
  -m, --menu                       open results in a menu to be edited with $EDITOR
                                   navigate through the menu using the following commands:
                                   - move up/down: k/j, p/n, up arrow/down arrow
                                   - move up/down with a bigger jump: K/J, P/N
                                   - move up/down paths: {/}, [/]
                                   - move to the start/end: g/G, </>, home/end
                                   - move up/down a page: ctrl + b/ctrl + f, pageup/pagedown
  -f, --files                      show the paths that have matches
      --links                      show linked paths for symbolic links
      --trim                       trim whitespace at the beginning of lines
      --pcre2                      enable PCRE2 if the searcher supports it
      --no-ignore                  don't use ignore files
      --max-depth <max-depth>      the max depth to search
      --threads <threads>          set the appropriate number of threads to use
      --max-length <max-length>    set the max length for a matched line
      --color <color>              set whether to color output [possible values: always, never]
  -s, --searcher <searcher>        executable to do the searching, currently supports rg  and tgrep
  -h, --help                       Print help
  -V, --version                    Print version
```
