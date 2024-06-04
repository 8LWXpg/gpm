//! Custom escape rules for Windows...

use std::ffi::OsStr;
use std::os::windows::process::CommandExt;
use std::process::Command;

/// What this does is basically wrap the string in single quotes and escape any double quote with a backslash.
///
/// Definitely will have unexpected results if the string contains a single quote, but I'm leaving that for now.
///
/// Let's just hope it will be easier to escape than CMD.
///
/// https://stackoverflow.com/a/59681993
pub fn escape_pwsh(s: &str) -> String {
    format!("'{}'", s.replace('"', "\\\""))
}

pub trait EscapePwsh {
    fn arg_pwsh<S>(&mut self, arg: S) -> &mut Command
    where
        S: AsRef<OsStr>;

    fn args_pwsh<I, S>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;
}

impl EscapePwsh for Command {
    fn arg_pwsh<S>(&mut self, arg: S) -> &mut Command
    where
        S: AsRef<OsStr>,
    {
        self.raw_arg(escape_pwsh(arg.as_ref().to_string_lossy().as_ref()))
    }

    fn args_pwsh<I, S>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.arg_pwsh(arg);
        }
        self
    }
}
