// SPDX-License-Identifier: MIT

mod args;
mod config;
mod errors;
mod log;
mod match_system;
mod matcher;
mod menu;
mod style;
mod term;
mod writer;
use clap::ArgMatches;
use clap_complete::generate;
use config::Config;
use errors::Message;
use match_system::Matches;
use std::{
    ffi::OsString,
    io::{StdoutLock, stdout},
    sync::{Arc, atomic::AtomicBool},
};
use term::Term;
use writer::{matches_to_display_lines, write_results};

fn main() {
    if cfg!(debug_assertions) {
        log::set_panic_hook();
    }
    let (matches, all_args) =
        config::get_matches(std::env::args_os().skip(1).collect()).unwrap_or_else(|e| e.exit());

    let (bold, colors) = Config::get_styling(&matches);
    let out: StdoutLock = stdout().lock();
    let err_prefix = style::error_prefix(bold, colors);
    let (c, mut term) = build_config_and_term(matches, all_args, out).unwrap_or_else(|e| {
        eprintln!("{} {}", err_prefix, e);
        std::process::exit(1);
    });
    if let Some(shell) = c.core.completion_target {
        generate(
            shell,
            &mut args::generate_command(),
            args::names::TREEGREP_BIN,
            &mut std::io::stdout(),
        );
        return;
    }

    let is_menu = c.core.menu;
    run(&mut term, c).unwrap_or_else(|e| {
        if is_menu {
            let _ = errors::view_error(&mut term, format!("{} {}", err_prefix, e)).map_err(|_| {
                std::process::exit(1);
            });
        } else {
            eprintln!("{} {}", err_prefix, e);
            std::process::exit(1);
        }
    });
}

fn build_config_and_term(
    matches: ArgMatches,
    all_args: Vec<OsString>,
    out: StdoutLock,
) -> Result<(Config, Term), Message> {
    let mut c = Config::get_config(matches, all_args)?;
    if let Some(repeated) = c.handle_repeat()? {
        c = repeated;
    }
    let term = Term::new(out, c.core.menu || c.core.select).map_err(|e| mes!("{}", e))?;
    Ok((c, term))
}

fn auto_open_target(m: &Matches, files_only: bool) -> Option<(&std::path::Path, Option<usize>)> {
    let files: Vec<&match_system::File> = match m {
        Matches::Dir(dirs) => dirs.iter().flat_map(|d| d.files.iter()).collect(),
        Matches::File(file) => vec![file],
    };
    if files_only {
        return if files.len() == 1 {
            Some((files[0].path.as_path(), None))
        } else {
            None
        };
    }
    let mut found: Option<(&std::path::Path, usize)> = None;
    let mut multiple = false;
    'outer: for file in &files {
        for line in &file.lines {
            if !line.matches.is_empty() {
                if found.is_some() {
                    multiple = true;
                    break 'outer;
                }
                found = Some((file.path.as_path(), line.line_num));
            }
        }
    }
    if multiple {
        return None;
    }
    if let Some((path, line_num)) = found {
        return Some((path, Some(line_num)));
    }
    if files.len() == 1 {
        Some((files[0].path.as_path(), None))
    } else {
        None
    }
}

fn run(term: &mut Term, c: Config) -> Result<(), Message> {
    if let Some(f) = &c.core.selection_file {
        std::fs::write(f, b"").map_err(|e| mes!("{}", e))?;
    }
    config::set_config(c);
    let c = config::base_config();

    let matches = if c.core.menu && c.regexps.is_empty() && !c.files {
        None
    } else {
        matcher::search(Arc::new(AtomicBool::new(false)), Arc::clone(&c))?
    };

    if c.core.auto_open
        && let Some(m) = matches.as_ref()
        && let Some((path, line)) = auto_open_target(m, c.files)
    {
        menu::open_path(path.as_os_str().to_os_string(), line).map_err(|e| mes!("{}", e))?;
        return Ok(());
    }

    if c.core.select || c.core.menu {
        if matches.is_none() && !c.core.menu {
            Ok(())
        } else {
            term.claim().map_err(|e| mes!("{}", e))?;
            menu::Menu::launch(term, matches).map_err(|e| {
                let _ = term.give();
                mes!("{}", e)
            })
        }
    } else if let Some(m) = matches {
        let lines = matches_to_display_lines(&m, Arc::clone(&c))?;
        write_results(term, &lines).map_err(|e| mes!("{}", e))
    } else {
        Ok(())
    }
}
