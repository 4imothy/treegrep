## treegrep

treegrep is a frontend for existing pattern matchers or a standalone pattern matcher which presents results in a tree format

[![tests](https://github.com/4imothy/treegrep/actions/workflows/ci.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)
[![release](https://github.com/4imothy/treegrep/actions/workflows/cr.yml/badge.svg)](https://github.com/4imothy/treegrep/actions)


https://github.com/4imothy/treegrep/assets/40186632/9c85c309-df78-4996-8127-ee5ad9f91ec3


### Currently Supported Backends
- *[ripgrep](https://github.com/BurntSushi/ripgrep)*
- *[treegrep](https://github.com/4imothy/treegrep)*

### Links
[crates.io](https://crates.io/crates/treegrep) | [GitHub](https://github.com/4imothy/treegrep) | [AUR](https://aur.archlinux.org/packages/treegrep-bin) | [NetBSD](https://pkgsrc.se/sysutils/treegrep)

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

### *--help* Output
```
treegrep 0.1.4
by Timothy Cronin

A pattern matcher frontend or backend which displays results in a tree

tgrep [OPTIONS] [positional regexp] [positional target]

Arguments:
  [positional regexp]  specify the regex expression
  [positional target]  specify the search target. If none provided, search the current directory.

Options:
  -e, --regexp <>      specify the regex expression
  -t, --target <>      specify the search target. If none provided, search the current directory.
  -c, --count          display number of files matched in directory and number of lines matched in a file if present
  -., --hidden         search hidden files
  -n, --line-number    show line number of match if present
  -m, --menu           open results in a menu to be edited with $EDITOR
                       navigate through the menu using the following commands:
                        - move up/down: k/j, p/n, up arrow/down arrow
                        - move up/down with a bigger jump: K/J, P/N
                        - move up/down paths: {/}, [/]
                        - move to the start/end: g/G, </>, home/end
                        - move up/down a page: b/f, pageup/pagedown
                        - quit: q, ctrl + c
  -f, --files          show the paths that have matches
      --links          show linked paths for symbolic links
      --trim           trim whitespace at the beginning of lines
      --pcre2          enable PCRE2 if the searcher supports it
      --no-ignore      don't use ignore files
      --no-color       don't use colors if present
      --max-depth <>   the max depth to search
      --threads <>     set the appropriate number of threads to use
      --max-length <>  set the max length for a matched line
  -s, --searcher <>    executable to do the searching [possible values: rg, tgrep]
  -l, --tree           display the files that would be search in tree format
      --glob <>        rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic
  -h, --help           Print help
  -V, --version        Print version

Any of the above can be set using the TREEGREP_DEFAULT_OPTS environment variable
```
