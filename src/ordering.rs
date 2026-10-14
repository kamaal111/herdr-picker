//! Public snapshots have no MRU history. Keep a deterministic, explicit fallback.
use crate::model::{Snapshot, Workspace};

pub fn workspaces(snapshot: &Snapshot) -> Vec<&Workspace> {
    let mut workspaces: Vec<_> = snapshot.workspaces.iter().collect();
    workspaces.sort_by_key(|w| {
        (
            Some(&w.workspace_id) != snapshot.focused_workspace_id.as_ref(),
            w.number,
        )
    });
    workspaces
}

pub fn initial_target(snapshot: &Snapshot) -> Option<&str> {
    let ordered = workspaces(snapshot);
    ordered
        .iter()
        .find(|w| Some(&w.workspace_id) != snapshot.focused_workspace_id.as_ref())
        .or_else(|| ordered.first())
        .map(|w| w.workspace_id.as_str())
}
