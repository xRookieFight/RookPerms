use std::str::FromStr;

use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::command::{CommandError, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::commands::CommandHandler;
use rookperms_api::context::ContextSet;
use rookperms_api::holder::PermissionHolder;
use rookperms_api::id::PlayerId;
use rookperms_api::meta::WeightedValue;
use rookperms_api::node::Node;
use rookperms_api::resolver::QueryContext;
use rookperms_api::user::User;

use crate::commands::args;
use crate::commands::group::{
    MetaField, ParentAction, PermissionAction, describe_nodes, describe_set, expiry_from,
    join_or_dash, permission_key,
};
use crate::commands::{failed, map_error, missing, service};
use crate::listeners;
use crate::player::id_of;
use crate::text;
use crate::time::now_unix;

const USER_META_PRIORITY: i32 = 100;
const CLEAR_KEYWORD: &str = "none";

struct Target {
    uuid: PlayerId,
    name: String,
}

pub struct UserInfoHandler;

impl CommandHandler for UserInfoHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = resolve_target(&server, &args)?;
        let user = load_user(&target)?;
        let now = now_unix();
        let query = QueryContext::new(ContextSet::default(), now);
        let resolved = service(|permissions| permissions.resolve_user(&user, &query))?;

        text::send_info(&sender, &format!("User '{}'", user.username()));
        text::send(&sender, text::entry("Uuid", &user.uuid().to_string()));
        text::send(&sender, text::entry("Primary group", user.primary_group()));
        text::send(
            &sender,
            text::entry(
                "Parents",
                &join_or_dash(user.parents().iter().map(String::as_str)),
            ),
        );
        text::send(
            &sender,
            text::entry("Prefix", resolved.prefix().unwrap_or("-")),
        );
        text::send(
            &sender,
            text::entry("Suffix", resolved.suffix().unwrap_or("-")),
        );
        text::send(
            &sender,
            text::entry(
                "Effective permissions",
                &resolved.permissions().len().to_string(),
            ),
        );

        for line in describe_nodes(user.nodes(), now) {
            text::send(&sender, text::entry(" ", &line));
        }

        Ok(1)
    }
}

pub struct UserCheckHandler;

impl CommandHandler for UserCheckHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = resolve_target(&server, &args)?;
        let key = permission_key(&args)?;
        let user = load_user(&target)?;
        let query = QueryContext::new(ContextSet::default(), now_unix());
        let resolved = service(|permissions| permissions.resolve_user(&user, &query))?;

        let state = match resolved.check(&key) {
            Some(true) => "true",
            Some(false) => "false",
            None => "undefined",
        };
        text::send_info(&sender, &format!("{} -> {key} = {state}", user.username()));
        Ok(1)
    }
}

pub struct UserPermissionHandler {
    action: PermissionAction,
}

impl UserPermissionHandler {
    pub const fn set() -> Self {
        Self {
            action: PermissionAction::Set,
        }
    }

    pub const fn set_temporary() -> Self {
        Self {
            action: PermissionAction::SetTemporary,
        }
    }

    pub const fn unset() -> Self {
        Self {
            action: PermissionAction::Unset,
        }
    }
}

impl CommandHandler for UserPermissionHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = resolve_target(&server, &args)?;
        let key = permission_key(&args)?;

        let (changed, message) = match self.action {
            PermissionAction::Unset => {
                let changed = service(|permissions| {
                    permissions.edit_user(target.uuid, |user| {
                        user.data_mut().unset_node(&key, &ContextSet::default())
                    })
                })?
                .map_err(map_error)?;
                (changed, format!("Unset '{key}' for {}.", target.name))
            }
            action => {
                let value = args::flag(&args, "value").ok_or_else(|| missing("value"))?;
                let expiry = match action {
                    PermissionAction::SetTemporary => Some(expiry_from(&args)?),
                    _ => None,
                };
                let node = Node::new(&key, value).with_expiry(expiry);
                let changed = service(|permissions| {
                    permissions.edit_user(target.uuid, |user| user.set_node(node))
                })?
                .map_err(map_error)?;
                (changed, describe_set(&key, value, expiry, &target.name))
            }
        };

        if !changed {
            return Err(failed(format!(
                "{} already had that permission state.",
                target.name
            )));
        }

        refresh_target(&server, &target);
        text::send_success(&sender, &message);
        Ok(1)
    }
}

pub struct UserParentHandler {
    action: ParentAction,
    replace: bool,
}

impl UserParentHandler {
    pub const fn add() -> Self {
        Self {
            action: ParentAction::Add,
            replace: false,
        }
    }

