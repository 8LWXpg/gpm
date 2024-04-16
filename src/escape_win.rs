//! Custom escape rules for Windows...

use std::{ffi::OsStr, os::windows::process::CommandExt, process::Command};

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
