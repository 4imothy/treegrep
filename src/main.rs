// SPDX-License-Identifier: MIT

mod args;
mod args_menu;
mod config;
mod errors;
mod log;
mod match_system;
mod matcher;
mod select_menu;
mod style;
mod term;
mod writer;
use clap::ArgMatches;
use clap_complete::generate;
use config::Config;
use errors::Message;
use match_system::Matches;
use select_menu::SelectMenu;
use std::{
    ffi::OsString,
    io::{StdoutLock, stdout},
    sync::OnceLock,
};
use term::Term;
use writer::{Entry, matches_to_display_lines, write_results};

static CONFIG: OnceLock<Config> = OnceLock::new();

fn config() -> &'static Config {
    CONFIG.get().unwrap()
}

fn main() {
    if cfg!(debug_assertions) {
        log::set_panic_hook();
    }
    let (matches, all_args) = config::get_matches(std::env::args_os().skip(1).collect(), true)
        .unwrap_or_else(|e| e.exit());

    let (bold, colors) = Config::get_styling(&matches);
    let out: StdoutLock = stdout().lock();
    let err_prefix = style::error_prefix(bold, colors);
    let (c, mut term) =
        build_config_and_term(matches, all_args, out, bold, colors).unwrap_or_else(|e| {
            eprintln!("{} {}", err_prefix, e);
            std::process::exit(1);
        });
    if let Some(shell) = c.completion_target {
        generate(
            shell,
            &mut args::generate_command(),
            args::names::TREEGREP_BIN,
            &mut std::io::stdout(),
        );
        return;
    }

    let menu = c.menu;
    run(&mut term, c).unwrap_or_else(|e| {
        if menu {
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
    bold: bool,
    colors: bool,
) -> Result<(Config, Term), Message> {
    let mut c = Config::get_config(matches, all_args, bold, colors)?;
    if let Some(mut new_c) = c.handle_repeat()? {
        new_c.selection_file = c.selection_file;
        c = new_c;
    }
    let term = Term::new(out, c.menu || c.select).map_err(|e| mes!("{}", e.to_string()))?;
    Ok((c, term))
}

fn run(term: &mut Term, mut c: Config) -> Result<(), Message> {
    if let Some(f) = &c.selection_file {
        std::fs::write(f, b"").map_err(|e| mes!("{}", e.to_string()))?;
    }

    if c.menu {
        term.claim().map_err(|e| mes!("{}", e.to_string()))?;
        match args_menu::launch(term, c)? {
            Some(new_c) => c = new_c,
            None => {
                term.give().map_err(|e| mes!("{}", e.to_string()))?;
                return Ok(());
            }
        }
    }
    CONFIG.set(c).ok().unwrap();

    let matches: Option<Matches> = matcher::search()?;

    if matches.is_none() {
        if config().menu {
            term.give().map_err(|e| mes!("{}", e.to_string()))?;
        }
        return Ok(());
    }

    let m = matches.unwrap();
    let lines: Vec<Box<dyn Entry>> = matches_to_display_lines(&m)?;

    if config().select {
        if !config().menu {
            term.claim().map_err(|e| mes!("{}", e.to_string()))?;
        }
        SelectMenu::launch(term, &lines).map_err(|e| {
            let _ = term.give();
            mes!("{}", e.to_string())
        })?;
    } else {
        write_results(term, &lines).map_err(|e| mes!("{}", e.to_string()))?;
    }

    Ok(())
}
