complete -c tgrep -s e -l regexp -d 'specify the regex expression' -r
complete -c tgrep -s t -l target -d 'specify the search target. If none provided, search the current directory.' -r -F
complete -c tgrep -l max-depth -d 'the max depth to search' -r
complete -c tgrep -l threads -d 'set appropriate number of threads to use' -r
complete -c tgrep -l max-length -d 'set the max length for a matched line' -r
complete -c tgrep -l color -d 'set whether to color output' -r -f -a "{always	'',never	''}"
complete -c tgrep -s s -l searcher -d 'executable to do the searching, currently supports rg  and tgrep' -r
complete -c tgrep -s c -l count -d 'display number of files matched in directory and number of lines matched in a file if present'
complete -c tgrep -s . -l hidden -d 'search hidden files'
complete -c tgrep -s n -l line-number -d 'show line number of match if present'
complete -c tgrep -s m -l menu -d 'open results in a menu to be opened with $EDITOR, move with j/k, n/p, up/down'
complete -c tgrep -s f -l files -d 'show the paths that have matches'
complete -c tgrep -l links -d 'show linked paths for symbolic links'
complete -c tgrep -l trim -d 'trim whitespace at beginning of lines'
complete -c tgrep -l pcre2 -d 'enable pcre2 if the searcher supports it'
complete -c tgrep -l no-ignore -d 'don\'t use ignore files'
complete -c tgrep -s h -l help -d 'Print help'
complete -c tgrep -s V -l version -d 'Print version'
