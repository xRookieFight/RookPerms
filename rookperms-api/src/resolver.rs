use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::context::ContextSet;
use crate::group::GroupRegistry;
use crate::holder::{HolderData, PermissionHolder};
use crate::meta::WeightedValue;
use crate::node::WILDCARD;
use crate::user::User;

#[derive(Clone, Debug, Default)]
pub struct QueryContext {
    contexts: ContextSet,
    now: u64,
}

impl QueryContext {
    pub fn new(contexts: ContextSet, now: u64) -> Self {
        Self { contexts, now }
    }

    pub fn contexts(&self) -> &ContextSet {
        &self.contexts
    }

    pub fn now(&self) -> u64 {
        self.now
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResolvedPermissions {
    permissions: BTreeMap<String, bool>,
    meta: BTreeMap<String, String>,
    prefix: Option<String>,
    suffix: Option<String>,
    inherited_groups: Vec<String>,
    weight: i32,
    valid_until: Option<u64>,
}

impl ResolvedPermissions {
    pub fn check(&self, node: &str) -> Option<bool> {
        let node = node.trim().to_lowercase();
        if let Some(value) = self.permissions.get(&node) {
            return Some(*value);
        }

        let mut cursor = node.as_str();
        while let Some(index) = cursor.rfind('.') {
            cursor = &cursor[..index];
            if let Some(value) = self.permissions.get(&format!("{cursor}.*")) {
                return Some(*value);
            }
        }

        self.permissions.get(WILDCARD).copied()
    }

    pub fn permissions(&self) -> &BTreeMap<String, bool> {
        &self.permissions
    }

    pub fn granted(&self) -> impl Iterator<Item = &str> {
        self.permissions
            .iter()
            .filter(|(_, value)| **value)
            .map(|(key, _)| key.as_str())
    }

    pub fn meta(&self) -> &BTreeMap<String, String> {
        &self.meta
    }

    pub fn meta_value(&self, key: &str) -> Option<&str> {
        self.meta.get(&key.to_lowercase()).map(String::as_str)
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    pub fn suffix(&self) -> Option<&str> {
        self.suffix.as_deref()
    }

    pub fn inherited_groups(&self) -> &[String] {
        &self.inherited_groups
    }

    pub fn weight(&self) -> i32 {
        self.weight
    }

    pub fn valid_until(&self) -> Option<u64> {
        self.valid_until
    }
}

struct InheritedHolder<'a> {
    name: String,
    depth: usize,
    weight: i32,
    data: &'a HolderData,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PermissionResolver;

impl PermissionResolver {
    pub fn resolve(
        user: &User,
        groups: &GroupRegistry,
        query: &QueryContext,
    ) -> ResolvedPermissions {
        let chain = Self::collect_chain(user, groups);

        let mut resolved = ResolvedPermissions {
            weight: chain.iter().map(|holder| holder.weight).max().unwrap_or(0),
            inherited_groups: chain.iter().map(|holder| holder.name.clone()).collect(),
            ..ResolvedPermissions::default()
        };

        let mut prefix: Option<(i32, usize, String)> = None;
        let mut suffix: Option<(i32, usize, String)> = None;

        for (index, holder) in chain
            .iter()
            .map(|it| it.data)
            .chain([user.data()])
            .enumerate()
        {
            Self::apply_nodes(holder, query, &mut resolved);
            Self::apply_meta(holder, &mut resolved.meta);
            Self::elect(holder.prefix(), index, &mut prefix);
            Self::elect(holder.suffix(), index, &mut suffix);
        }

        resolved.prefix = prefix.map(|(_, _, value)| value);
        resolved.suffix = suffix.map(|(_, _, value)| value);
        resolved
    }

    fn collect_chain<'a>(user: &User, groups: &'a GroupRegistry) -> Vec<InheritedHolder<'a>> {
        let mut visited: BTreeSet<String> = BTreeSet::new();
        let mut queue: VecDeque<(String, usize)> = user
            .parents()
            .iter()
            .map(|parent| (parent.clone(), 1))
            .collect();
        let mut chain: Vec<InheritedHolder<'a>> = Vec::new();

        while let Some((name, depth)) = queue.pop_front() {
            if !visited.insert(name.clone()) {
                continue;
            }
            let Some(group) = groups.get(&name) else {
                continue;
            };
            for parent in group.parents() {
                queue.push_back((parent.clone(), depth + 1));
            }
            chain.push(InheritedHolder {
                name: group.name().to_owned(),
                depth,
                weight: group.weight(),
                data: group.data(),
            });
        }

        chain.sort_by(|left, right| {
            right
                .depth
                .cmp(&left.depth)
                .then(left.weight.cmp(&right.weight))
                .then(left.name.cmp(&right.name))
        });
        chain
    }

    fn apply_nodes(holder: &HolderData, query: &QueryContext, resolved: &mut ResolvedPermissions) {
        for node in holder.nodes() {
            if !node.applies_to(query.contexts(), query.now()) {
                continue;
            }
            resolved
                .permissions
                .insert(node.key().to_owned(), node.value());
            if let Some(expiry) = node.expiry() {
                resolved.valid_until = Some(
                    resolved
                        .valid_until
                        .map_or(expiry, |current| current.min(expiry)),
                );
            }
        }
    }

    fn apply_meta(holder: &HolderData, meta: &mut BTreeMap<String, String>) {
        for (key, value) in holder.meta() {
            meta.insert(key.clone(), value.clone());
        }
    }

    fn elect(
        candidate: Option<&WeightedValue>,
        index: usize,
        current: &mut Option<(i32, usize, String)>,
    ) {
        let Some(candidate) = candidate else {
            return;
        };
        let wins = current.as_ref().is_none_or(|(priority, order, _)| {
            candidate.priority() > *priority
                || (candidate.priority() == *priority && index >= *order)
        });
        if wins {
            *current = Some((candidate.priority(), index, candidate.value().to_owned()));
        }
    }
}
