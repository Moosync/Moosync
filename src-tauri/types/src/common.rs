use std::collections::HashSet;

use serde::{Deserialize, Deserializer};

pub trait SearchByTerm {
    fn search_by_term(term: Option<String>) -> Self;
}

pub trait BridgeUtils {
    fn insert_value(entity: String, song: String) -> Self;
}

pub trait Unique<T> {
    fn unique(&mut self);
}

impl<T> Unique<T> for Vec<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    fn unique(self: &mut Vec<T>) {
        let mut seen: HashSet<T> = HashSet::new();
        self.retain(|item| seen.insert(item.clone()));
    }
}

#[tracing::instrument(level = "trace", skip(deserializer))]
pub fn deserialize_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    match T::deserialize(deserializer) {
        Ok(value) => Ok(value),
        Err(_) => Ok(T::default()),
    }
}
