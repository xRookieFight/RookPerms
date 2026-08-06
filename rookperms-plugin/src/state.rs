use std::sync::Mutex;

use rookperms_api::PermissionService;

static SERVICE: Mutex<Option<PermissionService>> = Mutex::new(None);

pub fn install(service: PermissionService) {
    replace(Some(service));
}

pub fn clear() {
    replace(None);
}

pub fn with_service<R>(action: impl FnOnce(&mut PermissionService) -> R) -> Option<R> {
    let mut guard = SERVICE.lock().ok()?;
    guard.as_mut().map(action)
}

fn replace(service: Option<PermissionService>) {
    match SERVICE.lock() {
        Ok(mut guard) => *guard = service,
        Err(poisoned) => *poisoned.into_inner() = service,
    }
}
