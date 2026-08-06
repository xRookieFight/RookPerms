use rookperms_api::context::{CONTEXT_WORLD, ContextSet};
use rookperms_api::group::Group;
use rookperms_api::holder::PermissionHolder;
use rookperms_api::id::PlayerId;
use rookperms_api::meta::WeightedValue;
use rookperms_api::node::Node;
use rookperms_api::user::User;

#[test]
fn group_round_trip() {
    let mut group = Group::new("Admin");
    group.set_weight(75);
    group.add_parent("Member");
    group.set_node(Node::new("essentials.*", true));
    group.set_node(
        Node::new("essentials.fly", false)
            .with_expiry(Some(1_700_000_000))
            .with_context(ContextSet::new().with(CONTEXT_WORLD, "nether")),
    );
    group
        .data_mut()
        .set_prefix(Some(WeightedValue::new("[A] ", 75)));
    group.data_mut().set_meta("chat-color", "red");

    let encoded = serde_json::to_string(&group).expect("serialize");
    let decoded: Group = serde_json::from_str(&encoded).expect("deserialize");

    assert_eq!(decoded.name(), "admin");
    assert_eq!(decoded.weight(), 75);
    assert_eq!(decoded.parents(), ["member"]);
    assert_eq!(decoded.nodes().len(), 2);
    assert_eq!(
        decoded.data().prefix().map(WeightedValue::value),
        Some("[A] ")
    );
    assert_eq!(
        decoded.data().meta().get("chat-color").map(String::as_str),
        Some("red")
    );
}

#[test]
fn user_round_trip() {
    let mut user = User::new(
        PlayerId::from_parts(0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210),
        "Rook",
    );
    user.set_primary_group("Admin");
    user.set_node(Node::new("worldedit.set", true));

    let encoded = serde_json::to_string(&user).expect("serialize");
    let decoded: User = serde_json::from_str(&encoded).expect("deserialize");

    assert_eq!(decoded.uuid(), user.uuid());
    assert_eq!(decoded.username(), "Rook");
    assert_eq!(decoded.primary_group(), "admin");
    assert_eq!(decoded.nodes().len(), 1);
}

#[test]
fn player_id_string_round_trip() {
    let id = PlayerId::from_parts(0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210);
    let text = id.to_string();
    assert_eq!(text, "01234567-89ab-cdef-fedc-ba9876543210");
    assert_eq!(text.parse::<PlayerId>().expect("parse"), id);
}
