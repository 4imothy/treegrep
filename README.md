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

home page: https://github.com/4imothy/treegrep

pattern matcher frontend or backend which displays results in a tree

tgrep [OPTIONS] [positional regexp] [positional target]
Arguments:
  [positional regexp]  the regex expression
  [positional target]  the path to search. If not provided, search the current directory.

Options:
  -e, --regexp <>            the regex expression
  -p, --path <>              the path to search. If not provided, search the current directory.
      --completions <shell>  generate completions for given shell [possible values: bash, elvish, fish, powershell, zsh]
  -t, --tree                 display the files that would be search in tree format
      --glob <>              rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic
  -s, --searcher <>          executable to do the searching [possible values: rg, tgrep]
      --threads <>           set the appropriate number of threads to use
  -., --hidden               search hidden files
  -n, --line-number          show line number of match
  -f, --files                don't show matched contents
      --links                search linked paths
      --trim                 trim whitespace at the beginning of lines
      --pcre2                enable PCRE2
      --no-ignore            don't use ignore files
  -c, --count                display number of files matched in directory and number of lines matched in a file
      --no-color             don't use colors
      --no-bold              don't bold anything
      --max-depth <>         the max depth to search
      --prefix-len <>        number of characters to show before a match [default: 3]
      --max-length <>        set the max length for a matched line [default: 1000]
      --long-branch-each <>  number of files to print on each branch [default: 5]
      --char-style <>        style of characters to use [possible values: ascii, single, double, heavy, rounded, none]
      --long-branch          multiple files from the same directory are shown on the same branch
  -m, --menu                 open results in a menu to be edited with $EDITOR
                             navigate through the menu using the following commands:
                              - move up/down: k/j, p/n, up arrow/down arrow
                              - move up/down with a bigger jump: K/J, P/N
                              - move up/down paths: {/}, [/]
                              - move to the start/end: g/G, </>, home/end
                              - move up/down a page: b/f, pageup/pagedown
                              - center cursor: z/l
                              - quit: q, ctrl + c
  -h, --help                 Print help
  -V, --version              Print version

any of the above can be set using the TREEGREP_DEFAULT_OPTS environment variable
```
