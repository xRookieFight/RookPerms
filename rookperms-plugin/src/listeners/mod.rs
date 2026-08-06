mod join;
mod leave;
mod permission_check;
mod world_change;

use pumpkin_plugin_api::events::EventPriority;
use pumpkin_plugin_api::events::player::{
    PlayerChangeWorldEvent, PlayerJoinEvent, PlayerLeaveEvent, PlayerPermissionCheckEvent,
};
use pumpkin_plugin_api::player::Player;
use pumpkin_plugin_api::{Context, Result};
use tracing::error;

use crate::player::{id_of, refresh_display_name, world_of};
use crate::{context, state};

use join::JoinListener;
use leave::LeaveListener;
use permission_check::PermissionCheckListener;
use world_change::WorldChangeListener;

pub fn register(context: &Context) -> Result<()> {
    context.register_event_handler::<PlayerJoinEvent, _>(
        JoinListener,
        EventPriority::Lowest,
        true,
    )?;
    context.register_event_handler::<PlayerLeaveEvent, _>(
        LeaveListener,
        EventPriority::Highest,
        true,
    )?;
    context.register_event_handler::<PlayerChangeWorldEvent, _>(
        WorldChangeListener,
        EventPriority::Lowest,
        true,
    )?;
    context.register_event_handler::<PlayerPermissionCheckEvent, _>(
        PermissionCheckListener,
        EventPriority::Highest,
        true,
    )?;
    Ok(())
}

pub fn synchronize(player: &Player) {
    let uuid = id_of(player);
    let username = player.get_name();

    let loaded = state::with_service(|service| service.load_user(uuid, &username));
    if let Some(Err(error)) = loaded {
        error!("failed to load permissions for {username}: {error}");
        return;
    }

    refresh(player);
}

pub fn refresh(player: &Player) {
    refresh_in(player, &world_of(player));
}

pub fn refresh_in(player: &Player, world: &str) {
    let uuid = id_of(player);
    context::track(uuid, world);

    let query = context::query_for(uuid);
    let resolved = state::with_service(|service| service.resolve(uuid, &query).cloned()).flatten();

    if let Some(resolved) = resolved {
        refresh_display_name(player, &resolved);
    }
}
