use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::command::{CommandError, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::commands::CommandHandler;
use rookperms_api::context::ContextSet;
use rookperms_api::group::{self, Group};
use rookperms_api::holder::PermissionHolder;
use rookperms_api::meta::WeightedValue;
use rookperms_api::node::{self, Node};

use crate::commands::args;
use crate::commands::{failed, map_error, missing, refresh_online, service};
use crate::text;
use crate::time::{format_duration, now_unix, parse_duration};

const CLEAR_KEYWORD: &str = "none";

#[derive(Clone, Copy)]
pub enum PermissionAction {
    Set,
    SetTemporary,
    Unset,
}

#[derive(Clone, Copy)]
pub enum ParentAction {
    Add,
    Remove,
}

#[derive(Clone, Copy)]
pub enum MetaField {
    Prefix,
    Suffix,
}

impl MetaField {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Prefix => "prefix",
            Self::Suffix => "suffix",
        }
    }

    fn apply(self, group: &mut Group, value: Option<WeightedValue>) {
        match self {
            Self::Prefix => group.data_mut().set_prefix(value),
            Self::Suffix => group.data_mut().set_suffix(value),
        }
    }
}

pub struct GroupCreateHandler;

impl CommandHandler for GroupCreateHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = group_name(&args)?;
        service(|permissions| permissions.create_group(&name))?.map_err(map_error)?;
        text::send_success(&sender, &format!("Created group '{name}'."));
        Ok(1)
    }
}

pub struct GroupDeleteHandler;

impl CommandHandler for GroupDeleteHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = group_name(&args)?;
        service(|permissions| permissions.delete_group(&name))?.map_err(map_error)?;
        refresh_online(&server);
        text::send_success(&sender, &format!("Deleted group '{name}'."));
        Ok(1)
    }
}

pub struct GroupInfoHandler;

impl CommandHandler for GroupInfoHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = group_name(&args)?;
        let group = service(|permissions| permissions.group(&name).cloned())?
            .map_err(map_error)?;
        let now = now_unix();

        text::send_info(&sender, &format!("Group '{}'", group.name()));
        text::send(&sender, text::entry("Weight", &group.weight().to_string()));
        text::send(
            &sender,
            text::entry(
                "Prefix",
                group.data().prefix().map_or("-", WeightedValue::value),
            ),
        );
        text::send(
            &sender,
            text::entry(
                "Suffix",
                group.data().suffix().map_or("-", WeightedValue::value),
            ),
        );
        text::send(
            &sender,
            text::entry(
                "Parents",
                &join_or_dash(group.parents().iter().map(String::as_str)),
            ),
        );
        text::send(
            &sender,
            text::entry("Permissions", &group.nodes().len().to_string()),
        );

        for line in describe_nodes(group.nodes(), now) {
            text::send(&sender, text::entry(" ", &line));
        }

        Ok(1)
    }
}

pub struct GroupPermissionHandler {
    action: PermissionAction,
}

impl GroupPermissionHandler {
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

impl CommandHandler for GroupPermissionHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = group_name(&args)?;
        let key = permission_key(&args)?;

        let (changed, message) = match self.action {
            PermissionAction::Unset => {
                let changed = service(|permissions| {
                    permissions.edit_group(&name, |group| {
                        group.data_mut().unset_node(&key, &ContextSet::default())
                    })
                })?
                .map_err(map_error)?;
                (changed, format!("Unset '{key}' for group '{name}'."))
            }
            action => {
                let value = args::flag(&args, "value").ok_or_else(|| missing("value"))?;
                let expiry = match action {
                    PermissionAction::SetTemporary => Some(expiry_from(&args)?),
                    _ => None,
                };
                let node = Node::new(&key, value).with_expiry(expiry);
                let changed = service(|permissions| {
                    permissions.edit_group(&name, |group| group.set_node(node))
                })?
                .map_err(map_error)?;
                (changed, describe_set(&key, value, expiry, &name))
            }
        };

        if !changed {
            return Err(failed(format!(
                "Group '{name}' already had that permission state."
            )));
        }

        refresh_online(&server);
        text::send_success(&sender, &message);
        Ok(1)
    }
}

pub struct GroupParentHandler {
    action: ParentAction,
}

impl GroupParentHandler {
    pub const fn add() -> Self {
        Self {
            action: ParentAction::Add,
        }
    }

    pub const fn remove() -> Self {
        Self {
            action: ParentAction::Remove,
        }
    }
}

