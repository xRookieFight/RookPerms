use rookperms_api::context::{CONTEXT_WORLD, ContextSet};
use rookperms_api::group::{DEFAULT_GROUP, Group, GroupRegistry};
use rookperms_api::holder::PermissionHolder;
use rookperms_api::id::PlayerId;
use rookperms_api::meta::WeightedValue;
use rookperms_api::node::Node;
use rookperms_api::resolver::{PermissionResolver, QueryContext, ResolvedPermissions};
use rookperms_api::user::User;

const NOW: u64 = 1_000;

fn registry(groups: Vec<Group>) -> GroupRegistry {
    let mut registry = GroupRegistry::new();
    registry.insert(Group::new(DEFAULT_GROUP));
    for group in groups {
        registry.insert(group);
    }
    registry
}

fn user() -> User {
    User::new(PlayerId::from_parts(1, 2), "Rook")
}

fn resolve(user: &User, groups: &GroupRegistry, contexts: ContextSet) -> ResolvedPermissions {
    PermissionResolver::resolve(user, groups, &QueryContext::new(contexts, NOW))
}

#[test]
fn inherits_group_nodes() {
    let mut moderator = Group::new("moderator");
    moderator.set_node(Node::new("minecraft:command.kick", true));

    let mut subject = user();
    subject.add_parent("moderator");

    let resolved = resolve(&subject, &registry(vec![moderator]), ContextSet::new());
    assert_eq!(resolved.check("minecraft:command.kick"), Some(true));
    assert_eq!(resolved.check("minecraft:command.ban"), None);
}

#[test]
fn higher_weight_group_wins() {
    let mut member = Group::new("member");
    member.set_weight(10);
    member.set_node(Node::new("essentials.fly", false));

    let mut admin = Group::new("admin");
    admin.set_weight(100);
    admin.set_node(Node::new("essentials.fly", true));

    let mut subject = user();
    subject.add_parent("member");
    subject.add_parent("admin");

    let resolved = resolve(&subject, &registry(vec![member, admin]), ContextSet::new());
    assert_eq!(resolved.check("essentials.fly"), Some(true));
}

#[test]
fn user_node_overrides_group() {
    let mut admin = Group::new("admin");
    admin.set_weight(100);
    admin.set_node(Node::new("essentials.fly", true));

    let mut subject = user();
    subject.add_parent("admin");
    subject.set_node(Node::new("essentials.fly", false));

    let resolved = resolve(&subject, &registry(vec![admin]), ContextSet::new());
    assert_eq!(resolved.check("essentials.fly"), Some(false));
}

#[test]
fn wildcard_grants_children() {
    let mut admin = Group::new("admin");
    admin.set_node(Node::new("essentials.*", true));

    let mut subject = user();
    subject.add_parent("admin");

    let resolved = resolve(&subject, &registry(vec![admin]), ContextSet::new());
    assert_eq!(resolved.check("essentials.fly"), Some(true));
    assert_eq!(resolved.check("essentials.home.other"), Some(true));
    assert_eq!(resolved.check("worldedit.set"), None);
}

#[test]
fn nested_inheritance_is_resolved() {
    let mut base = Group::new("base");
    base.set_node(Node::new("chat.speak", true));

    let mut trusted = Group::new("trusted");
    trusted.set_weight(5);
    trusted.add_parent("base");

    let mut subject = user();
    subject.add_parent("trusted");

    let resolved = resolve(&subject, &registry(vec![base, trusted]), ContextSet::new());
    assert_eq!(resolved.check("chat.speak"), Some(true));
    assert_eq!(resolved.inherited_groups().len(), 3);
}

#[test]
fn expired_nodes_are_ignored() {
    let mut subject = user();
    subject.set_node(Node::new("essentials.fly", true).with_expiry(Some(NOW - 1)));
    subject.set_node(Node::new("essentials.home", true).with_expiry(Some(NOW + 60)));

    let resolved = resolve(&subject, &registry(Vec::new()), ContextSet::new());
    assert_eq!(resolved.check("essentials.fly"), None);
    assert_eq!(resolved.check("essentials.home"), Some(true));
    assert_eq!(resolved.valid_until(), Some(NOW + 60));
}

#[test]
fn context_limits_nodes() {
    let mut subject = user();
    subject.set_node(
        Node::new("essentials.fly", true)
            .with_context(ContextSet::new().with(CONTEXT_WORLD, "nether")),
    );

    let overworld = resolve(
        &subject,
        &registry(Vec::new()),
        ContextSet::new().with(CONTEXT_WORLD, "overworld"),
    );
    assert_eq!(overworld.check("essentials.fly"), None);

    let nether = resolve(
        &subject,
        &registry(Vec::new()),
        ContextSet::new().with(CONTEXT_WORLD, "nether"),
    );
    assert_eq!(nether.check("essentials.fly"), Some(true));
}

#[test]
fn highest_priority_prefix_wins() {
    let mut member = Group::new("member");
    member
        .data_mut()
        .set_prefix(Some(WeightedValue::new("[M] ", 10)));

    let mut admin = Group::new("admin");
    admin
        .data_mut()
        .set_prefix(Some(WeightedValue::new("[A] ", 50)));

    let mut subject = user();
    subject.add_parent("member");
    subject.add_parent("admin");

    let resolved = resolve(&subject, &registry(vec![member, admin]), ContextSet::new());
    assert_eq!(resolved.prefix(), Some("[A] "));
}
