
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
            [CompletionResult]::new('--threads', 'threads', [CompletionResultType]::ParameterName, 'set appropriate number of threads to use')
            [CompletionResult]::new('--colors', 'colors', [CompletionResultType]::ParameterName, 'set whether to color output')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'executable to do the searching, currently supports rg, grep and tgrep')
            [CompletionResult]::new('--searcher', 'searcher', [CompletionResultType]::ParameterName, 'executable to do the searching, currently supports rg, grep and tgrep')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'display number of files matched in directory and number of lines matched in a file if present')
            [CompletionResult]::new('--count', 'count', [CompletionResultType]::ParameterName, 'display number of files matched in directory and number of lines matched in a file if present')
            [CompletionResult]::new('-.', '.', [CompletionResultType]::ParameterName, 'search hidden files')
            [CompletionResult]::new('--hidden', 'hidden', [CompletionResultType]::ParameterName, 'search hidden files')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'show line number of match if present')
            [CompletionResult]::new('--line-number', 'line-number', [CompletionResultType]::ParameterName, 'show line number of match if present')
            [CompletionResult]::new('-m', 'm', [CompletionResultType]::ParameterName, 'open results in a menu to be opened with $EDITOR')
            [CompletionResult]::new('--menu', 'menu', [CompletionResultType]::ParameterName, 'open results in a menu to be opened with $EDITOR')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'show paths that have matches')
            [CompletionResult]::new('--files', 'files', [CompletionResultType]::ParameterName, 'show paths that have matches')
            [CompletionResult]::new('--links', 'links', [CompletionResultType]::ParameterName, 'show linked paths for symbolic links')
            [CompletionResult]::new('--trim-left', 'trim-left', [CompletionResultType]::ParameterName, 'trim whitespace at beginning of lines')
            [CompletionResult]::new('--pcre2', 'pcre2', [CompletionResultType]::ParameterName, 'enable pcre2 if the searcher supports it')
            [CompletionResult]::new('--no-ignore', 'no-ignore', [CompletionResultType]::ParameterName, 'don''t use ignore files')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
