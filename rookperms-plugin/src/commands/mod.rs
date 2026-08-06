mod admin;
mod args;
mod group;
mod user;

use pumpkin_plugin_api::command::{ArgumentType, Command, CommandError, CommandNode, StringType};
use pumpkin_plugin_api::{Context, Server};
use rookperms_api::PermissionService;
use rookperms_api::error::RookPermsError;

use crate::permissions::COMMAND_MANAGE;
use crate::{listeners, state, text};

use admin::{HelpHandler, ListGroupsHandler, ReloadHandler, SaveHandler};
use group::{
    GroupCreateHandler, GroupDeleteHandler, GroupInfoHandler, GroupMetaHandler, GroupParentHandler,
    GroupPermissionHandler, GroupWeightHandler,
};
use user::{
    UserCheckHandler, UserInfoHandler, UserMetaHandler, UserParentHandler, UserPermissionHandler,
};

pub fn register(context: &Context) {
    let command = Command::new(
        &["rookperms".to_owned(), "rp".to_owned(), "perms".to_owned()],
        "Manage RookPerms groups and users.",
    );

    let command = command.execute(HelpHandler);

    command.then(group_branch());
    command.then(user_branch());
    command.then(CommandNode::literal("help").execute(HelpHandler));
    command.then(CommandNode::literal("listgroups").execute(ListGroupsHandler));
    command.then(CommandNode::literal("reload").execute(ReloadHandler));
    command.then(CommandNode::literal("save").execute(SaveHandler));

    context.register_command(command, COMMAND_MANAGE);
}

fn group_branch() -> CommandNode {
    let permission = branch(
        literal("permission"),
        vec![
            branch(
                literal("set"),
                vec![branch(
                    word("node"),
                    vec![boolean("value").execute(GroupPermissionHandler::set())],
                )],
            ),
            branch(
                literal("settemp"),
                vec![branch(
                    word("node"),
                    vec![branch(
                        boolean("value"),
                        vec![word("duration").execute(GroupPermissionHandler::set_temporary())],
                    )],
                )],
            ),
            branch(
                literal("unset"),
                vec![word("node").execute(GroupPermissionHandler::unset())],
            ),
        ],
    );

    let parent = branch(
        literal("parent"),
        vec![
            branch(
                literal("add"),
                vec![word("parent").execute(GroupParentHandler::add())],
            ),
            branch(
                literal("remove"),
                vec![word("parent").execute(GroupParentHandler::remove())],
            ),
        ],
    );

    let meta = branch(
        literal("meta"),
        vec![
            branch(
                literal("weight"),
                vec![integer("weight").execute(GroupWeightHandler)],
            ),
            branch(
                literal("prefix"),
                vec![greedy("value").execute(GroupMetaHandler::prefix())],
            ),
            branch(
                literal("suffix"),
                vec![greedy("value").execute(GroupMetaHandler::suffix())],
            ),
        ],
    );

    let target = branch(
        word("group"),
        vec![
            literal("create").execute(GroupCreateHandler),
            literal("delete").execute(GroupDeleteHandler),
            literal("info").execute(GroupInfoHandler),
            permission,
            parent,
            meta,
        ],
    );

    branch(literal("group"), vec![target])
}

fn user_branch() -> CommandNode {
    let permission = branch(
        literal("permission"),
        vec![
            branch(
                literal("set"),
                vec![branch(
                    word("node"),
                    vec![boolean("value").execute(UserPermissionHandler::set())],
                )],
            ),
            branch(
                literal("settemp"),
                vec![branch(
                    word("node"),
                    vec![branch(
                        boolean("value"),
                        vec![word("duration").execute(UserPermissionHandler::set_temporary())],
                    )],
                )],
            ),
            branch(
                literal("unset"),
                vec![word("node").execute(UserPermissionHandler::unset())],
            ),
        ],
    );

    let parent = branch(
        literal("parent"),
        vec![
            branch(
                literal("add"),
                vec![word("parent").execute(UserParentHandler::add())],
            ),
            branch(
                literal("remove"),
                vec![word("parent").execute(UserParentHandler::remove())],
            ),
            branch(
                literal("set"),
                vec![word("parent").execute(UserParentHandler::set())],
            ),
        ],
    );

    let meta = branch(
        literal("meta"),
        vec![
            branch(
                literal("prefix"),
                vec![greedy("value").execute(UserMetaHandler::prefix())],
            ),
            branch(
                literal("suffix"),
                vec![greedy("value").execute(UserMetaHandler::suffix())],
            ),
        ],
    );

    let target = branch(
        word("player"),
        vec![
            literal("info").execute(UserInfoHandler),
            branch(
                literal("check"),
                vec![word("node").execute(UserCheckHandler)],
            ),
            permission,
            parent,
            meta,
        ],
    );

    branch(literal("user"), vec![target])
}

fn branch(node: CommandNode, children: Vec<CommandNode>) -> CommandNode {
    for child in children {
        node.then(child);
    }
    node
}

fn literal(name: &str) -> CommandNode {
    CommandNode::literal(name)
}

fn word(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::SingleWord))
}

fn greedy(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::String(StringType::Greedy))
}

fn boolean(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::Bool)
}

fn integer(name: &str) -> CommandNode {
    CommandNode::argument(name, &ArgumentType::Integer((None, None)))
}

pub(crate) fn service<R>(
    action: impl FnOnce(&mut PermissionService) -> R,
) -> Result<R, CommandError> {
    state::with_service(action).ok_or_else(|| failed("RookPerms is not initialised."))
}

pub(crate) fn failed(message: impl AsRef<str>) -> CommandError {
    CommandError::CommandFailed(text::failure(message.as_ref()))
}

pub(crate) fn missing(argument: &str) -> CommandError {
    failed(format!("Missing or invalid argument '{argument}'."))
}

pub(crate) fn map_error(error: RookPermsError) -> CommandError {
    failed(capitalize(&error.to_string()))
}

pub(crate) fn refresh_online(server: &Server) {
    for player in server.get_all_players() {
        listeners::refresh(&player);
    }
}

fn capitalize(message: &str) -> String {
    let mut characters = message.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}
