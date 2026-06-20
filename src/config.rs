// SPDX-License-Identifier: MIT

use crate::{
    args::{self, Args, OpenStrategy, REPEAT_FILE, generate_command},
    errors::Message,
    mes, style,
};
use clap::{ArgMatches, Error, FromArgMatches};
use crossterm::{event::KeyCode, style::Color};
use std::{
    collections::HashSet,
    ffi::OsString,
    path::{Component, Path, PathBuf},
    sync::{Arc, OnceLock},
};

static CONFIG: OnceLock<Arc<Config>> = OnceLock::new();

pub fn base_config() -> Arc<Config> {
    Arc::clone(CONFIG.get().unwrap())
}

pub fn set_config(c: Config) {
    CONFIG.set(Arc::new(c)).ok().unwrap();
}

#[derive(Clone)]
pub struct KeyBindings {
    pub down: Vec<KeyCode>,
    pub up: Vec<KeyCode>,
    pub big_down: Vec<KeyCode>,
    pub big_up: Vec<KeyCode>,
    pub down_path: Vec<KeyCode>,
    pub up_path: Vec<KeyCode>,
    pub down_same_depth: Vec<KeyCode>,
    pub up_same_depth: Vec<KeyCode>,
    pub top: Vec<KeyCode>,
    pub bottom: Vec<KeyCode>,
    pub page_down: Vec<KeyCode>,
    pub page_up: Vec<KeyCode>,
    pub cycle_view: Vec<KeyCode>,
    pub help: Vec<KeyCode>,
    pub quit: Vec<KeyCode>,
    pub open: Vec<KeyCode>,
    pub fold: Vec<KeyCode>,
    pub filter: Vec<KeyCode>,
    pub search: Vec<KeyCode>,
    pub submit_search: Vec<KeyCode>,
}

#[derive(Clone)]
pub struct Characters {
    pub bl: char,
    pub br: char,
    pub tl: char,
    pub tr: char,
    pub v: char,
    pub h: char,
    pub tee: char,
    pub match_with_next: String,
    pub match_no_next: String,
    pub spacer_vert: String,
    pub spacer: String,
    pub selected_indicator: String,
    pub selected_indicator_clear: String,
    pub ellipsis: String,
    pub search_prompt: String,
    pub search_prompt_inactive: String,
    pub filter_prompt: String,
}

#[derive(Clone)]
pub struct Colors {
    pub file: Color,
    pub dir: Color,
    pub line_number: Color,
    pub text: Option<Color>,
    pub branch: Option<Color>,
    pub selected_indicator: Option<Color>,
    pub selected_bg: Color,
    pub matches: Vec<Color>,
    pub filter_highlight: Color,
}

impl args::Color {
    fn get(&self) -> Color {
        match *self {
            args::Color::Black => Color::Black,
            args::Color::White => Color::White,
            args::Color::Red => Color::Red,
            args::Color::Green => Color::Green,
            args::Color::Yellow => Color::Yellow,
            args::Color::Blue => Color::Blue,
            args::Color::Magenta => Color::Magenta,
            args::Color::Cyan => Color::Cyan,
            args::Color::Grey => Color::Grey,
            args::Color::Rgb(r, g, b) => Color::Rgb { r, g, b },
            args::Color::Ansi(value) => Color::AnsiValue(value),
        }
    }
}

#[derive(Clone)]
pub struct SearchParams {
    pub regexps: Vec<String>,
    pub path: PathBuf,
    pub globs: Vec<String>,
    pub hidden: bool,
    pub line_number: bool,
    pub files: bool,
    pub count: bool,
    pub links: bool,
    pub trim: bool,
    pub ignore: bool,
    pub max_depth: Option<usize>,
    pub max_length: Option<usize>,
    pub before_context: usize,
    pub after_context: usize,
    pub overview: bool,
    pub overview_only: bool,
    pub branch_each: usize,
}

#[derive(Clone)]
pub struct Config {
    pub search: SearchParams,
    pub is_dir: bool,
    pub with_bold: bool,
    pub with_colors: bool,
    pub branch_len: usize,
    pub chars: Characters,
    pub colors: Colors,
    pub threads: usize,
    pub selection_file: Option<PathBuf>,
    pub repeat_file: Option<PathBuf>,
    pub select: bool,
    pub menu: bool,
    pub live: bool,
    pub live_delay: Option<u64>,
    pub auto_open: bool,
    pub repeat: bool,
    pub editor: Option<String>,
    pub open_like: Option<OpenStrategy>,
    pub completion_target: Option<clap_complete::Shell>,
    pub keys: KeyBindings,
    pub mouse: bool,
    pub alternate_screen: bool,
}

