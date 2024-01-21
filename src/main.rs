// SPDX-License-Identifier: CC-BY-4.0

mod args;
mod config;
mod errors;
mod formats;
mod match_system;
mod matcher;
mod menu;
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
use writer::write_results;

// TODO add notarizing mac to exec can be used without needing to open from finder
// TODO need to throw error when trying to do unsupported things using tgrep like pcre2
// TODO do the --files to stop searching a file when a match is found or use the
// --files-with-matches flag for ripgrep
// TODO vim plugin put it in the plugin folder and run cargo build --release and get the dir

fn main() {
    let matches = Config::get_matches();
    let colors = Config::get_colors(&matches);
    run(matches, colors).unwrap_or_else(|e| {
        eprintln!("{} {}", formats::error_prefix(colors), e);
        std::process::exit(1);
    });
}

fn run(matches: ArgMatches, colors: bool) -> Result<(), Message> {
    let (config, starter) = Config::get_config(matches, colors)?;

    let matches: Option<Matches>;
    let mut out: StdoutLock = stdout().lock();
    if let Some(s) = starter {
        matches = get_matches_from_cmd(s, &config)?;
    } else {
        matches = matcher::search(&config)?;
    }

    if matches.is_none() {
        return Ok(());
    }

    if config.menu {
        Menu::enter(&mut out, matches.unwrap(), &config).map_err(|e| bail!("{}", e.to_string()))?;
    } else {
        write_results(&mut out, &matches.unwrap(), &config, None)
            .map_err(|e| bail!("{}", e.to_string()))?;
    }

    Ok(())
}

fn get_matches_from_cmd(starter: OsString, config: &Config) -> Result<Option<Matches>, Message> {
    let mut cmd: Command = Searchers::generate_command(config, starter)?;

    let output = cmd.output().map_err(|e| {
        bail!(
            "searcher `{}` didn't run message, `{}`",
            cmd.get_program().to_string_lossy().to_string(),
            e.to_string()
        )
    })?;

    if !output.status.success() && output.stderr.len() > 0 {
        return Err(bail!(
            "{} had errors, {}",
            cmd.get_program().to_string_lossy().to_string(),
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }
    let results: Vec<u8> = output.stdout;

    process_results(results, &config)
}
