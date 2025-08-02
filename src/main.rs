// SPDX-License-Identifier: MIT

mod args;
mod args_menu;
mod config;
mod errors;
mod formats;
mod log;
mod match_system;
mod matcher;
mod options;
mod output_processor;
mod searchers;
mod select_menu;
mod term;
mod writer;
use clap::ArgMatches;
use clap_complete::generate;
use config::Config;
use errors::{Message, mes};
use match_system::Matches;
use output_processor::process_results;
use searchers::Searchers;
use select_menu::SelectMenu;
use std::ffi::OsString;
use std::io::{StdoutLock, stdout};
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;
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

    let (bold, colors, menu, select) = Config::get_ui_info(&matches);
    let out: StdoutLock = stdout().lock();
    let err_prefix = formats::error_prefix(bold, colors);
    let mut term = Term::new(out, menu || select)
        .map_err(|e| mes!("{}", e.to_string()))
        .unwrap_or_else(|e| {
            eprintln!("{} {}", err_prefix, e);
            std::process::exit(1);
        });
    run(&mut term, matches, all_args, bold, colors, menu, select).unwrap_or_else(|e| {
        if menu {
            let _ = errors::view_error(&mut term, format!("{} {}", err_prefix, e))
                .map_err(|e| mes!("{}", e.to_string()));
        } else {
            eprintln!("{} {}", err_prefix, e);
            std::process::exit(1);
        }
    });
}

fn run(
    term: &mut Term,
    matches: ArgMatches,
    all_args: Vec<OsString>,
    bold: bool,
    colors: bool,
    menu: bool,
    select: bool,
) -> Result<(), Message> {
    let mut c = Config::get_config(matches, all_args, bold, colors, menu, select)?;
    if let Some(mut new_c) = c.handle_repeat()? {
        new_c.selection_file = c.selection_file;
        c = new_c;
    }

    if let Some(f) = &c.selection_file {
        std::fs::write(f, b"").map_err(|e| mes!("{}", e.to_string()))?;
    }

    if let Some(shell) = c.completion_target {
        generate(
            shell,
            &mut args::generate_command(),
            args::names::TREEGREP_BIN,
            &mut std::io::stdout(),
        );
        return Ok(());
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

    let matches: Option<Matches> = if config().just_files || config().searcher_path.is_none() {
        matcher::search()?
    } else {
        get_matches_from_cmd(config().searcher_path.as_ref().unwrap())?
    };

    if matches.is_none() {
        if config().menu {
            term.give().map_err(|e| mes!("{}", e.to_string()))?;
        }
        return Ok(());
    }

    let m = matches.unwrap();
    let mut path_ids = config().select.then(Vec::<usize>::new);
    let lines: Vec<Box<dyn Entry>> = matches_to_display_lines(&m, path_ids.as_mut())?;

    if config().select {
        if !config().menu {
            term.claim().map_err(|e| mes!("{}", e.to_string()))?;
        }
        SelectMenu::launch(
            term,
            &lines,
            path_ids
                .map(|mut p| {
                    p.shrink_to_fit();
                    p
                })
                .unwrap(),
        )
        .map_err(|e| {
            let _ = term.give();
            mes!("{}", e.to_string())
        })?;
    } else {
        write_results(term, &lines).map_err(|e| mes!("{}", e.to_string()))?;
    }

    Ok(())
}

fn get_matches_from_cmd(searcher_path: &Path) -> Result<Option<Matches>, Message> {
    let mut cmd: Command = Searchers::generate_command(searcher_path)?;

    let output = cmd.output().map_err(|e| {
        mes!(
            "searcher `{}` didn't run message, `{}`",
            cmd.get_program().to_string_lossy().to_string(),
            e.to_string()
        )
    })?;

    if !output.status.success() && !output.stderr.is_empty() {
        return Err(mes!(
            "{} had errors:\n{}",
            cmd.get_program().to_string_lossy(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    process_results(output.stdout)
}