fn canonicalize(p: &Path) -> Result<PathBuf, Message> {
    dunce::canonicalize(p)
        .map_err(|_| mes!("failed to canonicalize path `{}`", p.to_string_lossy()))
}

fn process_path(input: &Path, check_exists: bool) -> Result<PathBuf, Message> {
    let mut components = input.components();
    let mut path = PathBuf::new();
    match components.next() {
        Some(Component::Normal(c)) => {
            if c == "~" {
                path.push(std::env::var("HOME").map_err(|e| mes!("{}", e))?);
            } else {
                path.push(c);
            }
        }
        Some(c) => path.push(c),
        _ => {}
    }

    for c in components {
        path.push(c);
    }
    if check_exists {
        path.exists()
            .then(|| canonicalize(&path))
            .ok_or_else(|| mes!("failed to find path `{}`", path.to_string_lossy()))?
    } else {
        Ok(path)
    }
}

fn get_all_args(mut args: Vec<OsString>) -> Vec<OsString> {
    if let Some(env_args) = std::env::var_os(args::DEFAULT_OPTS_ENV_NAME) {
        args.splice(
            0..0,
            env_args
                .into_string()
                .unwrap_or_default()
                .split_whitespace()
                .map(OsString::from),
        );
    }
    args
}

pub fn get_matches(args: Vec<OsString>) -> Result<ArgMatches, Error> {
    generate_command().try_get_matches_from(get_all_args(args))
}

