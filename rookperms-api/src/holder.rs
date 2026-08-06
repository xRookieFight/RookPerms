use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::context::ContextSet;
use crate::meta::WeightedValue;
use crate::node::{Node, normalize_key};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HolderData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    nodes: Vec<Node>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    parents: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prefix: Option<WeightedValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    suffix: Option<WeightedValue>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    meta: BTreeMap<String, String>,
}

impl HolderData {
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn set_node(&mut self, node: Node) -> bool {
        let replaced = self
            .nodes
            .iter()
            .position(|existing| existing.same_identity(node.key(), node.context()));
        match replaced {
            Some(index) => {
                if self.nodes[index] == node {
                    return false;
                }
                self.nodes[index] = node;
            }
            None => self.nodes.push(node),
        }
        true
    }

    pub fn unset_node(&mut self, key: &str, context: &ContextSet) -> bool {
        let key = normalize_key(key);
        let before = self.nodes.len();
        self.nodes.retain(|node| !node.same_identity(&key, context));
        before != self.nodes.len()
    }

    pub fn clear_nodes(&mut self) -> bool {
        let had_nodes = !self.nodes.is_empty();
        self.nodes.clear();
        had_nodes
    }

    pub fn purge_expired(&mut self, now: u64) -> bool {
        let before = self.nodes.len();
        self.nodes.retain(|node| !node.is_expired(now));
        before != self.nodes.len()
    }

    pub fn parents(&self) -> &[String] {
        &self.parents
    }

    pub fn add_parent(&mut self, parent: impl AsRef<str>) -> bool {
        let parent = parent.as_ref().to_lowercase();
        if self.parents.contains(&parent) {
            return false;
        }
        self.parents.push(parent);
        true
    }

    pub fn remove_parent(&mut self, parent: &str) -> bool {
        let parent = parent.to_lowercase();
        let before = self.parents.len();
        self.parents.retain(|existing| existing != &parent);
        before != self.parents.len()
    }

    pub fn set_parents(&mut self, parents: Vec<String>) {
        self.parents = parents.into_iter().map(|it| it.to_lowercase()).collect();
    }

    pub fn prefix(&self) -> Option<&WeightedValue> {
        self.prefix.as_ref()
    }

    pub fn set_prefix(&mut self, prefix: Option<WeightedValue>) {
        self.prefix = prefix;
    }

    pub fn suffix(&self) -> Option<&WeightedValue> {
        self.suffix.as_ref()
    }

    pub fn set_suffix(&mut self, suffix: Option<WeightedValue>) {
        self.suffix = suffix;
    }

    pub fn meta(&self) -> &BTreeMap<String, String> {
        &self.meta
    }

    pub fn set_meta(&mut self, key: impl AsRef<str>, value: impl Into<String>) {
        self.meta.insert(key.as_ref().to_lowercase(), value.into());
    }

    pub fn remove_meta(&mut self, key: &str) -> bool {
        self.meta.remove(&key.to_lowercase()).is_some()
    }
}

pub trait PermissionHolder {
    fn identifier(&self) -> &str;

    fn data(&self) -> &HolderData;

    fn data_mut(&mut self) -> &mut HolderData;

    fn nodes(&self) -> &[Node] {
        self.data().nodes()
    }

    fn set_node(&mut self, node: Node) -> bool {
        self.data_mut().set_node(node)
    }

    fn unset_node(&mut self, key: &str, context: &ContextSet) -> bool {
        self.data_mut().unset_node(key, context)
    }

    fn parents(&self) -> &[String] {
        self.data().parents()
    }

    fn add_parent(&mut self, parent: impl AsRef<str>) -> bool {
        self.data_mut().add_parent(parent)
    }

    fn remove_parent(&mut self, parent: &str) -> bool {
        self.data_mut().remove_parent(parent)
    }
}
