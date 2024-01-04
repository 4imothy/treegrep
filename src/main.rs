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
use config::Config;
use errors::Errors;
use match_system::Matches;
use menu::Menu;
use output_processor::process_results;
use searchers::Searchers;
use std::ffi::OsString;
use std::io::{stdout, StdoutLock};
use std::process::Command;
use writer::write_results;

// TODO add notarizing mac to exec can be used without needing to open from finder
// TODO come up with a good way to test
//   Run a something on a testing dir which has links and have a predetirmed output that is run with a working version and test against it everything and assert that the things are equal
// TODO do the --files to stop searching a file when a match is found or use the
// --files-with-matches flag for ripgrep
// TODO vim plugin or something

fn main() {
    run().unwrap_or_else(|e| exit_error(e));
}

fn run() -> Result<(), Errors> {
    let (config, starter) = Config::get_config().unwrap_or_else(|e| exit_error(e));
    let matches: Option<Matches>;
    let mut out: StdoutLock = stdout().lock();
    if let Some(s) = starter {
        matches = get_matches_from_cmd(s, &config).unwrap_or_else(|e| exit_error(e));
    } else {
        matches = matcher::search(&config)?;
    }

    if matches.is_none() {
        return Ok(());
    }

    if config.menu {
        Menu::draw(&mut out, matches.unwrap(), &config).map_err(|e| Errors::IOError {
            info: e.to_string(),
        })?;
    } else {
        write_results(&mut out, &matches.unwrap(), &config).map_err(|e| Errors::IOError {
            info: e.to_string(),
        })?;
    }

    Ok(())
}

fn get_matches_from_cmd(starter: OsString, config: &Config) -> Result<Option<Matches>, Errors> {
    let mut cmd: Command =
        Searchers::generate_command(config, starter).unwrap_or_else(|e| exit_error(e));

    let output = cmd.output().unwrap_or_else(|e| {
        exit_error(Errors::RunFailed {
            info: e.to_string(),
            exe_name: cmd.get_program().to_string_lossy().to_string(),
        })
    });
    if !output.status.success() && output.stderr.len() > 0 {
        exit_error(Errors::ExeHasErrors {
            info: String::from_utf8_lossy(&output.stderr).to_string(),
            exe_name: cmd.get_program().to_string_lossy().to_string(),
        });
    }
    let results: Vec<u8> = output.stdout;

    process_results(results, &config)
}

fn exit_error(e: impl std::error::Error) -> ! {
    eprintln!("{}", e);
    std::process::exit(1);
}
