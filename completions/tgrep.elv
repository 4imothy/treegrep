
use builtin;
use str;

set edit:completion:arg-completer[tgrep] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'tgrep'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'tgrep'= {
            cand -e 'the regex expression'
            cand --regexp 'the regex expression'
            cand -p 'the path to search, if not provided, search the current directory'
            cand --path 'the path to search, if not provided, search the current directory'
            cand --glob 'rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic'
            cand -s 'executable to do the searching'
            cand --searcher 'executable to do the searching'
            cand --char-style 'style of characters to use'
            cand --editor 'command used to open selections'
            cand --open-like 'command line syntax for opening a file at a line'
            cand --completions 'generate completions for given shell'
            cand --threads 'set the appropriate number of threads to use'
            cand --max-depth 'the max depth to search'
            cand --prefix-len 'number of characters to show before a match'
            cand --max-length 'set the max length for a matched line'
            cand --long-branch-each 'number of files to print on each branch'
            cand --long-branch 'multiple files from the same directory are shown on the same branch'
            cand -. 'search hidden files'
            cand --hidden 'search hidden files'
            cand -n 'show line number of match'
            cand --line-number 'show line number of match'
            cand -f 'if a pattern is given hide matched content, otherwise show the files that would be searched'
            cand --files 'if a pattern is given hide matched content, otherwise show the files that would be searched'
            cand --links 'search linked paths'
            cand --trim 'trim whitespace at the beginning of lines'
            cand --pcre2 'enable PCRE2'
            cand --no-ignore 'don''t use ignore files'
            cand -c 'display number of files matched in directory and number of lines matched in a file'
            cand --count 'display number of files matched in directory and number of lines matched in a file'
            cand --no-color 'don''t use colors'
            cand --no-bold 'don''t bold anything'
            cand --overview 'conclude results with an overview'
            cand -m 'show results in a menu to be jumped to, press h in menu for help'
            cand --menu 'show results in a menu to be jumped to, press h in menu for help'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
