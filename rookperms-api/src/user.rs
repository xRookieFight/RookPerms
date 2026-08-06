use serde::{Deserialize, Serialize};

use crate::group::DEFAULT_GROUP;
use crate::holder::{HolderData, PermissionHolder};
use crate::id::PlayerId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    uuid: PlayerId,
    #[serde(default)]
    username: String,
    #[serde(default = "default_group_name")]
    primary_group: String,
    #[serde(flatten)]
    data: HolderData,
}

fn default_group_name() -> String {
    DEFAULT_GROUP.to_owned()
}

impl User {
    pub fn new(uuid: PlayerId, username: impl Into<String>) -> Self {
        let mut data = HolderData::default();
        data.add_parent(DEFAULT_GROUP);
        Self {
            uuid,
            username: username.into(),
            primary_group: default_group_name(),
            data,
        }
    }

    pub fn uuid(&self) -> PlayerId {
        self.uuid
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn set_username(&mut self, username: impl Into<String>) -> bool {
        let username = username.into();
        if self.username == username {
            return false;
        }
        self.username = username;
        true
    }

    pub fn primary_group(&self) -> &str {
        &self.primary_group
    }

    pub fn set_primary_group(&mut self, group: impl AsRef<str>) {
        let group = group.as_ref().to_lowercase();
        self.data.add_parent(&group);
        self.primary_group = group;
    }
}

impl PermissionHolder for User {
    fn identifier(&self) -> &str {
        &self.username
    }

    fn data(&self) -> &HolderData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut HolderData {
        &mut self.data
    }
}
