use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::holder::{HolderData, PermissionHolder};

pub const DEFAULT_GROUP: &str = "default";

pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 48
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(default)]
    weight: i32,
    #[serde(flatten)]
    data: HolderData,
}

impl Group {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_lowercase(),
            display_name: None,
            weight: 0,
            data: HolderData::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }

    pub fn set_display_name(&mut self, display_name: Option<String>) {
        self.display_name = display_name;
    }

    pub fn weight(&self) -> i32 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: i32) {
        self.weight = weight;
    }

    pub fn is_default(&self) -> bool {
        self.name == DEFAULT_GROUP
    }
}

impl PermissionHolder for Group {
    fn identifier(&self) -> &str {
        &self.name
    }

    fn data(&self) -> &HolderData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut HolderData {
        &mut self.data
    }
}

#[derive(Clone, Debug, Default)]
pub struct GroupRegistry {
    groups: BTreeMap<String, Group>,
}

impl GroupRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, group: Group) -> Option<Group> {
        self.groups.insert(group.name().to_owned(), group)
    }

    pub fn remove(&mut self, name: &str) -> Option<Group> {
        self.groups.remove(&name.to_lowercase())
    }

    pub fn get(&self, name: &str) -> Option<&Group> {
        self.groups.get(&name.to_lowercase())
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.groups.get_mut(&name.to_lowercase())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.groups.contains_key(&name.to_lowercase())
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.groups.keys().map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Group> {
        self.groups.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        self.groups.values_mut()
    }

    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn clear(&mut self) {
        self.groups.clear();
    }
}