    pub const fn remove() -> Self {
        Self {
            action: ParentAction::Remove,
            replace: false,
        }
    }

    pub const fn set() -> Self {
        Self {
            action: ParentAction::Add,
            replace: true,
        }
    }
}

impl CommandHandler for UserParentHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = resolve_target(&server, &args)?;
        let parent = args::text(&args, "parent")
            .ok_or_else(|| missing("parent"))?
            .to_lowercase();

        let known = service(|permissions| permissions.groups().contains(&parent))?;
        if !known {
            return Err(failed(format!("Group '{parent}' does not exist.")));
        }

        let replace = self.replace;
        let action = self.action;
        let changed = service(|permissions| {
            permissions.edit_user(target.uuid, |user| {
                if replace {
                    user.data_mut().set_parents(vec![parent.clone()]);
                    user.set_primary_group(&parent);
                    return true;
                }
                match action {
                    ParentAction::Add => user.add_parent(&parent),
                    ParentAction::Remove => {
                        let removed = user.remove_parent(&parent);
                        if removed && user.primary_group() == parent {
                            user.set_primary_group(rookperms_api::DEFAULT_GROUP);
                        }
                        removed
                    }
                }
            })
        })?
        .map_err(map_error)?;

        if !changed {
            return Err(failed(format!(
                "{} inheritance was already in that state.",
                target.name
            )));
        }

        refresh_target(&server, &target);
        let message = if replace {
            format!("{} is now only a member of '{parent}'.", target.name)
        } else {
            match action {
                ParentAction::Add => format!("Added {} to group '{parent}'.", target.name),
                ParentAction::Remove => {
                    format!("Removed {} from group '{parent}'.", target.name)
                }
            }
        };
        text::send_success(&sender, &message);
        Ok(1)
    }
}

pub struct UserMetaHandler {
    field: MetaField,
}

impl UserMetaHandler {
    pub const fn prefix() -> Self {
        Self {
            field: MetaField::Prefix,
        }
    }

    pub const fn suffix() -> Self {
        Self {
            field: MetaField::Suffix,
        }
    }
}

impl CommandHandler for UserMetaHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let target = resolve_target(&server, &args)?;
        let value = args::text(&args, "value").ok_or_else(|| missing("value"))?;
        let cleared = value.eq_ignore_ascii_case(CLEAR_KEYWORD);
        let field = self.field;

        service(|permissions| {
            permissions.edit_user(target.uuid, |user| {
                let replacement =
                    (!cleared).then(|| WeightedValue::new(&value, USER_META_PRIORITY));
                match field {
                    MetaField::Prefix => user.data_mut().set_prefix(replacement),
                    MetaField::Suffix => user.data_mut().set_suffix(replacement),
                }
                true
            })
        })?
        .map_err(map_error)?;

        refresh_target(&server, &target);
        let message = if cleared {
            format!("Cleared {} of {}.", field.label(), target.name)
        } else {
            format!("Set {} of {} to '{value}'.", field.label(), target.name)
        };
        text::send_success(&sender, &message);
        Ok(1)
    }
}

fn resolve_target(server: &Server, args: &ConsumedArgs) -> Result<Target, CommandError> {
    let input = args::text(args, "player").ok_or_else(|| missing("player"))?;

    if let Some(player) = server.get_player_by_name(&input) {
        return Ok(Target {
            uuid: id_of(&player),
            name: player.get_name(),
        });
    }

    if let Ok(uuid) = PlayerId::from_str(&input) {
        return Ok(Target { uuid, name: input });
    }

    let stored = service(|permissions| permissions.lookup_username(&input))?.map_err(map_error)?;
    stored
        .map(|uuid| Target {
            uuid,
            name: input.clone(),
        })
        .ok_or_else(|| failed(format!("Unknown player '{input}'.")))
}

fn load_user(target: &Target) -> Result<User, CommandError> {
    service(|permissions| permissions.fetch_user(target.uuid))?
        .map_err(map_error)?
        .ok_or_else(|| failed(format!("No stored data for '{}'.", target.name)))
}

fn refresh_target(server: &Server, target: &Target) {
    if let Some(player) = server.get_player_by_uuid(uuid_of(target.uuid)) {
        listeners::refresh(&player);
    }
}

fn uuid_of(id: PlayerId) -> pumpkin_plugin_api::uuid::Uuid {
    let (high, low) = id.parts();
    pumpkin_plugin_api::uuid::Uuid { high, low }
}
