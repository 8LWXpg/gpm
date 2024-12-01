//! Shared utilities for configuration handling.

use anyhow::Result;
use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::io::{self, Write};

#[macro_export]
macro_rules! tabwriter {
    ($fmt:expr, $($arg:tt)*) => {
        {
            let mut tw = tabwriter::TabWriter::new(vec![]);
            write!(&mut tw, $fmt, $($arg)*).expect("Failed to write to TabWriter");
            tw.flush().expect("Failed to flush TabWriter");
            println!("{}", String::from_utf8(tw.into_inner().unwrap()).unwrap());
        }
    };
}

#[macro_export]
macro_rules! print_message {
    ($symbol:expr, $color:ident, $msg:expr) => {
        $crate::tabwriter!("{} {}", $symbol.$color().bold(), $msg)
    };
    ($symbol:expr, $color:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::tabwriter!("{} {}", $symbol.$color().bold(), format!($fmt, $($arg)*))
    };
}

/// print message for adding an item.
#[macro_export]
macro_rules! add {
    ($($arg:tt)*) => {
        $crate::print_message!("+", bright_green, $($arg)*)
    };
}

/// print message for cloning an item.
#[macro_export]
macro_rules! clone {
    ($($arg:tt)*) => {
        $crate::print_message!("=", bright_blue, $($arg)*)
    };
}

/// print message for removing an item.
#[macro_export]
macro_rules! remove {
    ($($arg:tt)*) => {
        $crate::print_message!("-", bright_red, $($arg)*)
    };
}

pub fn sort_keys<T, S>(value: &HashMap<String, T>, serializer: S) -> Result<S::Ok, S::Error>
where
	T: Serialize,
	S: Serializer,
{
	value
		.iter()
		.collect::<BTreeMap<_, _>>()
		.serialize(serializer)
}

/// prompt the user for a yes/no response.
///
/// # Arguments
/// `message` - The prompt to display, appended with " [y/N]: "
pub fn prompt(message: &str) -> Result<bool> {
	let mut input = String::new();
	print!("{message} [y/N]: ");
	io::stdout().flush()?; // Make sure the prompt is immediately displayed
	io::stdin().read_line(&mut input)?;
	match input.trim().to_lowercase().as_str() {
		"y" => Ok(true),
		"n" => Ok(false),
		_ => Ok(false),
	}
}
