complete -c tgrep -s e -l regexp -d 'the regex expression' -r
complete -c tgrep -s p -l path -d 'the path to search, if not provided, search the current directory' -r -F
complete -c tgrep -l completions -d 'generate completions for given shell' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c tgrep -l glob -d 'rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic' -r
complete -c tgrep -s s -l searcher -d 'executable to do the searching' -r -f -a "rg\t''
tgrep\t''"
complete -c tgrep -l threads -d 'set the appropriate number of threads to use' -r
complete -c tgrep -l max-depth -d 'the max depth to search' -r
complete -c tgrep -l prefix-len -d 'number of characters to show before a match' -r
complete -c tgrep -l max-length -d 'set the max length for a matched line' -r
complete -c tgrep -l long-branch-each -d 'number of files to print on each branch' -r
complete -c tgrep -l char-style -d 'style of characters to use' -r -f -a "ascii\t''
single\t''
double\t''
heavy\t''
rounded\t''
none\t''"
complete -c tgrep -s . -l hidden -d 'search hidden files'
complete -c tgrep -s n -l line-number -d 'show line number of match'
complete -c tgrep -s f -l files -d 'if a pattern is given hide matched content, otherwise show the files that would be searched'
complete -c tgrep -l links -d 'search linked paths'
complete -c tgrep -l trim -d 'trim whitespace at the beginning of lines'
complete -c tgrep -l pcre2 -d 'enable PCRE2'
complete -c tgrep -l no-ignore -d 'don\'t use ignore files'
complete -c tgrep -s c -l count -d 'display number of files matched in directory and number of lines matched in a file'
complete -c tgrep -l no-color -d 'don\'t use colors'
complete -c tgrep -l no-bold -d 'don\'t bold anything'
complete -c tgrep -l long-branch -d 'multiple files from the same directory are shown on the same branch'
complete -c tgrep -s m -l menu -d 'open results in a menu to be edited with $EDITOR navigate through the menu using the following commands:  - move up/down: k/j, p/n, up arrow/down arrow  - move up/down with a bigger jump: K/J, P/N  - move up/down paths: {/}, [/]  - move to the start/end: g/G, </>, home/end  - move up/down a page: b/f, pageup/pagedown  - center cursor: z/l  - quit: q, ctrl + c'
complete -c tgrep -s h -l help -d 'Print help'
complete -c tgrep -s V -l version -d 'Print version'
