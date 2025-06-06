_tgrep() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="tgrep"
                ;;
            tgrep,completions)
                cmd="tgrep__completions"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        tgrep)
            opts="-e -p -s -. -n -f -c -m -h -V --regexp --path --glob --searcher --char-style --editor --open-like --long-branch --hidden --line-number --files --links --trim --pcre2 --no-ignore --count --no-color --no-bold --overview --menu --threads --max-depth --prefix-len --max-length --long-branch-each --help --version [positional regexp] [positional target] completions"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --regexp)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -e)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --glob)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --searcher)
                    COMPREPLY=($(compgen -W "rg tgrep" -- "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "rg tgrep" -- "${cur}"))
                    return 0
                    ;;
                --char-style)
                    COMPREPLY=($(compgen -W "ascii single double heavy rounded none" -- "${cur}"))
                    return 0
                    ;;
                --editor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --open-like)
                    COMPREPLY=($(compgen -W "vi hx code jed default" -- "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-depth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --prefix-len)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-length)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --long-branch-each)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        tgrep__completions)
            opts="-h --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _tgrep -o nosort -o bashdefault -o default tgrep
else
    complete -F _tgrep -o bashdefault -o default tgrep
fi
