// SPDX-License-Identifier: MIT

use clap::ValueEnum;
use clap_complete::{generate_to, Shell};
use std::env;
use std::fs::create_dir_all;
use std::io::Error;
use std::path::Path;

include!("src/args.rs");
const DEBUG: bool = cfg!(debug_assertions);

fn main() -> Result<(), Error> {
    if !DEBUG {
        let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let completions_dir_path = Path::new(&project_root).join("completions");
        if !completions_dir_path.exists() {
            create_dir_all(&completions_dir_path).unwrap_or_else(|e| panic!("{}", e));
        }

        let mut cmd = generate_command();
        for &shell in Shell::value_variants() {
            generate_to(shell, &mut cmd, names::TREEGREP_BIN, &completions_dir_path)?;
        }
    }

    Ok(())
}
