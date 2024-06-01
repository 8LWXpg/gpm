//! Shared utilities for configuration handling.

use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

/// Message for adding an item.
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

#[macro_export]
macro_rules! add {
    ($($arg:tt)*) => {
        $crate::print_message!("+", bright_green, $($arg)*)
    };
}

#[macro_export]
macro_rules! clone {
    ($($arg:tt)*) => {
        $crate::print_message!("=", bright_blue, $($arg)*)
    };
}

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
