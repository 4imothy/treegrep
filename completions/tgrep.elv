
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
            cand -e 'specify the regex expression'
            cand --regexp 'specify the regex expression'
            cand -t 'specify the search target. If none provided, search the current directory.'
            cand --target 'specify the search target. If none provided, search the current directory.'
            cand --max-depth 'the max depth to search'
            cand --threads 'set the appropriate number of threads to use'
            cand --prefix-len 'number of characters to show before a match'
            cand --max-length 'set the max length for a matched line'
            cand -s 'executable to do the searching'
            cand --searcher 'executable to do the searching'
            cand --box-chars 'style of box characters to use'
            cand --glob 'rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic'
            cand -c 'display number of files matched in directory and number of lines matched in a file if present'
            cand --count 'display number of files matched in directory and number of lines matched in a file if present'
            cand -. 'search hidden files'
            cand --hidden 'search hidden files'
            cand -n 'show line number of match if present'
            cand --line-number 'show line number of match if present'
            cand -m 'open results in a menu to be edited with $EDITOR navigate through the menu using the following commands:  - move up/down: k/j, p/n, up arrow/down arrow  - move up/down with a bigger jump: K/J, P/N  - move up/down paths: {/}, [/]  - move to the start/end: g/G, </>, home/end  - move up/down a page: b/f, pageup/pagedown  - quit: q, ctrl + c'
            cand --menu 'open results in a menu to be edited with $EDITOR navigate through the menu using the following commands:  - move up/down: k/j, p/n, up arrow/down arrow  - move up/down with a bigger jump: K/J, P/N  - move up/down paths: {/}, [/]  - move to the start/end: g/G, </>, home/end  - move up/down a page: b/f, pageup/pagedown  - quit: q, ctrl + c'
            cand -f 'show the paths that have matches'
            cand --files 'show the paths that have matches'
            cand --links 'show linked paths for symbolic links'
            cand --trim 'trim whitespace at the beginning of lines'
            cand --pcre2 'enable PCRE2 if the searcher supports it'
            cand --no-ignore 'don''t use ignore files'
            cand --no-color 'don''t use colors if present'
            cand -l 'display the files that would be search in tree format'
            cand --tree 'display the files that would be search in tree format'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
