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
    FeatureNotSupported { searcher: String, feature: String },
    ProcessingInternalSearcherAsExternal,
    BadlyFormattedLine { line: String },
    InvalidRegex { regex: String },
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
                write_error!(f, "no supported searcher found, tried `{}`", info)
            }
            Errors::ExeHasErrors { info, exe_name } => {
                write_error!(f, "{} had errors, {}", exe_name, info)
            }
            Errors::RunFailed { info, exe_name } => {
                write_error!(f, "Exe `{}` didn't run message, `{}`", exe_name, info)
            }
            Errors::FailedToGetCWD => {
                write_error!(f, "failed to get current working directory")
            }
            Errors::FailedToGetName { info } => {
                write_error!(f, "failed to get name of `{}`", info.to_string_lossy())
            }
            Errors::FailedToGetParent { info } => {
                write_error!(f, "failed to get parent to path {}", info.to_string_lossy())
            }
            Errors::OptionIsntUsize { option, given } => {
                write_error!(
                    f,
                    "failed to parse `{}` to a usize for option `{}`",
                    given,
                    option
                )
            }
            Errors::FailedToFindPath { info } => write_error!(f, "failed to find path {}", info),
            Errors::InvalidSearcherExe { info, supported } => {
                write_error!(f, "searcher {} is invalid, tried {}", info, supported)
            }
            Errors::ProcessingInternalSearcherAsExternal => write_error!(
                f,
                "tried to use external command when using the treegrep searcher {}",
                SUBMIT_ISSUE
            ),
            Errors::FailedToFindGivenSearcher { chosen } => {
                write_error!(f, "could not find searcher `{}`", chosen)
            }
            Errors::IOError { info } => write_error!(f, "IO Error `{}`", info),
            Errors::FailedToCanonicalizePath { info } => {
                write_error!(f, "failed to canonicalize given path `{}`", info)
            }
            Errors::FeatureNotSupported { searcher, feature } => {
                write_error!(f, "searcher `{}` does not support {}", searcher, feature)
            }
            Errors::BadlyFormattedLine { line } => {
                write_error!(f, "output line `{}` is badly formatted", line)
            }
            Errors::InvalidRegex { regex } => {
                write_error!(f, "regex expression `{}` is invalid", regex)
            }
        }
    }
}
