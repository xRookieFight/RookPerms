use pumpkin_plugin_api::permission::{Permission, PermissionDefault, PermissionLevel};
use pumpkin_plugin_api::{Context, Result};

pub const NAMESPACE: &str = "RookPerms";
pub const COMMAND_MANAGE: &str = "RookPerms:command.manage";

pub fn register(context: &Context) -> Result<()> {
    context.register_permission(&Permission {
        node: COMMAND_MANAGE.to_owned(),
        description: "Allows managing RookPerms groups and users.".to_owned(),
        default: PermissionDefault::Op(PermissionLevel::Three),
        children: Vec::new(),
    })
}
