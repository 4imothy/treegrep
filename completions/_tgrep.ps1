
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
            [CompletionResult]::new('-e', '-e', [CompletionResultType]::ParameterName, 'the regex expression')
            [CompletionResult]::new('--regexp', '--regexp', [CompletionResultType]::ParameterName, 'the regex expression')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'the path to search, if not provided, search the current directory')
            [CompletionResult]::new('--path', '--path', [CompletionResultType]::ParameterName, 'the path to search, if not provided, search the current directory')
            [CompletionResult]::new('--glob', '--glob', [CompletionResultType]::ParameterName, 'rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'executable to do the searching')
            [CompletionResult]::new('--searcher', '--searcher', [CompletionResultType]::ParameterName, 'executable to do the searching')
            [CompletionResult]::new('--char-style', '--char-style', [CompletionResultType]::ParameterName, 'style of characters to use')
            [CompletionResult]::new('--editor', '--editor', [CompletionResultType]::ParameterName, 'command used to open selections')
            [CompletionResult]::new('--open-like', '--open-like', [CompletionResultType]::ParameterName, 'command line syntax for opening a file at a line')
            [CompletionResult]::new('--threads', '--threads', [CompletionResultType]::ParameterName, 'set the appropriate number of threads to use')
            [CompletionResult]::new('--max-depth', '--max-depth', [CompletionResultType]::ParameterName, 'the max depth to search')
            [CompletionResult]::new('--prefix-len', '--prefix-len', [CompletionResultType]::ParameterName, 'number of characters to show before a match')
            [CompletionResult]::new('--max-length', '--max-length', [CompletionResultType]::ParameterName, 'set the max length for a matched line')
            [CompletionResult]::new('--long-branch-each', '--long-branch-each', [CompletionResultType]::ParameterName, 'number of files to print on each branch')
            [CompletionResult]::new('--long-branch', '--long-branch', [CompletionResultType]::ParameterName, 'multiple files from the same directory are shown on the same branch')
            [CompletionResult]::new('-.', '-.', [CompletionResultType]::ParameterName, 'search hidden files')
            [CompletionResult]::new('--hidden', '--hidden', [CompletionResultType]::ParameterName, 'search hidden files')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'show line number of match')
            [CompletionResult]::new('--line-number', '--line-number', [CompletionResultType]::ParameterName, 'show line number of match')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'if a pattern is given hide matched content, otherwise show the files that would be searched')
            [CompletionResult]::new('--files', '--files', [CompletionResultType]::ParameterName, 'if a pattern is given hide matched content, otherwise show the files that would be searched')
            [CompletionResult]::new('--links', '--links', [CompletionResultType]::ParameterName, 'search linked paths')
            [CompletionResult]::new('--trim', '--trim', [CompletionResultType]::ParameterName, 'trim whitespace at the beginning of lines')
            [CompletionResult]::new('--pcre2', '--pcre2', [CompletionResultType]::ParameterName, 'enable PCRE2')
            [CompletionResult]::new('--no-ignore', '--no-ignore', [CompletionResultType]::ParameterName, 'don''t use ignore files')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'display number of files matched in directory and number of lines matched in a file')
            [CompletionResult]::new('--count', '--count', [CompletionResultType]::ParameterName, 'display number of files matched in directory and number of lines matched in a file')
            [CompletionResult]::new('--no-color', '--no-color', [CompletionResultType]::ParameterName, 'don''t use colors')
            [CompletionResult]::new('--no-bold', '--no-bold', [CompletionResultType]::ParameterName, 'don''t bold anything')
            [CompletionResult]::new('--overview', '--overview', [CompletionResultType]::ParameterName, 'conclude results with an overview')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'show results in a menu to be jumped to navigate through with the following commands:  - move up/down: k/j, p/n, up arrow/down arrow  - move up/down with a bigger jump: K/J, P/N  - move up/down paths: {/}, [/]  - move to the start/end: g/G, </>, home/end  - move up/down a page: b/f, pageup/pagedown  - center cursor: z/l  - quit: q, ctrl + c')
            [CompletionResult]::new('--menu', '--menu', [CompletionResultType]::ParameterName, 'show results in a menu to be jumped to navigate through with the following commands:  - move up/down: k/j, p/n, up arrow/down arrow  - move up/down with a bigger jump: K/J, P/N  - move up/down paths: {/}, [/]  - move to the start/end: g/G, </>, home/end  - move up/down a page: b/f, pageup/pagedown  - center cursor: z/l  - quit: q, ctrl + c')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'generate completions for given shell')
            break
        }
        'tgrep;completions' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
