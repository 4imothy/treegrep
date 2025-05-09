// SPDX-License-Identifier: MIT

mod args;
mod config;
mod errors;
mod formats;
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
use clap_complete::{aot::Shell, generate};
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

const LOG: bool = cfg!(debug_assertions);

fn config() -> &'static Config {
    CONFIG.get().unwrap()
}

fn main() {
    if LOG {
        log::set_panic_hook();
    }
    let matches = Config::get_matches();
    let (bold, colors) = Config::get_styling(&matches);
    run(matches, bold, colors).unwrap_or_else(|e| {
        eprintln!("{} {}", formats::error_prefix(bold, colors), e);
        std::process::exit(1);
    });
}

fn run(matches: ArgMatches, bold: bool, colors: bool) -> Result<(), Message> {
    if gen_completions_if_needed(&matches)? {
        return Ok(());
    }
    let (c, searcher_path) = Config::get_config(matches, bold, colors)?;
    CONFIG.set(c).ok().unwrap();

    let matches: Option<Matches>;
    if config().just_files || searcher_path.is_none() {
        matches = matcher::search()?;
    } else {
        matches = get_matches_from_cmd(searcher_path.unwrap())?;
    }

    if matches.is_none() {
        return Ok(());
    }

    let mut out: StdoutLock = stdout().lock();
    let m = matches.unwrap();
    let mut path_ids = config().menu.then(|| Vec::<usize>::new());
    let lines: Vec<Box<dyn Entry>> = matches_to_display_lines(&m, path_ids.as_mut());

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

    if !output.status.success() && output.stderr.len() > 0 {
        return Err(mes!(
            "{} had errors:\n{}",
            cmd.get_program().to_string_lossy(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let results: Vec<u8> = output.stdout;

    process_results(results)
}

fn gen_completions_if_needed(matches: &ArgMatches) -> Result<bool, Message> {
    if let Some(shell) = matches.get_one::<Shell>(args::COMPLETIONS.id) {
        let mut cmd = args::generate_command();
        let mut fd = std::io::stdout();
        match shell {
            Shell::Bash => generate(Shell::Bash, &mut cmd, args::names::TREEGREP_BIN, &mut fd),
            Shell::Zsh => generate(Shell::Zsh, &mut cmd, args::names::TREEGREP_BIN, &mut fd),
            Shell::Elvish => generate(Shell::Bash, &mut cmd, args::names::TREEGREP_BIN, &mut fd),
            Shell::PowerShell => generate(
                Shell::PowerShell,
                &mut cmd,
                args::names::TREEGREP_BIN,
                &mut fd,
            ),
            Shell::Fish => generate(Shell::Fish, &mut cmd, args::names::TREEGREP_BIN, &mut fd),
            _ => return Err(mes!("cannot generate completions for {shell}")),
        }
        return Ok(true);
    }
    return Ok(false);
}
