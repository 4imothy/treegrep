// SPDX-License-Identifier: CC-BY-4.0

use crate::formats::error_prefix;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;

pub enum Errors {
    NoSupportedBinary { info: String },
    FailedToFindGivenSearcher { chosen: String },
    RunFailed { info: String, exe_name: String },
    ExeHasErrors { info: String, exe_name: String },
    FailedToGetCWD,
    FailedToCanonicalizePath { info: String },
    FailedToGetName { info: OsString },
    FailedToGetParent { info: OsString },
    FailedToFindPath { info: String },
    OptionIsntUsize { option: String, given: String },
    InvalidSearcherExe { info: String, supported: String },
    IOError { info: String },
    ProcessingInternalSearcherAsExternal,
    InvalidRegex { regex: String },
    InvalidJson { serde_json_mes: String },
}

const SUBMIT_ISSUE: &str = "Please submit an issue, github.com/4imothy/treegrep/issues";

macro_rules! write_error {
    ($f:expr, $($arg:tt)*) => {
        write!($f, "{} {}", error_prefix(), format_args!($($arg)*))
    };
}

impl Error for Errors {}

impl fmt::Debug for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Errors::NoSupportedBinary { info } => {
                write_error!(f, "no supported searcher found, tried `{info}`")
            }
            Errors::ExeHasErrors { info, exe_name } => {
                write_error!(f, "{exe_name} had errors, {info}")
            }
            Errors::RunFailed { info, exe_name } => {
                write_error!(f, "Exe `{exe_name}` didn't run message, `{info}`")
            }
            Errors::FailedToGetCWD => {
                write_error!(f, "failed to get current working directory")
            }
            Errors::FailedToGetName { info } => {
                write_error!(f, "failed to get name of `{}`", info.to_string_lossy())
            }
            Errors::FailedToGetParent { info } => {
                write_error!(
                    f,
                    "failed to get parent to path `{}`",
                    info.to_string_lossy()
                )
            }
            Errors::OptionIsntUsize { option, given } => {
                write_error!(
                    f,
                    "failed to parse `{given}` to a usize for option `{option}`"
                )
            }
            Errors::FailedToFindPath { info } => write_error!(f, "failed to find path {info}"),
            Errors::InvalidSearcherExe { info, supported } => {
                write_error!(f, "searcher {info} is invalid, tried {supported}")
            }
            Errors::ProcessingInternalSearcherAsExternal => write_error!(
                f,
                "tried to use external command when using the treegrep searcher {SUBMIT_ISSUE}",
            ),
            Errors::FailedToFindGivenSearcher { chosen } => {
                write_error!(f, "could not find searcher `{chosen}`")
            }
            Errors::IOError { info } => write_error!(f, "IO Error `{info}`"),
            Errors::FailedToCanonicalizePath { info } => {
                write_error!(f, "failed to canonicalize given path `{info}`")
            }
            Errors::InvalidRegex { regex } => {
                write_error!(f, "regex expression `{regex}` is invalid")
            }
            Errors::InvalidJson { serde_json_mes } => {
                write_error!(f, "json output is invalid: `{serde_json_mes}` is invalid")
            }
        }
    }
}
