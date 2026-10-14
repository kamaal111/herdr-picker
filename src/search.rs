use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};

use crate::{
    config::SearchMode,
    model::{Snapshot, Target, Workspace},
    ordering,
};

#[derive(Clone, Debug)]
pub struct Ancestor {
    pub label: String,
    pub matches: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub target: Target,
    pub workspace_id: String,
    pub label: String,
    pub ancestors: Vec<Ancestor>,
    pub depth: usize,
    pub inline_tab: Option<String>,
    pub inline_tab_matches: Vec<usize>,
    pub label_matches: Vec<usize>,
    pub score: u32,
    pub active: bool,
}

#[derive(Default)]
pub struct Search {
    matcher: Matcher,
    buffer: Vec<char>,
}

impl Search {
    fn matched(&mut self, label: &str, term: &Atom) -> Option<(u32, Vec<usize>)> {
        self.buffer.clear();
        self.buffer.extend(label.chars());
        let haystack = Utf32Str::Unicode(&self.buffer);
        let mut indices = Vec::new();
        let score = term.indices(haystack, &mut self.matcher, &mut indices)?;
        Some((
            u32::from(score),
            indices.into_iter().map(|i| i as usize).collect(),
        ))
    }

    fn match_row(&mut self, row: &mut Row, terms: &[Atom]) -> bool {
        let labels: Vec<_> = row
            .ancestors
            .iter()
            .map(|ancestor| ancestor.label.as_str())
            .chain(std::iter::once(row.label.as_str()))
            .chain(row.inline_tab.as_deref())
            .collect();
        let mut matches = vec![Vec::new(); labels.len()];
        let mut score = 0;
        for term in terms {
            let best = labels
                .iter()
                .enumerate()
                .filter_map(|(level, label)| {
                    self.matched(label, term)
                        .map(|(score, indices)| (score, level, indices))
                })
                .max_by_key(|(score, level, _)| (*score, *level));
            let Some((term_score, level, indices)) = best else {
                return false;
            };
            score += term_score;
            matches[level].extend(indices);
        }
        for indices in &mut matches {
            indices.sort_unstable();
            indices.dedup();
        }
        for (ancestor, indices) in row.ancestors.iter_mut().zip(&matches) {
            ancestor.matches.clone_from(indices);
        }
        row.label_matches.clone_from(&matches[row.depth]);
        if row.inline_tab.is_some() {
            row.inline_tab_matches.clone_from(&matches[row.depth + 1]);
        }
        row.score = score;
        true
    }

    fn workspace_rows(
        &mut self,
        snapshot: &Snapshot,
        workspace: &Workspace,
        worktrees: &[&Workspace],
        ancestors: &[Ancestor],
        terms: &[Atom],
        mode: SearchMode,
    ) -> Vec<Row> {
        let tabs: Vec<_> = snapshot
            .tabs
            .iter()
            .filter(|tab| tab.workspace_id == workspace.workspace_id)
            .collect();
        let inline_tab = match tabs.as_slice() {
            [tab] => Some(tab.label.clone()),
            _ => None,
        };
        let mut row = Row {
            target: Target::Workspace(workspace.workspace_id.clone()),
            workspace_id: workspace.workspace_id.clone(),
            label: workspace.label.clone(),
            ancestors: ancestors.to_vec(),
            depth: ancestors.len(),
            inline_tab,
            inline_tab_matches: Vec::new(),
            label_matches: Vec::new(),
            score: 0,
            active: snapshot.focused_workspace_id.as_ref() == Some(&workspace.workspace_id),
        };
        let matched = self.match_row(&mut row, terms);
        let mut child_ancestors = ancestors.to_vec();
        child_ancestors.push(Ancestor {
            label: workspace.label.clone(),
            matches: Vec::new(),
        });
        let mut children = Vec::new();
        if row.inline_tab.is_none() {
            for tab in tabs {
                let mut child = Row {
                    target: Target::Tab(tab.tab_id.clone()),
                    workspace_id: workspace.workspace_id.clone(),
                    label: tab.label.clone(),
                    ancestors: child_ancestors.clone(),
                    depth: child_ancestors.len(),
                    inline_tab: None,
                    inline_tab_matches: Vec::new(),
                    label_matches: Vec::new(),
                    score: 0,
                    active: snapshot.focused_tab_id.as_ref() == Some(&tab.tab_id),
                };
                if self.match_row(&mut child, terms) {
                    children.push(child);
                }
            }
        }
        for worktree in worktrees {
            children.extend(self.workspace_rows(
                snapshot,
                worktree,
                &[],
                &child_ancestors,
                terms,
                mode,
            ));
        }
        let mut rows = Vec::new();
        if matched || (mode == SearchMode::Hierarchical && !children.is_empty()) {
            if mode == SearchMode::Hierarchical {
                for child in &children {
                    for (ancestor, context) in row.ancestors.iter_mut().zip(&child.ancestors) {
                        ancestor.matches.extend(&context.matches);
                        ancestor.matches.sort_unstable();
                        ancestor.matches.dedup();
                    }
                    row.label_matches
                        .extend(&child.ancestors[row.depth].matches);
                }
                row.label_matches.sort_unstable();
                row.label_matches.dedup();
            }
            rows.push(row);
        }
        rows.extend(children);
        rows
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
        for group in ordering::groups(snapshot) {
            rows.extend(self.workspace_rows(
                snapshot,
                group.root,
                &group.worktrees,
                &[],
                &terms,
                mode,
            ));
        }
        if mode == SearchMode::Flat && !terms.is_empty() {
            rows.sort_by_key(|r| std::cmp::Reverse(r.score));
        }
        rows
    }
}
