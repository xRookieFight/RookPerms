use serde::{Deserialize, Serialize};

pub const META_PREFIX: &str = "prefix";
pub const META_SUFFIX: &str = "suffix";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeightedValue {
    value: String,
    #[serde(default)]
    priority: i32,
}

impl WeightedValue {
    pub fn new(value: impl Into<String>, priority: i32) -> Self {
        Self {
            value: value.into(),
            priority,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }
}