fn apply_matches(
    base_non_search: NonSearchFields,
    matches: &ArgMatches,
    only_explicit: bool,
) -> Result<Config, Message> {
    let args = Args::from_arg_matches(matches).map_err(|e| mes!("{}", e))?;

    if only_explicit {
        let applies =
            |id: &str| matches.value_source(id) == Some(clap::parser::ValueSource::CommandLine);
        let mut c = (*base_config()).clone();
        c.selection_file = base_non_search.selection_file;
        c.repeat_file = base_non_search.repeat_file;
        c.select = base_non_search.select;
        c.menu = base_non_search.menu;
        c.live = base_non_search.live;
        c.live_delay = base_non_search.live_delay;
        c.auto_open = base_non_search.auto_open;
        c.repeat = base_non_search.repeat;
        c.editor = base_non_search.editor;
        c.open_like = base_non_search.open_like;
        c.completion_target = base_non_search.completion_target;
        c.keys = base_non_search.keys;
        c.mouse = base_non_search.mouse;
        c.alternate_screen = base_non_search.alternate_screen;
        if let Some(p) = args.positional_path.as_ref().or(args.path.as_ref()) {
            c.search.path = process_path(p, true)?;
            c.is_dir = c.search.path.is_dir();
        }
        if applies(args::EXPRESSION_POSITIONAL) || applies(args::EXPRESSION) {
            c.search.regexps.clear();
            if let Some(expr) = &args.positional_regexp {
                c.search.regexps.push(expr.clone());
            }
            c.search.regexps.extend(args.regexp.iter().cloned());
        }
        if applies(args::GLOB) {
            c.search.globs = args.glob.clone();
        }
        if applies(args::HIDDEN) {
            c.search.hidden = args.hidden;
        }
        if applies(args::LINE_NUMBER) {
            c.search.line_number = args.line_number;
        }
        if applies(args::FILES) {
            c.search.files = args.files;
        }
        if applies(args::COUNT) {
            c.search.count = args.count;
        }
        if applies(args::LINKS) {
            c.search.links = args.links;
        }
        if applies(args::TRIM_LEFT) {
            c.search.trim = args.trim;
        }
        if applies(args::NO_IGNORE) {
            c.search.ignore = !args.no_ignore;
        }
        if applies(args::MAX_DEPTH) {
            c.search.max_depth = args.max_depth;
        }
        if applies(args::MAX_LENGTH) {
            c.search.max_length = args.max_length;
        }
        if applies(args::LONG_BRANCHES_EACH) {
            c.search.branch_each = args.branch_each;
        }
        let context_both = if applies(args::CONTEXT) {
            args.context.unwrap_or(0)
        } else {
            0
        };
        if applies(args::BEFORE_CONTEXT) || applies(args::CONTEXT) {
            c.search.before_context = args.before_context.unwrap_or(context_both);
        }
        if applies(args::AFTER_CONTEXT) || applies(args::CONTEXT) {
            c.search.after_context = args.after_context.unwrap_or(context_both);
        }
        if applies(args::NO_COLORS) {
            c.with_colors = !args.no_color;
        }
        if applies(args::NO_BOLD) {
            c.with_bold = !args.no_bold;
        }
        if applies(args::OVERVIEW) || applies(args::OVERVIEW_ONLY) {
            c.search.overview = args.overview || args.overview_only;
        }
        if applies(args::OVERVIEW_ONLY) {
            c.search.overview_only = args.overview_only;
        }
        if applies(args::BRANCH_LEN) {
            c.branch_len = args.branch_len;
        }
        if applies(args::CHAR_VERTICAL) {
            c.chars.v = args.char_vertical;
        }
        if applies(args::CHAR_HORIZONTAL) {
            c.chars.h = args.char_horizontal;
        }
        if applies(args::CHAR_TOP_LEFT) {
            c.chars.tl = args.char_top_left;
        }
        if applies(args::CHAR_TOP_RIGHT) {
            c.chars.tr = args.char_top_right;
        }
        if applies(args::CHAR_BOTTOM_LEFT) {
            c.chars.bl = args.char_bottom_left;
        }
        if applies(args::CHAR_BOTTOM_RIGHT) {
            c.chars.br = args.char_bottom_right;
        }
        if applies(args::CHAR_TEE) {
            c.chars.tee = args.char_tee;
        }
        let s = c.branch_len;
        c.chars.match_with_next = format!("{}{}", c.chars.tee, style::repeat(c.chars.h, s - 1));
        c.chars.match_no_next = format!("{}{}", c.chars.bl, style::repeat(c.chars.h, s - 1));
        c.chars.spacer_vert = format!("{}{}", c.chars.v, style::repeat(' ', s - 1));
        c.chars.spacer = " ".repeat(s);
        if applies(args::SELECTED_INDICATOR) {
            c.chars.selected_indicator_clear = " ".repeat(args.selected_indicator.chars().count());
            c.chars.selected_indicator = args.selected_indicator.clone();
        }
        if applies(args::ELLIPSES) {
            c.chars.ellipsis = args.ellipsis.clone();
        }
        if applies(args::SEARCH_PROMPT) {
            c.chars.search_prompt = args.search_prompt.clone();
        }
        if applies(args::SEARCH_PROMPT_INACTIVE) {
            c.chars.search_prompt_inactive = args.search_prompt_inactive.clone();
        }
        if applies(args::FILTER_PROMPT) {
            c.chars.filter_prompt = args.filter_prompt.clone();
        }
        if applies("file_color")
            && let Some(v) = &args.file_color
        {
            c.colors.file = v.get();
        }
        if applies("dir_color")
            && let Some(v) = &args.dir_color
        {
            c.colors.dir = v.get();
        }
        if applies("text_color") {
            c.colors.text = args.text_color.as_ref().map(|v| v.get());
        }
        if applies("branch_color") {
            c.colors.branch = args.branch_color.as_ref().map(|v| v.get());
        }
        if applies("line_number_color")
            && let Some(v) = &args.line_number_color
        {
            c.colors.line_number = v.get();
        }
        if applies("match_colors") && !args.match_colors.is_empty() {
            c.colors.matches = args.match_colors.iter().map(|v| v.get()).collect();
        }
        if applies("selected_indicator_color") {
            c.colors.selected_indicator = args.selected_indicator_color.as_ref().map(|v| v.get());
        }
        if applies("selected_bg_color")
            && let Some(v) = &args.selected_bg_color
        {
            c.colors.selected_bg = v.get();
        }
        if applies("filter_highlight_color")
            && let Some(v) = &args.filter_highlight_color
        {
            c.colors.filter_highlight = v.get();
        }
        return Ok(c);
    }

    let (path, is_dir) = match args.positional_path.as_ref().or(args.path.as_ref()) {
        Some(p) => {
            let p = process_path(p, true)?;
            let d = p.is_dir();
            (p, d)
        }
        None => {
            let p = canonicalize(
                &std::env::current_dir()
                    .map_err(|_| mes!("failed to get current working directory"))?,
            )?;
            let d = p.is_dir();
            (p, d)
        }
    };
    let mut regexps = Vec::new();
    if let Some(expr) = &args.positional_regexp {
        regexps.push(expr.clone());
    }
    regexps.extend(args.regexp.iter().cloned());
    let context = args.context.unwrap_or(0);
    let branch_len = args.branch_len;
    let chars = Config::get_characters(&args);
    Ok(Config {
        search: SearchParams {
            path,
            regexps,
            globs: args.glob.clone(),
            hidden: args.hidden,
            line_number: args.line_number,
            files: args.files,
            count: args.count,
            links: args.links,
            trim: args.trim,
            ignore: !args.no_ignore,
            max_depth: args.max_depth,
            max_length: args.max_length,
            before_context: args.before_context.unwrap_or(context),
            after_context: args.after_context.unwrap_or(context),
            overview: args.overview || args.overview_only,
            overview_only: args.overview_only,
            branch_each: args.branch_each,
        },
        is_dir,
        with_bold: !args.no_bold,
        with_colors: !args.no_color,
        branch_len,
        threads: base_non_search.threads,
        chars,
        colors: Config::get_colors(&args),
        selection_file: base_non_search.selection_file,
        repeat_file: base_non_search.repeat_file,
        select: base_non_search.select,
        menu: base_non_search.menu,
        live: base_non_search.live,
        live_delay: base_non_search.live_delay,
        auto_open: base_non_search.auto_open,
        repeat: base_non_search.repeat,
        editor: base_non_search.editor,
        open_like: base_non_search.open_like,
        completion_target: base_non_search.completion_target,
        keys: base_non_search.keys,
        mouse: base_non_search.mouse,
        alternate_screen: base_non_search.alternate_screen,
    })
}

