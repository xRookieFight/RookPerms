use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use rookperms_api::error::{Result, RookPermsError};
use rookperms_api::group::{self, Group};
use rookperms_api::id::PlayerId;
use rookperms_api::storage::PermissionStorage;
use rookperms_api::user::User;
use serde::Serialize;
use serde::de::DeserializeOwned;

const GROUPS_DIR: &str = "groups";
const USERS_DIR: &str = "users";
const INDEX_FILE: &str = "usernames.json";

type UsernameIndex = BTreeMap<String, PlayerId>;

pub struct JsonStorage {
    root: PathBuf,
}

impl JsonStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn prepare(&self) -> Result<()> {
        create_dir(&self.root)?;
        create_dir(&self.groups_dir())?;
        create_dir(&self.users_dir())
    }

    fn groups_dir(&self) -> PathBuf {
        self.root.join(GROUPS_DIR)
    }

    fn users_dir(&self) -> PathBuf {
        self.root.join(USERS_DIR)
    }

    fn index_path(&self) -> PathBuf {
        self.root.join(INDEX_FILE)
    }

    fn group_path(&self, name: &str) -> Result<PathBuf> {
        let name = name.to_lowercase();
        if !group::is_valid_name(&name) {
            return Err(RookPermsError::Storage(format!(
                "invalid group name '{name}'"
            )));
        }
        Ok(self.groups_dir().join(format!("{name}.json")))
    }

    fn user_path(&self, uuid: PlayerId) -> PathBuf {
        self.users_dir().join(format!("{uuid}.json"))
    }

    fn read_index(&self) -> Result<UsernameIndex> {
        Ok(read_json(&self.index_path())?.unwrap_or_default())
    }
}

impl PermissionStorage for JsonStorage {
    fn load_groups(&self) -> Result<Vec<Group>> {
        let directory = self.groups_dir();
        if !directory.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&directory).map_err(to_storage_error)?;
        let mut groups = Vec::new();

        for entry in entries {
            let path = entry.map_err(to_storage_error)?.path();
            if path.extension().is_none_or(|extension| extension != "json") {
                continue;
            }
            match read_json::<Group>(&path)? {
                Some(group) => groups.push(group),
                None => continue,
            }
        }

        Ok(groups)
    }

    fn save_group(&self, group: &Group) -> Result<()> {
        write_json(&self.group_path(group.name())?, group)
    }

    fn delete_group(&self, name: &str) -> Result<()> {
        let path = self.group_path(name)?;
        if !path.exists() {
            return Ok(());
        }
        fs::remove_file(path).map_err(to_storage_error)
    }

    fn load_user(&self, uuid: PlayerId) -> Result<Option<User>> {
        read_json(&self.user_path(uuid))
    }

    fn save_user(&self, user: &User) -> Result<()> {
        write_json(&self.user_path(user.uuid()), user)
    }

    fn lookup_username(&self, username: &str) -> Result<Option<PlayerId>> {
        Ok(self.read_index()?.get(&username.to_lowercase()).copied())
    }

    fn index_username(&self, username: &str, uuid: PlayerId) -> Result<()> {
        let mut index = self.read_index()?;
        let key = username.to_lowercase();
        if index.get(&key) == Some(&uuid) {
            return Ok(());
        }
        index.insert(key, uuid);
        write_json(&self.index_path(), &index)
    }
}

fn create_dir(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(path).map_err(to_storage_error)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(to_storage_error)?;
    serde_json::from_str(&raw)
        .map(Some)
        .map_err(|error| RookPermsError::Storage(format!("{}: {error}", path.display())))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir(parent)?;
    }
    let raw = serde_json::to_string_pretty(value)
        .map_err(|error| RookPermsError::Storage(error.to_string()))?;
    fs::write(path, raw).map_err(to_storage_error)
}

fn to_storage_error(error: std::io::Error) -> RookPermsError {
    RookPermsError::Storage(error.to_string())
}
