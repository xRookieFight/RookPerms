use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub const CONTEXT_WORLD: &str = "world";
pub const CONTEXT_SERVER: &str = "server";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContextSet {
    entries: BTreeMap<String, BTreeSet<String>>,
}

impl ContextSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.add(key, value);
        self
    }

    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries
            .entry(key.into().to_lowercase())
            .or_default()
            .insert(value.into().to_lowercase());
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(&key.to_lowercase()).is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.values().map(BTreeSet::len).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &BTreeSet<String>)> {
        self.entries
            .iter()
            .map(|(key, values)| (key.as_str(), values))
    }

    pub fn is_satisfied_by(&self, query: &Self) -> bool {
        self.entries.iter().all(|(key, values)| {
            query
                .entries
                .get(key)
                .is_some_and(|available| values.iter().any(|value| available.contains(value)))
        })
    }
}

impl std::fmt::Display for ContextSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("global");
        }
        let rendered: Vec<String> = self
            .entries
            .iter()
            .flat_map(|(key, values)| values.iter().map(move |value| format!("{key}={value}")))
            .collect();
        f.write_str(&rendered.join(", "))
    }
}