struct NonSearchFields {
    selection_file: Option<PathBuf>,
    repeat_file: Option<PathBuf>,
    select: bool,
    menu: bool,
    live: bool,
    live_delay: Option<u64>,
    auto_open: bool,
    repeat: bool,
    threads: usize,
    editor: Option<String>,
    open_like: Option<OpenStrategy>,
    completion_target: Option<clap_complete::Shell>,
    keys: KeyBindings,
    mouse: bool,
    alternate_screen: bool,
}

impl Config {
    pub fn get_styling(matches: &ArgMatches) -> (bool, bool) {
        let args = Args::from_arg_matches(matches).expect("failed to parse args");
        (!args.no_bold, !args.no_color)
    }

    fn read_repeat_file(f: &Path) -> Result<Vec<OsString>, Message> {
        let data = std::fs::read(f).map_err(|e| mes!("{}", e))?;
        let mut pos = 0;
        let mut args = Vec::new();
        while pos < data.len() {
            let len = u32::from_le_bytes(
                data[pos..pos + size_of::<u32>()]
                    .try_into()
                    .map_err(|e| mes!("{}", e))?,
            ) as usize;
            pos += size_of::<u32>();
            let bytes = &data[pos..pos + len];
            pos += len;
            unsafe {
                args.push(OsString::from_encoded_bytes_unchecked(bytes.to_vec()));
            }
        }
        Ok(args)
    }

    pub fn handle_repeat(&self) -> Result<Option<Self>, Message> {
        if let Some(f) = &self.repeat_file {
            if self.repeat {
                let args = Self::read_repeat_file(f)?;
                let matches = get_matches(args).map_err(|e| mes!("{}", e))?;
                let mut c = Self::get_config(matches)?;
                c.is_dir = c.search.path.is_dir();
                c.with_bold = self.with_bold;
                c.with_colors = self.with_colors;
                c.branch_len = self.branch_len;
                c.chars = self.chars.clone();
                c.colors = self.colors.clone();
                c.selection_file = self.selection_file.clone();
                c.repeat_file = self.repeat_file.clone();
                c.select = self.select;
                c.menu = self.menu;
                c.live = self.live;
                c.live_delay = self.live_delay;
                c.auto_open = self.auto_open;
                c.editor = self.editor.clone();
                c.open_like = self.open_like.clone();
                c.keys = self.keys.clone();
                c.mouse = self.mouse;
                c.alternate_screen = self.alternate_screen;
                Ok(Some(c))
            } else {
                if !self.search.regexps.is_empty() {
                    save_search_params(f, self)?;
                }
                Ok(None)
            }
        } else if self.repeat {
            Err(mes!("cannot repeat without a {} specified", REPEAT_FILE))
        } else {
            Ok(None)
        }
    }

