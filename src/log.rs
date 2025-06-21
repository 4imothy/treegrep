// SPDX-License-Identifier: MIT

use std::fs;
use std::panic;
use std::sync::{Mutex, OnceLock};

pub static LOG: OnceLock<Mutex<fs::File>> = OnceLock::new();

pub fn gen_log() -> Mutex<fs::File> {
    Mutex::new(fs::File::create("log").unwrap())
}

#[allow(unused_macros)]
macro_rules! log {
    ($($arg:tt)*) => {
        use std::io::Write;
        writeln!($crate::log::LOG.get_or_init(||
                $crate::log::gen_log()).lock().unwrap(),
                $($arg)*).unwrap();
    };
}

pub fn set_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        log!("{}", info);
        default_hook(info);
    }));
}

#[allow(unused_imports)]
pub(crate) use log;
