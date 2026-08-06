mod commands;
mod context;
mod listeners;
mod permissions;
mod player;
mod state;
mod storage;
mod text;
mod time;

use pumpkin_plugin_api::permissions as host_permissions;
use pumpkin_plugin_api::{Context, Plugin, PluginMetadata, Result, register_plugin};
use rookperms_api::PermissionService;
use tracing::{error, info};

use crate::storage::JsonStorage;

const DATA_FOLDER: &str = "data";

struct RookPerms;

impl Plugin for RookPerms {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: permissions::NAMESPACE.into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["xRookieFight".into()],
            description: "Group based permission management plugin for PumpkinMC.".into(),
            dependencies: vec![],
            permissions: vec![
                host_permissions::FS_READ_DATA.into(),
                host_permissions::FS_WRITE_DATA.into(),
            ],
        }
    }

    fn on_load(&mut self, context: Context) -> Result<()> {
        let storage = JsonStorage::new(DATA_FOLDER);
        storage.prepare().map_err(|error| error.to_string())?;

        let mut service = PermissionService::new(Box::new(storage));
        service.load().map_err(|error| error.to_string())?;
        let groups = service.groups().len();
        state::install(service);

        permissions::register(&context)?;
        listeners::register(&context)?;
        commands::register(&context);

        info!("RookPerms loaded with {groups} group(s).");
        Ok(())
    }

    fn on_unload(&mut self, _context: Context) -> Result<()> {
        if let Some(Err(error)) = state::with_service(rookperms_api::PermissionService::save_all) {
            error!("failed to save permission data: {error}");
        }
        context::clear();
        state::clear();
        Ok(())
    }
}

register_plugin!(RookPerms);
