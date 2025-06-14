// SPDX-License-Identifier: MIT

mod args;
mod config;
mod errors;
mod formats;
#[macro_use]
mod log;
mod match_system;
mod matcher;
mod menu;
mod options;
mod output_processor;
mod searchers;
mod term;
mod writer;
use clap::ArgMatches;
use clap_complete::generate;
use config::Config;
use errors::{mes, Message};
use match_system::Matches;
use menu::Menu;
use output_processor::process_results;
use searchers::Searchers;
use std::ffi::OsString;
use std::io::{stdout, StdoutLock};
use std::process::Command;
use std::sync::OnceLock;
use writer::{matches_to_display_lines, write_results, Entry};

static CONFIG: OnceLock<Config> = OnceLock::new();

fn config() -> &'static Config {
    CONFIG.get().unwrap()
}

fn main() {
    if cfg!(debug_assertions) {
        log::set_panic_hook();
    }
    let matches = Config::fill();
    let (bold, colors) = Config::get_styling(&matches);
    run(matches, bold, colors).unwrap_or_else(|e| {
        eprintln!("{} {}", formats::error_prefix(bold, colors), e);
        std::process::exit(1);
    });
}

fn run(matches: ArgMatches, bold: bool, colors: bool) -> Result<(), Message> {
    if let Some((c, sub_m)) = matches.subcommand() {
        if c == args::COMPLETIONS.id {
            let &shell = sub_m
                .get_one::<clap_complete::Shell>(args::SHELL_ID)
                .unwrap();
            let mut cmd = args::generate_command();
            let mut fd = std::io::stdout();
            generate(shell, &mut cmd, args::names::TREEGREP_BIN, &mut fd);
            return Ok(());
        }
    }

    let (c, searcher_path) = Config::get_config(matches, bold, colors)?;
    CONFIG.set(c).ok().unwrap();

    let matches: Option<Matches> = if config().just_files || searcher_path.is_none() {
        matcher::search()?
    } else {
        get_matches_from_cmd(searcher_path.unwrap())?
    };

    if matches.is_none() {
        return Ok(());
    }

    let mut out: StdoutLock = stdout().lock();
    let m = matches.unwrap();
    let mut path_ids = config().menu.then(Vec::<usize>::new);
    let lines: Vec<Box<dyn Entry>> = matches_to_display_lines(&m, path_ids.as_mut())?;

    if config().menu {
        Menu::enter(
            out,
            &lines,
            path_ids
                .map(|mut p| {
                    p.shrink_to_fit();
                    p
                })
                .unwrap(),
        )
        .map_err(|e| mes!("{}", e.to_string()))?;
    } else {
        write_results(&mut out, &lines).map_err(|e| mes!("{}", e.to_string()))?;
    }

    Ok(())
}

fn get_matches_from_cmd(searcher_path: OsString) -> Result<Option<Matches>, Message> {
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
