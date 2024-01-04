// SPDX-License-Identifier: CC-BY-4.0

use clap::ValueEnum;
use clap_complete::{generate_to, Shell};
use std::env;
use std::fs::create_dir_all;
use std::io::Error;
use std::path::Path;

include!("src/args.rs");

fn main() -> Result<(), Error> {
    let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let completions_dir_path = Path::new(&project_root).join("completions");
    if !completions_dir_path.exists() {
        match create_dir_all(&completions_dir_path) {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e.to_string());
            }
        }
    }

    let mut cmd = generate_command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, names::BIN_NAME, &completions_dir_path)?;
    }

    Ok(())
}
