use std::collections::HashSet;

use anyhow::{Result, ensure};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Workspace {
    pub workspace_id: String,
    pub label: String,
    pub number: usize,
    pub active_tab_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Tab {
    pub tab_id: String,
    pub workspace_id: String,
    pub label: String,
    pub number: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Snapshot {
    pub workspaces: Vec<Workspace>,
    pub tabs: Vec<Tab>,
    pub focused_workspace_id: Option<String>,
    pub focused_tab_id: Option<String>,
}

impl Snapshot {
    pub fn validate(&mut self) -> Result<()> {
        let mut workspace_ids = HashSet::new();
        for workspace in &self.workspaces {
            ensure!(
                !workspace.workspace_id.trim().is_empty(),
                "missing workspace ID"
            );
            ensure!(
                workspace_ids.insert(&workspace.workspace_id),
                "duplicate workspace ID"
            );
        }
        let mut tab_ids = HashSet::new();
        for tab in &self.tabs {
            ensure!(!tab.tab_id.trim().is_empty(), "missing tab ID");
            ensure!(tab_ids.insert(&tab.tab_id), "duplicate tab ID");
            ensure!(
                workspace_ids.contains(&tab.workspace_id),
                "tab has unknown parent workspace"
            );
        }
        for workspace in &self.workspaces {
            ensure!(
                self.tabs
                    .iter()
                    .any(|tab| tab.tab_id == workspace.active_tab_id
                        && tab.workspace_id == workspace.workspace_id),
                "workspace has invalid active tab ID"
            );
        }
        ensure!(
            self.focused_workspace_id
                .as_ref()
                .is_none_or(|id| workspace_ids.contains(id)),
            "unknown focused workspace ID"
        );
        ensure!(
            self.focused_tab_id.as_ref().is_none_or(|id| self
                .tabs
                .iter()
                .any(|tab| &tab.tab_id == id
                    && Some(&tab.workspace_id) == self.focused_workspace_id.as_ref())),
            "invalid focused tab ID"
        );
        self.workspaces.sort_by_key(|w| w.number);
        self.tabs.sort_by_key(|t| t.number);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Workspace(String),
    Tab(String),
}
