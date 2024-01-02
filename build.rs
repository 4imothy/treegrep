use clap::ValueEnum;
use clap_complete::{generate_to, Shell};
use std::env;
use std::io::Error;

include!("src/args.rs");

fn main() -> Result<(), Error> {
    let _outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = generate_command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, names::BIN_NAME, "completions")?;
    }

    Ok(())
}
