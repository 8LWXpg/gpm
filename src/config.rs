//! Shared utilities for configuration handling.

use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

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
