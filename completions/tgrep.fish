# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_tgrep_global_optspecs
	string join \n e/regexp= p/path= glob= s/searcher= char-style= editor= open-like= long-branch ./hidden n/line-number f/files links trim pcre2 no-ignore c/count no-color no-bold overview m/menu threads= max-depth= prefix-len= max-length= long-branch-each= h/help V/version
end

function __fish_tgrep_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_tgrep_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_tgrep_using_subcommand
	set -l cmd (__fish_tgrep_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c tgrep -n "__fish_tgrep_needs_command" -s e -l regexp -d 'the regex expression' -r
complete -c tgrep -n "__fish_tgrep_needs_command" -s p -l path -d 'the path to search, if not provided, search the current directory' -r -F
complete -c tgrep -n "__fish_tgrep_needs_command" -l glob -d 'rules match .gitignore globs, but ! has inverted meaning, overrides other ignore logic' -r
complete -c tgrep -n "__fish_tgrep_needs_command" -s s -l searcher -d 'executable to do the searching' -r -f -a "rg\t''
tgrep\t''"
complete -c tgrep -n "__fish_tgrep_needs_command" -l char-style -d 'style of characters to use' -r -f -a "ascii\t''
single\t''
double\t''
heavy\t''
rounded\t''
none\t''"
complete -c tgrep -n "__fish_tgrep_needs_command" -l editor -d 'command used to open selections' -r
complete -c tgrep -n "__fish_tgrep_needs_command" -l open-like -d 'command line syntax for opening a file at a line' -r -f -a "vi\t''
hx\t''
code\t''
jed\t''
default\t''"
complete -c tgrep -n "__fish_tgrep_needs_command" -l threads -d 'set the appropriate number of threads to use' -r
complete -c tgrep -n "__fish_tgrep_needs_command" -l max-depth -d 'the max depth to search' -r
complete -c tgrep -n "__fish_tgrep_needs_command" -l prefix-len -d 'number of characters to show before a match' -r
complete -c tgrep -n "__fish_tgrep_needs_command" -l max-length -d 'set the max length for a matched line' -r
complete -c tgrep -n "__fish_tgrep_needs_command" -l long-branch-each -d 'number of files to print on each branch' -r
complete -c tgrep -n "__fish_tgrep_needs_command" -l long-branch -d 'multiple files from the same directory are shown on the same branch'
complete -c tgrep -n "__fish_tgrep_needs_command" -s . -l hidden -d 'search hidden files'
complete -c tgrep -n "__fish_tgrep_needs_command" -s n -l line-number -d 'show line number of match'
complete -c tgrep -n "__fish_tgrep_needs_command" -s f -l files -d 'if a pattern is given hide matched content, otherwise show the files that would be searched'
complete -c tgrep -n "__fish_tgrep_needs_command" -l links -d 'search linked paths'
complete -c tgrep -n "__fish_tgrep_needs_command" -l trim -d 'trim whitespace at the beginning of lines'
complete -c tgrep -n "__fish_tgrep_needs_command" -l pcre2 -d 'enable PCRE2'
complete -c tgrep -n "__fish_tgrep_needs_command" -l no-ignore -d 'don\'t use ignore files'
complete -c tgrep -n "__fish_tgrep_needs_command" -s c -l count -d 'display number of files matched in directory and number of lines matched in a file'
complete -c tgrep -n "__fish_tgrep_needs_command" -l no-color -d 'don\'t use colors'
complete -c tgrep -n "__fish_tgrep_needs_command" -l no-bold -d 'don\'t bold anything'
complete -c tgrep -n "__fish_tgrep_needs_command" -l overview -d 'conclude results with an overview'
complete -c tgrep -n "__fish_tgrep_needs_command" -s m -l menu -d 'show results in a menu to be jumped to navigate through with the following commands:  - move up/down: k/j, p/n, up arrow/down arrow  - move up/down with a bigger jump: K/J, P/N  - move up/down paths: {/}, [/]  - move to the start/end: g/G, </>, home/end  - move up/down a page: b/f, pageup/pagedown  - center cursor: z/l  - quit: q, ctrl + c'
complete -c tgrep -n "__fish_tgrep_needs_command" -s h -l help -d 'Print help'
complete -c tgrep -n "__fish_tgrep_needs_command" -s V -l version -d 'Print version'
complete -c tgrep -n "__fish_tgrep_needs_command" -a "completions" -d 'generate completions for given shell'
complete -c tgrep -n "__fish_tgrep_using_subcommand completions" -s h -l help -d 'Print help'
