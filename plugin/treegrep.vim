" plugin/treegrep.vim

if has('nvim')
  finish
endif

vim9script

if !exists('g:tgrep_selection_file')
    g:tgrep_selection_file = ''
endif
if !exists('g:tgrep_repeat_file')
    g:tgrep_repeat_file = ''
endif
if !exists('g:tgrep_win_width_pct')
    g:tgrep_win_width_pct = 0.7
endif
if !exists('g:tgrep_win_height_pct')
    g:tgrep_win_height_pct = 0.7
endif
if !exists('g:tgrep_borderchars')
    g:tgrep_borderchars = ['─', '│', '─', '│', '╭', '╮', '╯', '╰']
endif
if !exists('g:Tgrep_win_opts_fn')
    g:Tgrep_win_opts_fn = v:null
endif

const file_path = resolve(expand('<sfile>:p'))
const project_root = fnamemodify(file_path, ':h:h')
const tgrep_bin = project_root .. '/target/release/tgrep'

def g:TgrepWith(args: string)
    assert_true(g:tgrep_selection_file !=# '', '`selection_file` must be provided')
    assert_true(args !=# '', 'arguments must be provided to `TgrepWith`')

    var original_win = win_getid()
    var tgrep = filereadable(tgrep_bin) ? tgrep_bin : 'tgrep'
    var cmd = tgrep .. ' --selection-file=' .. fnameescape(g:tgrep_selection_file)

    if g:tgrep_repeat_file !=# ''
        cmd ..= ' --repeat-file=' .. fnameescape(g:tgrep_repeat_file)
    endif

    if args !=# ''
        cmd ..= ' ' .. args
    endif

    var popup_opts: dict<any>
    if g:Tgrep_win_opts_fn != v:null
        echomsg 'calling func'
        popup_opts = g:Tgrep_win_opts_fn()
    else
        echomsg 'not calling func'
        var width = float2nr(&columns * g:tgrep_win_width_pct)
        var height = float2nr(&lines * g:tgrep_win_height_pct)

        popup_opts = {
            line: float2nr((&lines - height) / 2),
            col: float2nr((&columns - width) / 2),
            minwidth: width,
            minheight: height,
            maxwidth: width,
            maxheight: height,
            border: [1, 1, 1, 1],
            borderchars: g:tgrep_borderchars,
            padding: [0, 0, 0, 0],
            zindex: 1,
            drag: 1,
            pos: 'center',
        }
    endif

    var win: number
    var buf: number

    buf = term_start(cmd, {
        term_name: 'tgrep',
        hidden: true,
        term_finish: 'close',
        exit_cb: (_, _) => {
            popup_close(win)
            if bufexists(buf)
                execute 'bdelete!' .. buf
            endif
            call win_gotoid(original_win)
            if filereadable(g:tgrep_selection_file)
                var lines = readfile(g:tgrep_selection_file)
                if len(lines) == 0
                    return
                endif
                var edit_cmd = 'edit ' .. fnameescape(lines[0])
                if len(lines) > 1
                    edit_cmd ..= ' | :' .. lines[1]
                endif
                execute edit_cmd .. ' | redraw'
            endif
        }
    })

    win = popup_create(buf, popup_opts)
enddef

def TgrepBuildCallback(job: job, exit_status: number)
    if exit_status == 0
        echomsg 'tgrep built'
    else
        echomsg 'tgrep build failed'
    endif
enddef

def g:TgrepBuild()
    var manifest = project_root .. '/Cargo.toml'
    var cmd = 'cargo build --release --manifest-path=' .. fnameescape(manifest)
    echomsg 'building tgrep'
    var output = system(cmd)
    if v:shell_error == 0
        echomsg 'tgrep built'
    else
        echomsg 'tgrep build failed: ' .. output
    endif
enddef
