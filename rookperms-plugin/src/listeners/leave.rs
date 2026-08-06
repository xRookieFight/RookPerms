use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::EventHandler;
use pumpkin_plugin_api::events::player::PlayerLeaveEvent;
use pumpkin_plugin_api::events_wit::PlayerLeaveEventData;
use tracing::error;

use crate::player::id_of;
use crate::{context, state};

pub struct LeaveListener;

impl EventHandler<PlayerLeaveEvent> for LeaveListener {
    fn handle(&self, _server: Server, event: PlayerLeaveEventData) -> PlayerLeaveEventData {
        let uuid = id_of(&event.player);

        if let Some(Err(error)) = state::with_service(|service| service.unload_user(uuid)) {
            error!("failed to save permissions for {uuid}: {error}");
        }
        context::forget(uuid);

        event
    }
}