    pub fn get_config(matches: ArgMatches) -> Result<Self, Message> {
        let args = Args::from_arg_matches(&matches).map_err(|e| mes!("{}", e))?;

        let threads = args.threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map_or(1, |n| n.get())
                .min(12)
        });
        let selection_file = args
            .selection_file
            .as_ref()
            .map(|p| process_path(p, false))
            .transpose()?;
        let repeat_file = args
            .repeat_file
            .as_ref()
            .map(|p| process_path(p, false))
            .transpose()?;

        let non_search = NonSearchFields {
            select: args.select,
            menu: args.menu,
            live: args.live,
            live_delay: args.live_delay,
            auto_open: args.auto_open,
            repeat: args.repeat,
            threads,
            editor: args.editor.clone(),
            open_like: args.open_like.clone(),
            selection_file,
            repeat_file,
            completion_target: args.completions,
            keys: Config::get_key_bindings(&args, &matches),
            mouse: !args.no_mouse,
            alternate_screen: !args.no_alternate_screen,
        };
        apply_matches(non_search, &matches, false)
    }

    fn get_colors(args: &Args) -> Colors {
        Colors {
            file: args
                .file_color
                .as_ref()
                .map(|v| v.get())
                .unwrap_or(style::FILE_COLOR_DEFAULT),
            dir: args
                .dir_color
                .as_ref()
                .map(|v| v.get())
                .unwrap_or(style::DIR_COLOR_DEFAULT),
            line_number: args
                .line_number_color
                .as_ref()
                .map(|v| v.get())
                .unwrap_or(style::LINE_NUMBER_COLOR_DEFAULT),
            text: args.text_color.as_ref().map(|v| v.get()),
            branch: args.branch_color.as_ref().map(|v| v.get()),
            selected_bg: args
                .selected_bg_color
                .as_ref()
                .map(|v| v.get())
                .unwrap_or(style::SELECTED_BG_DEFAULT),
            selected_indicator: args.selected_indicator_color.as_ref().map(|v| v.get()),
            filter_highlight: args
                .filter_highlight_color
                .as_ref()
                .map(|v| v.get())
                .unwrap_or(style::FILTER_HIGHLIGHT_DEFAULT),
            matches: if args.match_colors.is_empty() {
                style::MATCHED_COLORS_DEFAULT.to_vec()
            } else {
                args.match_colors.iter().map(|v| v.get()).collect()
            },
        }
    }

    fn get_characters(args: &Args) -> Characters {
        let v = args.char_vertical;
        let h = args.char_horizontal;
        let tl = args.char_top_left;
        let tr = args.char_top_right;
        let bl = args.char_bottom_left;
        let br = args.char_bottom_right;
        let tee = args.char_tee;
        let spacer = args.branch_len;
        let ellipsis = args.ellipsis.clone();
        let selected_indicator = args.selected_indicator.clone();
        let selected_indicator_clear = " ".repeat(selected_indicator.chars().count());
        let search_prompt = args.search_prompt.clone();
        let search_prompt_inactive = args.search_prompt_inactive.clone();
        let filter_prompt = args.filter_prompt.clone();

        Characters {
            bl,
            br,
            tl,
            tr,
            v,
            h,
            tee,
            selected_indicator,
            selected_indicator_clear,
            match_with_next: format!("{}{}", tee, style::repeat(h, spacer - 1)),
            match_no_next: format!("{}{}", bl, style::repeat(h, spacer - 1)),
            spacer_vert: format!("{}{}", v, style::repeat(' ', spacer - 1)),
            spacer: " ".repeat(spacer),
            ellipsis,
            search_prompt,
            search_prompt_inactive,
            filter_prompt,
        }
    }

    fn get_key_bindings(args: &Args, matches: &ArgMatches) -> KeyBindings {
        let all_ids = [
            args::KEY_DOWN,
            args::KEY_UP,
            args::KEY_BIG_DOWN,
            args::KEY_BIG_UP,
            args::KEY_DOWN_PATH,
            args::KEY_UP_PATH,
            args::KEY_DOWN_SAME_DEPTH,
            args::KEY_UP_SAME_DEPTH,
            args::KEY_TOP,
            args::KEY_BOTTOM,
            args::KEY_PAGE_DOWN,
            args::KEY_PAGE_UP,
            args::KEY_CYCLE_VIEW,
            args::KEY_HELP,
            args::KEY_QUIT,
            args::KEY_OPEN,
            args::KEY_FOLD,
            args::KEY_FILTER,
            args::KEY_SEARCH,
        ];
        let is_user_set = |id: &str| -> bool {
            matches.value_source(id) == Some(clap::parser::ValueSource::CommandLine)
        };
        let overrides: HashSet<KeyCode> = all_ids
            .iter()
            .filter(|id| is_user_set(id))
            .flat_map(|id| key_field_by_id(args, id))
            .copied()
            .collect();
        let binding = |id: &str, keys: &[KeyCode]| -> Vec<KeyCode> {
            if is_user_set(id) {
                keys.to_vec()
            } else {
                keys.iter()
                    .copied()
                    .filter(|k| !overrides.contains(k))
                    .collect()
            }
        };
        KeyBindings {
            down: binding(args::KEY_DOWN, &args.key_down),
            up: binding(args::KEY_UP, &args.key_up),
            big_down: binding(args::KEY_BIG_DOWN, &args.key_big_down),
            big_up: binding(args::KEY_BIG_UP, &args.key_big_up),
            down_path: binding(args::KEY_DOWN_PATH, &args.key_down_path),
            up_path: binding(args::KEY_UP_PATH, &args.key_up_path),
            down_same_depth: binding(args::KEY_DOWN_SAME_DEPTH, &args.key_down_same_depth),
            up_same_depth: binding(args::KEY_UP_SAME_DEPTH, &args.key_up_same_depth),
            top: binding(args::KEY_TOP, &args.key_top),
            bottom: binding(args::KEY_BOTTOM, &args.key_bottom),
            page_down: binding(args::KEY_PAGE_DOWN, &args.key_page_down),
            page_up: binding(args::KEY_PAGE_UP, &args.key_page_up),
            cycle_view: binding(args::KEY_CYCLE_VIEW, &args.key_cycle_view),
            help: binding(args::KEY_HELP, &args.key_help),
            quit: binding(args::KEY_QUIT, &args.key_quit),
            open: binding(args::KEY_OPEN, &args.key_open),
            fold: binding(args::KEY_FOLD, &args.key_fold),
            filter: binding(args::KEY_FILTER, &args.key_filter),
            search: binding(args::KEY_SEARCH, &args.key_search),
            submit_search: binding(args::KEY_SUBMIT_SEARCH, &args.key_submit_search),
        }
    }
}

