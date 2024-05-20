
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'tgrep' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'tgrep'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'tgrep' {
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'specify the regex expression')
            [CompletionResult]::new('--regexp', 'regexp', [CompletionResultType]::ParameterName, 'specify the regex expression')
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'specify the search target. If none provided, search the current directory.')
            [CompletionResult]::new('--target', 'target', [CompletionResultType]::ParameterName, 'specify the search target. If none provided, search the current directory.')
            [CompletionResult]::new('--max-depth', 'max-depth', [CompletionResultType]::ParameterName, 'the max depth to search')
            [CompletionResult]::new('--threads', 'threads', [CompletionResultType]::ParameterName, 'set the appropriate number of threads to use')
            [CompletionResult]::new('--max-length', 'max-length', [CompletionResultType]::ParameterName, 'set the max length for a matched line')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'executable to do the searching')
            [CompletionResult]::new('--searcher', 'searcher', [CompletionResultType]::ParameterName, 'executable to do the searching')
            [CompletionResult]::new('--glob', 'glob', [CompletionResultType]::ParameterName, 'rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'display number of files matched in directory and number of lines matched in a file if present')
            [CompletionResult]::new('--count', 'count', [CompletionResultType]::ParameterName, 'display number of files matched in directory and number of lines matched in a file if present')
            [CompletionResult]::new('-.', '.', [CompletionResultType]::ParameterName, 'search hidden files')
            [CompletionResult]::new('--hidden', 'hidden', [CompletionResultType]::ParameterName, 'search hidden files')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'show line number of match if present')
            [CompletionResult]::new('--line-number', 'line-number', [CompletionResultType]::ParameterName, 'show line number of match if present')
            [CompletionResult]::new('-m', 'm', [CompletionResultType]::ParameterName, 'open results in a menu to be edited with $EDITOR navigate through the menu using the following commands: 	- move up/down: k/j, p/n, up arrow/down arrow 	- move up/down with a bigger jump: K/J, P/N 	- move up/down paths: {/}, [/] 	- move to the start/end: g/G, </>, home/end 	- move up/down a page: ctrl + b/ctrl + f, pageup/pagedown')
            [CompletionResult]::new('--menu', 'menu', [CompletionResultType]::ParameterName, 'open results in a menu to be edited with $EDITOR navigate through the menu using the following commands: 	- move up/down: k/j, p/n, up arrow/down arrow 	- move up/down with a bigger jump: K/J, P/N 	- move up/down paths: {/}, [/] 	- move to the start/end: g/G, </>, home/end 	- move up/down a page: ctrl + b/ctrl + f, pageup/pagedown')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'show the paths that have matches')
            [CompletionResult]::new('--files', 'files', [CompletionResultType]::ParameterName, 'show the paths that have matches')
            [CompletionResult]::new('--links', 'links', [CompletionResultType]::ParameterName, 'show linked paths for symbolic links')
            [CompletionResult]::new('--trim', 'trim', [CompletionResultType]::ParameterName, 'trim whitespace at the beginning of lines')
            [CompletionResult]::new('--pcre2', 'pcre2', [CompletionResultType]::ParameterName, 'enable PCRE2 if the searcher supports it')
            [CompletionResult]::new('--no-ignore', 'no-ignore', [CompletionResultType]::ParameterName, 'don''t use ignore files')
            [CompletionResult]::new('--no-color', 'no-color', [CompletionResultType]::ParameterName, 'don''t use colors if present')
            [CompletionResult]::new('-l', 'l', [CompletionResultType]::ParameterName, 'display the files that would be search in tree format')
            [CompletionResult]::new('--tree', 'tree', [CompletionResultType]::ParameterName, 'display the files that would be search in tree format')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
