use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};

use crate::{
    config::SearchMode,
    model::{Snapshot, Target},
    ordering,
};

#[derive(Clone, Debug)]
pub struct Row {
    pub target: Target,
    pub workspace_id: String,
    pub label: String,
    pub parent: Option<String>,
    pub inline_tab: Option<String>,
    pub inline_tab_matches: Vec<usize>,
    pub label_matches: Vec<usize>,
    pub parent_matches: Vec<usize>,
    pub score: u32,
    pub active: bool,
}

#[derive(Default)]
pub struct Search {
    matcher: Matcher,
    buffer: Vec<char>,
}

impl Search {
    fn matched(&mut self, label: &str, terms: &[Atom]) -> Option<(u32, Vec<usize>)> {
        self.buffer.clear();
        self.buffer.extend(label.chars());
        let haystack = Utf32Str::Unicode(&self.buffer);
        let mut indices = Vec::new();
        let mut score = 0;
        for term in terms {
            score += u32::from(term.indices(haystack, &mut self.matcher, &mut indices)?);
        }
        indices.sort_unstable();
        indices.dedup();
        Some((score, indices.into_iter().map(|i| i as usize).collect()))
    }

    fn hierarchy_match(
        &mut self,
        parent: &str,
        label: &str,
        terms: &[Atom],
    ) -> Option<HierarchyMatch> {
        let mut result = HierarchyMatch::default();
        for term in terms {
            let parent_match = self.matched(parent, std::slice::from_ref(term));
            let label_match = self.matched(label, std::slice::from_ref(term));
            match (parent_match, label_match) {
                (Some((parent_score, parent_indices)), Some((label_score, label_indices))) => {
                    if label_score >= parent_score {
                        result.score += label_score;
                        result.label.extend(label_indices);
                    } else {
                        result.score += parent_score;
                        result.parent.extend(parent_indices);
                    }
                }
                (Some((score, indices)), None) => {
                    result.score += score;
                    result.parent.extend(indices);
                }
                (None, Some((score, indices))) => {
                    result.score += score;
                    result.label.extend(indices);
                }
                (None, None) => return None,
            }
        }
        Some(result)
    }

    pub fn rows(&mut self, snapshot: &Snapshot, query: &str, mode: SearchMode) -> Vec<Row> {
        let terms: Vec<_> = query
            .split_whitespace()
            .map(|term| {
                Atom::new(
                    term,
                    CaseMatching::Ignore,
                    Normalization::Smart,
                    AtomKind::Fuzzy,
                    false,
                )
            })
            .collect();
        let mut rows = Vec::new();
        for workspace in ordering::workspaces(snapshot) {
            let tabs: Vec<_> = snapshot
                .tabs
                .iter()
                .filter(|tab| tab.workspace_id == workspace.workspace_id)
                .collect();
            if let [tab] = tabs.as_slice() {
                if let Some(matched) = self.hierarchy_match(&workspace.label, &tab.label, &terms) {
                    rows.push(Row {
                        target: Target::Workspace(workspace.workspace_id.clone()),
                        workspace_id: workspace.workspace_id.clone(),
                        label: workspace.label.clone(),
                        parent: None,
                        inline_tab: Some(tab.label.clone()),
                        inline_tab_matches: matched.label,
                        label_matches: matched.parent,
                        parent_matches: Vec::new(),
                        score: matched.score,
                        active: snapshot.focused_workspace_id.as_ref()
                            == Some(&workspace.workspace_id),
                    });
                }
                continue;
            }
            let workspace_match = self.matched(&workspace.label, &terms);
            let mut children = Vec::new();
            let mut context_matches = Vec::new();
            for tab in tabs {
                if let Some(matched) = self.hierarchy_match(&workspace.label, &tab.label, &terms) {
                    context_matches.extend(&matched.parent);
                    children.push(Row {
                        target: Target::Tab(tab.tab_id.clone()),
                        workspace_id: workspace.workspace_id.clone(),
                        label: tab.label.clone(),
                        parent: Some(workspace.label.clone()),
                        inline_tab: None,
                        inline_tab_matches: Vec::new(),
                        label_matches: matched.label,
                        parent_matches: matched.parent,
                        score: matched.score,
                        active: snapshot.focused_tab_id.as_ref() == Some(&tab.tab_id),
                    });
                }
            }
            if workspace_match.is_some()
                || (mode == SearchMode::Hierarchical && !children.is_empty())
            {
                let (score, label_matches) = workspace_match.unwrap_or((0, context_matches));
                rows.push(Row {
                    target: Target::Workspace(workspace.workspace_id.clone()),
                    workspace_id: workspace.workspace_id.clone(),
                    label: workspace.label.clone(),
                    parent: None,
                    inline_tab: None,
                    inline_tab_matches: Vec::new(),
                    label_matches,
                    parent_matches: Vec::new(),
                    score,
                    active: snapshot.focused_workspace_id.as_ref() == Some(&workspace.workspace_id),
                });
            }
            rows.extend(children);
        }
        if mode == SearchMode::Flat && !terms.is_empty() {
            rows.sort_by_key(|r| std::cmp::Reverse(r.score));
        }
        rows
    }
}

#[derive(Default)]
struct HierarchyMatch {
    score: u32,
    label: Vec<usize>,
    parent: Vec<usize>,
}