fn key_field_by_id<'a>(args: &'a Args, id: &str) -> &'a [KeyCode] {
    match id {
        _ if id == args::KEY_DOWN => &args.key_down,
        _ if id == args::KEY_UP => &args.key_up,
        _ if id == args::KEY_BIG_DOWN => &args.key_big_down,
        _ if id == args::KEY_BIG_UP => &args.key_big_up,
        _ if id == args::KEY_DOWN_PATH => &args.key_down_path,
        _ if id == args::KEY_UP_PATH => &args.key_up_path,
        _ if id == args::KEY_DOWN_SAME_DEPTH => &args.key_down_same_depth,
        _ if id == args::KEY_UP_SAME_DEPTH => &args.key_up_same_depth,
        _ if id == args::KEY_TOP => &args.key_top,
        _ if id == args::KEY_BOTTOM => &args.key_bottom,
        _ if id == args::KEY_PAGE_DOWN => &args.key_page_down,
        _ if id == args::KEY_PAGE_UP => &args.key_page_up,
        _ if id == args::KEY_CYCLE_VIEW => &args.key_cycle_view,
        _ if id == args::KEY_HELP => &args.key_help,
        _ if id == args::KEY_QUIT => &args.key_quit,
        _ if id == args::KEY_OPEN => &args.key_open,
        _ if id == args::KEY_FOLD => &args.key_fold,
        _ if id == args::KEY_FILTER => &args.key_filter,
        _ if id == args::KEY_SEARCH => &args.key_search,
        _ => &[],
    }
}

const MENU_UNSUPPORTED: &[&str] = &[
    args::SELECT,
    args::MENU,
    args::REPEAT,
    args::REPEAT_FILE,
    args::SELECTION_FILE,
    args::AUTO_OPEN,
    args::COMPLETIONS,
    args::KEY_DOWN,
    args::KEY_UP,
    args::KEY_BIG_DOWN,
    args::KEY_BIG_UP,
    args::KEY_DOWN_PATH,
    args::KEY_UP_PATH,
    args::KEY_DOWN_SAME_DEPTH,
    args::KEY_UP_SAME_DEPTH,
    args::KEY_TOP,
    args::KEY_BOTTOM,
    args::KEY_PAGE_DOWN,
    args::KEY_PAGE_UP,
    args::KEY_CYCLE_VIEW,
    args::KEY_HELP,
    args::KEY_QUIT,
    args::KEY_OPEN,
    args::KEY_FOLD,
    args::KEY_FILTER,
    args::KEY_SEARCH,
    args::KEY_SUBMIT_SEARCH,
];

