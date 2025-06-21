_tgrep() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="tgrep"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        tgrep)
            opts="-e -p -. -n -f -c -s -h -V --regexp --path --glob --searcher --char-style --editor --open-like --long-branch --completions --plugin-support --selection-file --repeat-file --hidden --repeat --line-number --files --links --no-ignore --count --no-color --no-bold --overview --select --menu --trim --pcre2 --threads --max-depth --prefix-len --max-length --long-branch-each --help --version [positional regexp] [positional target]"
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
                --completions)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                --selection-file)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --repeat-file)
                    COMPREPLY=($(compgen -f "${cur}"))
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
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _tgrep -o nosort -o bashdefault -o default tgrep
else
    complete -F _tgrep -o bashdefault -o default tgrep
fi
