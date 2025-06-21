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
use errors::{mes, Message};
use match_system::Matches;
use output_processor::process_results;
use searchers::Searchers;
use select_menu::PickerMenu;
use std::ffi::OsString;
use std::io::{stdout, StdoutLock};
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;
use term::Term;
use writer::{matches_to_display_lines, write_results, Entry};

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
    run(matches, all_args, bold, colors).unwrap_or_else(|e| {
        eprintln!("{} {}", formats::error_prefix(bold, colors), e);
        std::process::exit(1);
    });
}

fn run(
    matches: ArgMatches,
    all_args: Vec<OsString>,
    bold: bool,
    colors: bool,
) -> Result<(), Message> {
    let mut c = Config::get_config(matches, all_args, bold, colors)?;
    if let Some(mut new_c) = c.handle_repeat()? {
        new_c.selection_file = c.selection_file;
        c = new_c;
    }

    if c.plugin_support {
        return Ok(());
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

    let out: StdoutLock = stdout().lock();
    let mut term = Term::new(out).map_err(|e| mes!("{}", e.to_string()))?;
    if c.menu {
        match args_menu::launch(&mut term, c) {
            Ok(Some(new_c)) => c = new_c,
            Ok(None) => {
                term.give().map_err(|e| mes!("{}", e.to_string()))?;
                return Ok(());
            }
            Err(e) => {
                return args_menu::view_error(&mut term, e.to_string())
                    .map_err(|e| mes!("{}", e.to_string()));
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
        PickerMenu::enter(
            &mut term,
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
        write_results(&mut term, &lines).map_err(|e| mes!("{}", e.to_string()))?;
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
