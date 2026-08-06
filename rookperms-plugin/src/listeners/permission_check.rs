use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::EventHandler;
use pumpkin_plugin_api::events::player::PlayerPermissionCheckEvent;
use pumpkin_plugin_api::events_wit::PlayerPermissionCheckEventData;

use crate::player::id_of;
use crate::{context, state};

pub struct PermissionCheckListener;

impl EventHandler<PlayerPermissionCheckEvent> for PermissionCheckListener {
    fn handle(
        &self,
        _server: Server,
        mut event: PlayerPermissionCheckEventData,
    ) -> PlayerPermissionCheckEventData {
        let uuid = id_of(&event.player);
        let query = context::query_for(uuid);

        let decision =
            state::with_service(|service| service.check(uuid, &event.permission, &query)).flatten();

        if let Some(value) = decision {
            event.permission_result = value;
        }

        event
    }
}
