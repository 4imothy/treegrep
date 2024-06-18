// SPDX-License-Identifier: CC-BY-4.0

mod args;
mod config;
mod errors;
mod formats;
mod match_system;
mod matcher;
mod menu;
mod options;
mod output_processor;
mod searchers;
mod writer;
use clap::ArgMatches;
use config::Config;
use errors::{bail, Message};
use match_system::Matches;
use menu::Menu;
use output_processor::process_results;
use searchers::Searchers;
use std::ffi::OsString;
use std::io::{stdout, StdoutLock};
use std::process::Command;
use std::sync::OnceLock;
use writer::write_results;

// TODO option to configure different colors
// TODO add notarizing mac so exec can be used without needing to open from finder
// TODO support for searching PDFs maybe
// TODO --plugin option that starts the program with alternate screen which prompts the user for their args
// TODO nvim plugin to open a popup window, select a match to open in $EDITOR
// TODO tmux plugin to open a popup window, select a match to open in $EDITOR
// TODO zellij plugin to open a popup window, select a match to open in $EDITOR

static CONFIG: OnceLock<Config> = OnceLock::new();

fn config() -> &'static Config {
    CONFIG.get().unwrap()
}

fn main() {
    let matches = Config::get_matches();
    let colors = Config::use_color(&matches);
    run(matches, colors).unwrap_or_else(|e| {
        eprintln!("{} {}", formats::error_prefix(colors), e);
        std::process::exit(1);
    });
}

fn run(matches: ArgMatches, colors: bool) -> Result<(), Message> {
    let (c, searcher_path) = Config::get_config(matches, colors)?;
    CONFIG.set(c).ok().unwrap();

    let matches: Option<Matches>;
    if config().tree || searcher_path.is_none() {
        matches = matcher::search()?;
    } else {
        matches = get_matches_from_cmd(searcher_path.unwrap())?;
    }

    if matches.is_none() {
        return Ok(());
    }

    let mut out: StdoutLock = stdout().lock();
    if config().menu {
        Menu::enter(&mut out, matches.unwrap()).map_err(|e| bail!("{}", e.to_string()))?;
    } else {
        write_results(&mut out, &matches.unwrap(), None).map_err(|e| bail!("{}", e.to_string()))?;
    }

    Ok(())
}

fn get_matches_from_cmd(searcher_path: OsString) -> Result<Option<Matches>, Message> {
    let mut cmd: Command = Searchers::generate_command(searcher_path)?;

    let output = cmd.output().map_err(|e| {
        bail!(
            "searcher `{}` didn't run message, `{}`",
            cmd.get_program().to_string_lossy().to_string(),
            e.to_string()
        )
    })?;

    if !output.status.success() && output.stderr.len() > 0 {
        return Err(bail!(
            "{} had errors:\n{}",
            cmd.get_program().to_string_lossy().to_string(),
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }
    let results: Vec<u8> = output.stdout;

    process_results(results)
}
