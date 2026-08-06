use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::player::Player;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::uuid::Uuid;
use rookperms_api::id::PlayerId;
use rookperms_api::resolver::ResolvedPermissions;

use crate::text::colored;

pub fn player_id(uuid: Uuid) -> PlayerId {
    PlayerId::from_parts(uuid.high, uuid.low)
}

pub fn id_of(player: &Player) -> PlayerId {
    player_id(player.get_id())
}

pub fn world_of(player: &Player) -> String {
    player.get_world().get_id()
}

pub fn refresh_display_name(player: &Player, resolved: &ResolvedPermissions) {
    let prefix = resolved.prefix().unwrap_or_default();
    let suffix = resolved.suffix().unwrap_or_default();
    if prefix.is_empty() && suffix.is_empty() {
        return;
    }

    let display = TextComponent::text("");
    if !prefix.is_empty() {
        display.add_child(TextComponent::text(prefix));
    }
    display.add_child(colored(&player.get_name(), NamedColor::White));
    if !suffix.is_empty() {
        display.add_child(TextComponent::text(suffix));
    }
    player.set_display_name(display);
}
