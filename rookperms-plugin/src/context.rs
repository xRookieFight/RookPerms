use std::collections::BTreeMap;
use std::sync::Mutex;

use rookperms_api::context::{CONTEXT_WORLD, ContextSet};
use rookperms_api::id::PlayerId;
use rookperms_api::resolver::QueryContext;

use crate::time::now_unix;

static CONTEXTS: Mutex<BTreeMap<PlayerId, ContextSet>> = Mutex::new(BTreeMap::new());

pub fn track(uuid: PlayerId, world: &str) {
    let contexts = ContextSet::new().with(CONTEXT_WORLD, world);
    if let Ok(mut guard) = CONTEXTS.lock() {
        guard.insert(uuid, contexts);
    }
}

pub fn forget(uuid: PlayerId) {
    if let Ok(mut guard) = CONTEXTS.lock() {
        guard.remove(&uuid);
    }
}

pub fn clear() {
    if let Ok(mut guard) = CONTEXTS.lock() {
        guard.clear();
    }
}

pub fn query_for(uuid: PlayerId) -> QueryContext {
    let contexts = CONTEXTS
        .lock()
        .ok()
        .and_then(|guard| guard.get(&uuid).cloned())
        .unwrap_or_default();
    QueryContext::new(contexts, now_unix())
}
