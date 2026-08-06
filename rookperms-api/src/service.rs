use std::collections::BTreeMap;

use crate::context::ContextSet;
use crate::error::{Result, RookPermsError};
use crate::group::{DEFAULT_GROUP, Group, GroupRegistry};
use crate::holder::PermissionHolder;
use crate::id::PlayerId;
use crate::resolver::{PermissionResolver, QueryContext, ResolvedPermissions};
use crate::storage::PermissionStorage;
use crate::user::User;

struct CachedPermissions {
    contexts: ContextSet,
    resolved: ResolvedPermissions,
}

impl CachedPermissions {
    fn is_valid_for(&self, query: &QueryContext) -> bool {
        self.contexts == *query.contexts()
            && self
                .resolved
                .valid_until()
                .is_none_or(|deadline| deadline > query.now())
    }
}

pub struct PermissionService {
    storage: Box<dyn PermissionStorage>,
    groups: GroupRegistry,
    users: BTreeMap<PlayerId, User>,
    cache: BTreeMap<PlayerId, CachedPermissions>,
}

impl PermissionService {
    pub fn new(storage: Box<dyn PermissionStorage>) -> Self {
        Self {
            storage,
            groups: GroupRegistry::new(),
            users: BTreeMap::new(),
            cache: BTreeMap::new(),
        }
    }

    pub fn load(&mut self) -> Result<()> {
        self.groups.clear();
        self.users.clear();
        self.cache.clear();

        for group in self.storage.load_groups()? {
            self.groups.insert(group);
        }

        if !self.groups.contains(DEFAULT_GROUP) {
            let group = Group::new(DEFAULT_GROUP);
            self.storage.save_group(&group)?;
            self.groups.insert(group);
        }

        Ok(())
    }

    pub fn groups(&self) -> &GroupRegistry {
        &self.groups
    }

    pub fn group(&self, name: &str) -> Result<&Group> {
        self.groups
            .get(name)
            .ok_or_else(|| RookPermsError::GroupNotFound(name.to_lowercase()))
    }

    pub fn create_group(&mut self, name: &str) -> Result<()> {
        if self.groups.contains(name) {
            return Err(RookPermsError::GroupAlreadyExists(name.to_lowercase()));
        }
        let group = Group::new(name);
        self.storage.save_group(&group)?;
        self.groups.insert(group);
        Ok(())
    }

    pub fn delete_group(&mut self, name: &str) -> Result<()> {
        let name = name.to_lowercase();
        if name == DEFAULT_GROUP {
            return Err(RookPermsError::ProtectedGroup(name));
        }
        if !self.groups.contains(&name) {
            return Err(RookPermsError::GroupNotFound(name));
        }

        self.storage.delete_group(&name)?;
        self.groups.remove(&name);

        let orphaned: Vec<String> = self
            .groups
            .iter_mut()
            .filter(|group| group.parents().contains(&name))
            .map(|group| {
                group.remove_parent(&name);
                group.name().to_owned()
            })
            .collect();
        for group in orphaned {
            if let Some(group) = self.groups.get(&group) {
                self.storage.save_group(group)?;
            }
        }

        for user in self.users.values_mut() {
            if user.remove_parent(&name) && user.primary_group() == name {
                user.set_primary_group(DEFAULT_GROUP);
            }
        }
        self.save_loaded_users()?;
        self.invalidate_all();
        Ok(())
    }

    pub fn edit_group<F>(&mut self, name: &str, edit: F) -> Result<bool>
    where
        F: FnOnce(&mut Group) -> bool,
    {
        let group = self
            .groups
            .get_mut(name)
            .ok_or_else(|| RookPermsError::GroupNotFound(name.to_lowercase()))?;
        if !edit(group) {
            return Ok(false);
        }
        self.storage.save_group(group)?;
        self.invalidate_all();
        Ok(true)
    }

