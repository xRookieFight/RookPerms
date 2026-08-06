use serde::{Deserialize, Serialize};

use crate::context::ContextSet;

pub const WILDCARD: &str = "*";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    key: String,
    value: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expiry: Option<u64>,
    #[serde(default, skip_serializing_if = "ContextSet::is_empty")]
    context: ContextSet,
}

impl Node {
    pub fn new(key: impl AsRef<str>, value: bool) -> Self {
        Self {
            key: key.as_ref().trim().to_lowercase(),
            value,
            expiry: None,
            context: ContextSet::new(),
        }
    }

    pub fn with_expiry(mut self, expiry: Option<u64>) -> Self {
        self.expiry = expiry;
        self
    }

    pub fn with_context(mut self, context: ContextSet) -> Self {
        self.context = context;
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> bool {
        self.value
    }

    pub fn expiry(&self) -> Option<u64> {
        self.expiry
    }

    pub fn context(&self) -> &ContextSet {
        &self.context
    }

    pub fn is_expired(&self, now: u64) -> bool {
        self.expiry.is_some_and(|expiry| expiry <= now)
    }

    pub fn is_temporary(&self) -> bool {
        self.expiry.is_some()
    }

    pub fn applies_to(&self, query: &ContextSet, now: u64) -> bool {
        !self.is_expired(now) && self.context.is_satisfied_by(query)
    }

    pub fn is_wildcard(&self) -> bool {
        self.key == WILDCARD || self.key.ends_with(".*")
    }

    pub fn matches(&self, target: &str) -> bool {
        if self.key == WILDCARD {
            return true;
        }
        match self.key.strip_suffix(".*") {
            Some(prefix) => target == prefix || target.starts_with(&format!("{prefix}.")),
            None => self.key == target,
        }
    }

    pub fn same_identity(&self, key: &str, context: &ContextSet) -> bool {
        self.key == key.trim().to_lowercase() && &self.context == context
    }
}

pub fn normalize_key(key: &str) -> String {
    key.trim().to_lowercase()
}

pub fn is_valid_key(key: &str) -> bool {
    let key = key.trim();
    !key.is_empty()
        && key.len() <= 128
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '*' | ':' | '/'))
}