impl CommandHandler for GroupParentHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = group_name(&args)?;
        let parent = args::text(&args, "parent")
            .ok_or_else(|| missing("parent"))?
            .to_lowercase();

        if name == parent {
            return Err(failed("A group cannot inherit from itself."));
        }

        let known = service(|permissions| permissions.groups().contains(&parent))?;
        if !known {
            return Err(failed(format!("Group '{parent}' does not exist.")));
        }

        let changed = service(|permissions| {
            permissions.edit_group(&name, |group| match self.action {
                ParentAction::Add => group.add_parent(&parent),
                ParentAction::Remove => group.remove_parent(&parent),
            })
        })?
        .map_err(map_error)?;

        if !changed {
            return Err(failed(format!(
                "Group '{name}' inheritance was already in that state."
            )));
        }

        refresh_online(&server);
        let verb = match self.action {
            ParentAction::Add => "now inherits",
            ParentAction::Remove => "no longer inherits",
        };
        text::send_success(&sender, &format!("Group '{name}' {verb} '{parent}'."));
        Ok(1)
    }
}

pub struct GroupWeightHandler;

impl CommandHandler for GroupWeightHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = group_name(&args)?;
        let weight = args::integer(&args, "weight").ok_or_else(|| missing("weight"))?;

        service(|permissions| {
            permissions.edit_group(&name, |group| {
                if group.weight() == weight {
                    return false;
                }
                group.set_weight(weight);
                true
            })
        })?
        .map_err(map_error)?;

        refresh_online(&server);
        text::send_success(&sender, &format!("Group '{name}' weight is now {weight}."));
        Ok(1)
    }
}

pub struct GroupMetaHandler {
    field: MetaField,
}

impl GroupMetaHandler {
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

impl CommandHandler for GroupMetaHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let name = group_name(&args)?;
        let value = args::text(&args, "value").ok_or_else(|| missing("value"))?;
        let field = self.field;
        let cleared = value.eq_ignore_ascii_case(CLEAR_KEYWORD);

        service(|permissions| {
            permissions.edit_group(&name, |group| {
                let replacement = (!cleared).then(|| WeightedValue::new(&value, group.weight()));
                field.apply(group, replacement);
                true
            })
        })?
        .map_err(map_error)?;

        refresh_online(&server);
        let message = if cleared {
            format!("Cleared {} of group '{name}'.", field.label())
        } else {
            format!("Set {} of group '{name}' to '{value}'.", field.label())
        };
        text::send_success(&sender, &message);
        Ok(1)
    }
}

fn group_name(args: &ConsumedArgs) -> Result<String, CommandError> {
    let name = args::text(args, "group")
        .ok_or_else(|| missing("group"))?
        .to_lowercase();
    if !group::is_valid_name(&name) {
        return Err(failed(format!("'{name}' is not a valid group name.")));
    }
    Ok(name)
}

pub(super) fn permission_key(args: &ConsumedArgs) -> Result<String, CommandError> {
    let key = args::text(args, "node").ok_or_else(|| missing("node"))?;
    if !node::is_valid_key(&key) {
        return Err(failed(format!("'{key}' is not a valid permission node.")));
    }
    Ok(node::normalize_key(&key))
}

pub(super) fn expiry_from(args: &ConsumedArgs) -> Result<u64, CommandError> {
    let raw = args::text(args, "duration").ok_or_else(|| missing("duration"))?;
    let seconds = parse_duration(&raw)
        .filter(|seconds| *seconds > 0)
        .ok_or_else(|| failed(format!("'{raw}' is not a valid duration (e.g. 30m, 7d).")))?;
    Ok(now_unix().saturating_add(seconds))
}

pub(super) fn describe_set(key: &str, value: bool, expiry: Option<u64>, holder: &str) -> String {
    let state = if value { "true" } else { "false" };
    match expiry {
        Some(expiry) => format!(
            "Set '{key}' to {state} for '{holder}' for {}.",
            format_duration(expiry.saturating_sub(now_unix()))
        ),
        None => format!("Set '{key}' to {state} for '{holder}'."),
    }
}

pub(super) fn describe_nodes(nodes: &[Node], now: u64) -> Vec<String> {
    nodes
        .iter()
        .take(25)
        .map(|node| {
            let state = if node.value() { "+" } else { "-" };
            match node.expiry() {
                Some(expiry) => format!(
                    "{state}{} ({})",
                    node.key(),
                    format_duration(expiry.saturating_sub(now))
                ),
                None => format!("{state}{}", node.key()),
            }
        })
        .collect()
}

pub(super) fn join_or_dash<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let joined = values.collect::<Vec<_>>().join(", ");
    if joined.is_empty() {
        "-".to_owned()
    } else {
        joined
    }
}
