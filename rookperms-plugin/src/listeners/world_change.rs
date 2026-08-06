use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::EventHandler;
use pumpkin_plugin_api::events::player::PlayerChangeWorldEvent;
use pumpkin_plugin_api::events_wit::PlayerChangeWorldEventData;

pub struct WorldChangeListener;

impl EventHandler<PlayerChangeWorldEvent> for WorldChangeListener {
    fn handle(
        &self,
        _server: Server,
        event: PlayerChangeWorldEventData,
    ) -> PlayerChangeWorldEventData {
        super::refresh_in(&event.player, &event.new_world.get_id());
        event
    }
}
