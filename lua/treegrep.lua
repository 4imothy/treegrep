local M = {}

local defaults = {
    selection_file = nil,
    repeat_file = nil,
    win_width_pct = 0.7,
    win_height_pct = 0.7,
    win_border = 'rounded',
    win_style = 'minimal',
    win_relative = 'editor',
    win_mouse = true,
    win_opts_fn = nil,
}

local config = vim.deepcopy(defaults)

function M.setup(user_opts)
    assert(user_opts.selection_file, '`selection_file` must be defined')
    config = vim.tbl_deep_extend('force', config, user_opts or {})
end

local current_file_path = debug.getinfo(1, 'S').source:sub(2)
local project_root = vim.fs.abspath(vim.fs.dirname(vim.fs.dirname(current_file_path)))
local tgrep_bin = vim.fs.joinpath(project_root, 'target', 'release', 'tgrep')

local function default_win_opts()
    local width = math.floor(vim.o.columns * config.win_width_pct)
    local height = math.floor(vim.o.lines * config.win_height_pct)
    return {
        relative = config.win_relative,
        style = config.win_style,
        border = config.win_border,
        width = width,
        height = height,
        col = math.floor((vim.o.columns - width) / 2),
        row = math.floor((vim.o.lines - height) / 2),
        mouse = config.win_mouse
    }
end

function M.win_opts()
    if config.win_opts then
        return config.win_opts()
    end
    return default_win_opts()
end

function M.tgrep_with(args)
    vim.validate('args', args, 'string')

    local buf = vim.api.nvim_create_buf(false, true)
    local original_win = vim.api.nvim_get_current_win()

    local win = vim.api.nvim_open_win(buf, true, M.win_opts())
    local cmd = {
        vim.uv.fs_stat(tgrep_bin) and tgrep_bin or 'tgrep',
        '--selection-file=' .. vim.fn.fnameescape(config.selection_file)
    }

    if config.repeat_file then
      table.insert(cmd, '--repeat-file=' .. vim.fn.fnameescape(config.repeat_file))
    end

    for arg in args:gmatch('%S+') do
        table.insert(cmd, arg)
    end

    vim.fn.jobstart(cmd, {
        term = true,
        on_exit = function()
            vim.api.nvim_win_close(win, true)
            vim.api.nvim_buf_delete(buf, { force = true })
            vim.api.nvim_set_current_win(original_win)
            if vim.fn.filereadable(config.selection_file) == 1 then
                local lines = vim.fn.readfile(config.selection_file)
                if #lines == 0 then return end
                local edit_cmd = 'edit ' .. vim.fn.fnameescape(lines[1])
                if #lines > 1 then
                    edit_cmd = edit_cmd .. ' | ' .. lines[2]
                end
                vim.cmd(edit_cmd .. ' | redraw')
            end
        end
    })

    vim.cmd('startinsert')
end

function M.build_tgrep()
    local cmd = {
        'cargo',
        'build',
        '--release',
        '--manifest-path=' .. vim.fn.fnameescape(vim.fs.joinpath(project_root, 'Cargo.toml'))
    }
    print('building tgrep')
    vim.system(cmd, {text = true},
    function(obj)
        if obj.code == 0 then
            print('tgrep built')
        else
            error('tgrep build failed: ' .. obj.stderr)
        end
    end):wait()
end

return M
