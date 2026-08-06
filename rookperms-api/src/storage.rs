use crate::error::Result;
use crate::group::Group;
use crate::id::PlayerId;
use crate::user::User;

pub trait PermissionStorage: Send + Sync {
    fn load_groups(&self) -> Result<Vec<Group>>;

    fn save_group(&self, group: &Group) -> Result<()>;

    fn delete_group(&self, name: &str) -> Result<()>;

    fn load_user(&self, uuid: PlayerId) -> Result<Option<User>>;

    fn save_user(&self, user: &User) -> Result<()>;

    fn lookup_username(&self, username: &str) -> Result<Option<PlayerId>>;

    fn index_username(&self, username: &str, uuid: PlayerId) -> Result<()>;
}