    pub fn load_user(&mut self, uuid: PlayerId, username: &str) -> Result<()> {
        let mut user = self
            .storage
            .load_user(uuid)?
            .unwrap_or_else(|| User::new(uuid, username));

        let renamed = user.set_username(username);
        if renamed || user.parents().is_empty() {
            if user.parents().is_empty() {
                user.add_parent(DEFAULT_GROUP);
            }
            self.storage.save_user(&user)?;
        }

        self.storage.index_username(username, uuid)?;
        self.users.insert(uuid, user);
        self.cache.remove(&uuid);
        Ok(())
    }

    pub fn unload_user(&mut self, uuid: PlayerId) -> Result<()> {
        self.cache.remove(&uuid);
        if let Some(user) = self.users.remove(&uuid) {
            self.storage.save_user(&user)?;
        }
        Ok(())
    }

    pub fn is_loaded(&self, uuid: PlayerId) -> bool {
        self.users.contains_key(&uuid)
    }

    pub fn user(&self, uuid: PlayerId) -> Option<&User> {
        self.users.get(&uuid)
    }

    pub fn fetch_user(&self, uuid: PlayerId) -> Result<Option<User>> {
        match self.users.get(&uuid) {
            Some(user) => Ok(Some(user.clone())),
            None => self.storage.load_user(uuid),
        }
    }

    pub fn edit_user<F>(&mut self, uuid: PlayerId, edit: F) -> Result<bool>
    where
        F: FnOnce(&mut User) -> bool,
    {
        if let Some(user) = self.users.get_mut(&uuid) {
            if !edit(user) {
                return Ok(false);
            }
            self.storage.save_user(user)?;
            self.cache.remove(&uuid);
            return Ok(true);
        }

        let mut user = self
            .storage
            .load_user(uuid)?
            .ok_or_else(|| RookPermsError::UserNotLoaded(uuid.to_string()))?;
        if !edit(&mut user) {
            return Ok(false);
        }
        self.storage.save_user(&user)?;
        Ok(true)
    }

    pub fn create_user(&mut self, uuid: PlayerId, username: &str) -> Result<User> {
        let user = User::new(uuid, username);
        self.storage.save_user(&user)?;
        self.storage.index_username(username, uuid)?;
        Ok(user)
    }

    pub fn lookup_username(&self, username: &str) -> Result<Option<PlayerId>> {
        self.storage.lookup_username(username)
    }

    pub fn resolve(
        &mut self,
        uuid: PlayerId,
        query: &QueryContext,
    ) -> Option<&ResolvedPermissions> {
        let is_valid = self
            .cache
            .get(&uuid)
            .is_some_and(|cached| cached.is_valid_for(query));

        if !is_valid {
            let user = self.users.get(&uuid)?;
            let resolved = PermissionResolver::resolve(user, &self.groups, query);
            self.cache.insert(
                uuid,
                CachedPermissions {
                    contexts: query.contexts().clone(),
                    resolved,
                },
            );
        }

        self.cache.get(&uuid).map(|cached| &cached.resolved)
    }

    pub fn resolve_user(&self, user: &User, query: &QueryContext) -> ResolvedPermissions {
        PermissionResolver::resolve(user, &self.groups, query)
    }

    pub fn check(&mut self, uuid: PlayerId, node: &str, query: &QueryContext) -> Option<bool> {
        self.resolve(uuid, query)?.check(node)
    }

    pub fn invalidate(&mut self, uuid: PlayerId) {
        self.cache.remove(&uuid);
    }

    pub fn invalidate_all(&mut self) {
        self.cache.clear();
    }

    pub fn loaded_users(&self) -> impl Iterator<Item = &User> {
        self.users.values()
    }

    pub fn save_all(&mut self) -> Result<()> {
        for group in self.groups.iter() {
            self.storage.save_group(group)?;
        }
        self.save_loaded_users()
    }

    fn save_loaded_users(&self) -> Result<()> {
        for user in self.users.values() {
            self.storage.save_user(user)?;
        }
        Ok(())
    }
}