fn id_to_flag(id: &str) -> String {
    format!("--{}", id.replace('_', "-"))
}

impl SearchParams {
    fn to_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();
        for r in &self.regexps {
            args.push(format!("{}={r}", id_to_flag(args::EXPRESSION)).into());
        }
        args.push(format!("{}={}", id_to_flag(args::PATH), self.path.display()).into());
        for g in &self.globs {
            args.push(format!("{}={g}", id_to_flag(args::GLOB)).into());
        }
        if self.hidden {
            args.push(id_to_flag(args::HIDDEN).into());
        }
        if self.line_number {
            args.push(id_to_flag(args::LINE_NUMBER).into());
        }
        if self.files {
            args.push(id_to_flag(args::FILES).into());
        }
        if self.count {
            args.push(id_to_flag(args::COUNT).into());
        }
        if self.links {
            args.push(id_to_flag(args::LINKS).into());
        }
        if self.trim {
            args.push(id_to_flag(args::TRIM_LEFT).into());
        }
        if !self.ignore {
            args.push(id_to_flag(args::NO_IGNORE).into());
        }
        if let Some(d) = self.max_depth {
            args.push(format!("{}={d}", id_to_flag(args::MAX_DEPTH)).into());
        }
        if let Some(l) = self.max_length {
            args.push(format!("{}={l}", id_to_flag(args::MAX_LENGTH)).into());
        }
        if self.before_context > 0 {
            args.push(
                format!(
                    "{}={}",
                    id_to_flag(args::BEFORE_CONTEXT),
                    self.before_context
                )
                .into(),
            );
        }
        if self.after_context > 0 {
            args.push(format!("{}={}", id_to_flag(args::AFTER_CONTEXT), self.after_context).into());
        }
        if self.overview {
            args.push(id_to_flag(args::OVERVIEW).into());
        }
        if self.overview_only {
            args.push(id_to_flag(args::OVERVIEW_ONLY).into());
        }
        if self.branch_each > 1 {
            args.push(
                format!(
                    "{}={}",
                    id_to_flag(args::LONG_BRANCHES_EACH),
                    self.branch_each
                )
                .into(),
            );
        }
        args
    }
}

pub fn save_search_params(file: &Path, config: &Config) -> Result<(), Message> {
    save_to_repeat_file(file, &config.search.to_args())
}

pub fn save_to_repeat_file(file: &std::path::Path, args: &[OsString]) -> Result<(), Message> {
    let mut buffer = Vec::new();
    for arg in args {
        let bytes = arg.as_encoded_bytes();
        let len = bytes.len() as u32;
        buffer.extend_from_slice(&len.to_le_bytes());
        buffer.extend_from_slice(bytes);
    }
    std::fs::write(file, buffer).map_err(|e| mes!("{}", e))
}

