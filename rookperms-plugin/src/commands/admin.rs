use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::command::{CommandError, CommandSender, ConsumedArgs};
use pumpkin_plugin_api::commands::CommandHandler;

use crate::commands::{map_error, service};
use crate::text;

const GROUP_USAGES: [(&str, &str); 9] = [
    ("/rp group <group> create", "Creates a group"),
    ("/rp group <group> delete", "Deletes a group"),
    (
        "/rp group <group> info",
        "Shows weight, meta, parents and nodes",
    ),
    (
        "/rp group <group> permission set <node> <true|false>",
        "Sets a permission",
    ),
    (
        "/rp group <group> permission settemp <node> <true|false> <duration>",
        "Sets a temporary permission",
    ),
    (
        "/rp group <group> permission unset <node>",
        "Removes a permission",
    ),
    (
        "/rp group <group> parent add|remove <parent>",
        "Manages inheritance",
    ),
    ("/rp group <group> meta weight <number>", "Sets the weight"),
    (
        "/rp group <group> meta prefix|suffix <value>",
        "Sets meta, 'none' clears it",
    ),
];

const USER_USAGES: [(&str, &str); 8] = [
    ("/rp user <player> info", "Shows stored and resolved state"),
    (
        "/rp user <player> check <node>",
        "Shows the resolved value of a node",
    ),
    (
        "/rp user <player> permission set <node> <true|false>",
        "Sets a permission",
    ),
    (
        "/rp user <player> permission settemp <node> <true|false> <duration>",
        "Sets a temporary permission",
    ),
    (
        "/rp user <player> permission unset <node>",
        "Removes a permission",
    ),
    (
        "/rp user <player> parent add|remove <group>",
        "Manages group membership",
    ),
    (
        "/rp user <player> parent set <group>",
        "Replaces all groups with one",
    ),
    (
        "/rp user <player> meta prefix|suffix <value>",
        "Sets meta, 'none' clears it",
    ),
];

const MAINTENANCE_USAGES: [(&str, &str); 3] = [
    ("/rp listgroups", "Lists every group with its weight"),
    ("/rp reload", "Reloads all data from disk"),
    ("/rp save", "Writes all loaded data to disk"),
];

pub struct HelpHandler;

impl CommandHandler for HelpHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        text::send_info(
            &sender,
            &format!(
                "RookPerms {} - aliases: /rp, /perms",
                env!("CARGO_PKG_VERSION")
            ),
        );

        for (title, usages) in [
            ("Groups", GROUP_USAGES.as_slice()),
            ("Users", USER_USAGES.as_slice()),
            ("Maintenance", MAINTENANCE_USAGES.as_slice()),
        ] {
            text::send(&sender, text::heading(title));
            for (command, description) in usages {
                text::send(&sender, text::usage(command, description));
            }
        }

        text::send_info(
            &sender,
            "Players may be given by name or uuid. Durations accept 30m, 12h, 7d or 1d12h.",
        );
        Ok(1)
    }
}

pub struct ListGroupsHandler;

impl CommandHandler for ListGroupsHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let groups = service(|permissions| {
            permissions
                .groups()
                .iter()
                .map(|group| (group.name().to_owned(), group.weight()))
                .collect::<Vec<_>>()
        })?;

        text::send_info(&sender, &format!("Groups ({})", groups.len()));
        for (name, weight) in groups {
            text::send(&sender, text::entry(&name, &format!("weight {weight}")));
        }
        Ok(1)
    }
}

pub struct ReloadHandler;

impl CommandHandler for ReloadHandler {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        service(|permissions| permissions.load())?.map_err(map_error)?;

        for player in server.get_all_players() {
            crate::listeners::synchronize(&player);
        }

        text::send_success(&sender, "Reloaded permissions from disk.");
        Ok(1)
    }
}

pub struct SaveHandler;

impl CommandHandler for SaveHandler {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        service(|permissions| permissions.save_all())?.map_err(map_error)?;
        text::send_success(&sender, "Saved all permission data.");
        Ok(1)
    }
}
