// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

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