pub fn parse_menu_query(query: &str) -> Result<Config, Message> {
    let base = base_config();
    let tokens = shlex::split(query).ok_or_else(|| mes!("invalid query syntax"))?;

    let full_args: Vec<OsString> = tokens.iter().map(OsString::from).collect();

    let matches = generate_command()
        .try_get_matches_from(full_args)
        .map_err(|mut e| {
            use clap::error::ContextKind;
            e.remove(ContextKind::Usage);
            e.remove(ContextKind::Suggested);
            mes!("{}", e.render().to_string().trim_end())
        })?;

    let unsupported: Vec<&str> = matches
        .ids()
        .map(|id| id.as_str())
        .filter(|id| {
            matches.value_source(id) == Some(clap::parser::ValueSource::CommandLine)
                && MENU_UNSUPPORTED.contains(id)
        })
        .collect();
    if !unsupported.is_empty() {
        return Err(mes!(
            "unsupported in menu search: {}",
            unsupported
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let args = Args::from_arg_matches(&matches).map_err(|e| mes!("{}", e))?;
    let explicit =
        |id: &str| matches.value_source(id) == Some(clap::parser::ValueSource::CommandLine);
    let non_search = NonSearchFields {
        selection_file: base.selection_file.clone(),
        repeat_file: base.repeat_file.clone(),
        select: false,
        menu: true,
        live: base.live,
        live_delay: base.live_delay,
        auto_open: base.auto_open,
        repeat: base.repeat,
        threads: base.threads,
        editor: base.editor.clone(),
        open_like: base.open_like.clone(),
        completion_target: base.completion_target,
        keys: base.keys.clone(),
        mouse: base.mouse,
        alternate_screen: base.alternate_screen,
    };
    let mut c = apply_matches(non_search, &matches, true)?;
    let explicit_pattern = explicit(args::EXPRESSION_POSITIONAL) || explicit(args::EXPRESSION);
    if explicit(args::FILES) && !explicit_pattern {
        c.search.regexps.clear();
    }
    if explicit_pattern && !explicit(args::FILES) {
        c.search.files = false;
    }
    drop(args);
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args;

    static EXAMPLE_LONG_OPTS: &[&str] = &[
        "posexpr",
        "--line-number",
        "--max-depth=5",
        "--max-length=20",
        "--no-ignore",
        "--hidden",
        "--threads=8",
        "--count",
        "--links",
        "--trim",
        "--select",
        "--files",
        "--overview",
        "--glob=*.rs",
        "--regexp=regexp1",
        "--regexp=regexp2",
        "--after-context=2",
        "--before-context=3",
        "--context=1",
    ];

    pub fn get_config_from<I, T>(args: I) -> Config
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let matches = generate_command().get_matches_from(args);
        Config::get_config(matches).ok().unwrap()
    }

    #[test]
    fn test_env_opts() {
        unsafe { std::env::set_var(args::DEFAULT_OPTS_ENV_NAME, EXAMPLE_LONG_OPTS.join(" ")) };
        let matches = get_matches(Vec::new()).unwrap();
        let config = Config::get_config(matches).ok().unwrap();
        check_parsed_config_from_example_opts(config);
    }

    #[test]
    fn test_longs() {
        let config = get_config_from(EXAMPLE_LONG_OPTS);
        check_parsed_config_from_example_opts(config);
    }

    fn check_parsed_config_from_example_opts(config: Config) {
        assert!(config.search.line_number);
        assert_eq!(config.search.max_depth, Some(5));
        assert_eq!(config.search.max_length, Some(20));
        assert!(!config.search.ignore);
        assert!(config.search.hidden);
        assert!(config.threads == 8);
        assert!(config.search.count);
        assert!(config.search.links);
        assert!(config.search.trim);
        assert!(config.with_colors);
        assert!(config.select);
        assert!(config.search.files);
        assert!(config.search.overview);
        assert_eq!(config.search.globs, vec!["*.rs"]);
        assert_eq!(config.search.before_context, 3);
        assert_eq!(config.search.after_context, 2);
        assert_eq!(config.search.regexps, vec!["posexpr", "regexp1", "regexp2"]);
    }

    #[test]
    fn test_shorts() {
        let config = get_config_from([
            "posexpr",
            "-n.csfl",
            "-e=regexp1",
            "-e=regexp2",
            "-d=5",
            "-g=*.rs",
            "-o",
            "-A=2",
            "-B=3",
            "-C=1",
        ]);
        assert!(config.search.line_number);
        assert!(config.search.hidden);
        assert!(config.search.count);
        assert!(config.select);
        assert!(config.search.files);
        assert!(config.search.links);
        assert!(config.search.overview);
        assert_eq!(config.search.max_depth, Some(5));
        assert_eq!(config.search.globs, vec!["*.rs"]);
        assert_eq!(config.search.after_context, 2);
        assert_eq!(config.search.before_context, 3);
        assert_eq!(config.search.regexps, vec!["posexpr", "regexp1", "regexp2"]);
    }

    #[test]
    fn test_longs_files() {
        let config = get_config_from([
            "--files",
            "--max-depth=5",
            "--no-ignore",
            "--hidden",
            "--links",
            "--select",
        ]);
        assert_eq!(config.search.max_depth, Some(5));
        assert!(!config.search.ignore);
        assert!(config.search.hidden);
        assert!(config.search.links);
        assert!(config.with_colors);
        assert!(config.select);
    }

    #[test]
    fn test_key_binding_hardware_named() {
        let config = get_config_from(["expression", "--key-quit=f10", "--key-down=x"]);
        assert!(config.keys.quit.contains(&KeyCode::F(10)));
        assert!(config.keys.down.contains(&KeyCode::Char('x')));
        assert!(!config.keys.down.contains(&KeyCode::Char('j')));
        assert!(!config.keys.down.contains(&KeyCode::Down));
    }

    #[test]
    fn test_shorts_files() {
        let config = get_config_from(["-.fslo", "-d=3"]);
        assert!(config.search.hidden);
        assert!(config.search.links);
        assert!(config.search.overview);
        assert_eq!(config.search.max_depth, Some(3));
        assert!(config.select);
    }
}
