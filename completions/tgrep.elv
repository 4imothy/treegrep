
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
            cand -p 'the path to search. If not provided, search the current directory.'
            cand --path 'the path to search. If not provided, search the current directory.'
            cand --completions 'generate completions for given shell'
            cand --glob 'rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic'
            cand -s 'executable to do the searching'
            cand --searcher 'executable to do the searching'
            cand --threads 'set the appropriate number of threads to use'
            cand --max-depth 'the max depth to search'
            cand --prefix-len 'number of characters to show before a match'
            cand --max-length 'set the max length for a matched line'
            cand --char-style 'style of characters to use'
            cand -t 'display the files that would be search in tree format'
            cand --tree 'display the files that would be search in tree format'
            cand -. 'search hidden files'
            cand --hidden 'search hidden files'
            cand -n 'show line number of match'
            cand --line-number 'show line number of match'
            cand -f 'don''t show matched contents'
            cand --files 'don''t show matched contents'
            cand --links 'show linked paths for symbolic links'
            cand --trim 'trim whitespace at the beginning of lines'
            cand --pcre2 'enable PCRE2 if the searcher supports it'
            cand --no-ignore 'don''t use ignore files'
            cand -c 'display number of files matched in directory and number of lines matched in a file'
            cand --count 'display number of files matched in directory and number of lines matched in a file'
            cand --no-color 'don''t use colors'
            cand --no-bold 'don''t bold anything'
            cand --long-branch 'multiple files from the same directory are shown on the same branch'
            cand -m 'open results in a menu to be edited with $EDITOR navigate through the menu using the following commands:  - move up/down: k/j, p/n, up arrow/down arrow  - move up/down with a bigger jump: K/J, P/N  - move up/down paths: {/}, [/]  - move to the start/end: g/G, </>, home/end  - move up/down a page: b/f, pageup/pagedown  - center cursor: z/l  - quit: q, ctrl + c'
            cand --menu 'open results in a menu to be edited with $EDITOR navigate through the menu using the following commands:  - move up/down: k/j, p/n, up arrow/down arrow  - move up/down with a bigger jump: K/J, P/N  - move up/down paths: {/}, [/]  - move to the start/end: g/G, </>, home/end  - move up/down a page: b/f, pageup/pagedown  - center cursor: z/l  - quit: q, ctrl + c'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
