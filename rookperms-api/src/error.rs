use std::fmt;

pub type Result<T> = std::result::Result<T, RookPermsError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RookPermsError {
    GroupNotFound(String),
    GroupAlreadyExists(String),
    UserNotLoaded(String),
    ProtectedGroup(String),
    InvalidNode(String),
    Storage(String),
}

impl fmt::Display for RookPermsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GroupNotFound(name) => write!(f, "group '{name}' does not exist"),
            Self::GroupAlreadyExists(name) => write!(f, "group '{name}' already exists"),
            Self::UserNotLoaded(name) => write!(f, "user '{name}' is not loaded"),
            Self::ProtectedGroup(name) => write!(f, "group '{name}' cannot be removed"),
            Self::InvalidNode(node) => write!(f, "'{node}' is not a valid permission node"),
            Self::Storage(reason) => write!(f, "storage error: {reason}"),
        }
    }
}

impl std::error::Error for RookPermsError {}
