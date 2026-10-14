use herdr_picker::{
    api::parse_snapshot,
    config::SearchMode,
    model::Target,
    search::{Row, RowKind, Search},
};

fn rows(query: &str, mode: SearchMode) -> Vec<Row> {
    let snapshot = parse_snapshot(include_bytes!("fixtures/snapshot.json")).unwrap();
    Search::default().rows(&snapshot, query, mode)
}
fn ids(rows: &[Row]) -> Vec<Target> {
    rows.iter().map(|r| r.target.clone()).collect()
}
fn workspace(id: &str) -> Target {
    Target::Workspace(id.into())
}
fn tab(id: &str) -> Target {
    Target::Tab(id.into())
}

#[test]
fn workspace_match_reveals_all_tabs_in_ui_order() {
    assert_eq!(
        ids(&rows("ColorPicker", SearchMode::Hierarchical)),
        vec![workspace("w1"), tab("w1:t4"), tab("w1:t8"), tab("w1:t9")]
    );
}
#[test]
fn noncontiguous_case_insensitive_workspace_match() {
    assert_eq!(rows("CLP", SearchMode::Hierarchical).len(), 4);
}
#[test]
fn partial_tab_matches_include_selectable_parent() {
    assert_eq!(
        ids(&rows("serv", SearchMode::Hierarchical)),
        vec![workspace("w2"), tab("w2:t2")]
    );
}
#[test]
fn tab_matches_keep_group_order_instead_of_fuzzy_rank() {
    assert_eq!(
        ids(&rows("codex", SearchMode::Hierarchical)),
        vec![
            workspace("w2"),
            tab("w2:t7"),
            workspace("w1"),
            tab("w1:t8"),
            workspace("w3")
        ]
    );
}
#[test]
fn query_terms_match_across_parent_and_child() {
    assert_eq!(
        ids(&rows("color cod", SearchMode::Hierarchical)),
        vec![workspace("w1"), tab("w1:t4"), tab("w1:t8")]
    );
}
#[test]
fn full_names_match_across_hierarchy() {
    assert_eq!(
        ids(&rows("colorpicker codex", SearchMode::Hierarchical)),
        vec![workspace("w1"), tab("w1:t8")]
    );
}
#[test]
fn multiple_terms_have_and_semantics() {
    assert!(
        rows(
            "colorpicker randomnonexistentterm",
            SearchMode::Hierarchical
        )
        .is_empty()
    );
}
#[test]
fn no_results() {
    assert!(rows("zzzzzzz", SearchMode::Flat).is_empty());
}
#[test]
fn empty_query_has_every_selectable_item() {
    assert_eq!(rows("", SearchMode::Hierarchical).len(), 8);
}
#[test]
fn whitespace_query_has_every_selectable_item() {
    assert_eq!(rows("  \t", SearchMode::Flat).len(), 8);
}
#[test]
fn branch_metadata_is_not_searchable() {
    assert!(rows("secret-branch", SearchMode::Flat).is_empty());
}
#[test]
fn paths_are_not_searchable() {
    assert!(rows("secret/path", SearchMode::Flat).is_empty());
}
#[test]
fn flat_workspace_result_is_independent() {
    assert_eq!(
        ids(&rows("TCG", SearchMode::Flat)),
        vec![workspace("w2"), tab("w2:t7"), tab("w2:t2")]
    );
}
#[test]
fn flat_tab_retains_parent_context() {
    let result = rows("server", SearchMode::Flat);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ancestors[0].label, "TCG");
    assert_eq!(result[0].target, tab("w2:t2"));
}
#[test]
fn flat_rank_puts_exact_name_above_longer_matches() {
    let result = rows("Codex", SearchMode::Flat);
    assert_eq!(result[0].label, "Codex");
    assert!(result.windows(2).all(|r| r[0].score >= r[1].score));
}
#[test]
fn highlights_match_both_hierarchy_labels() {
    let result = rows("col codex", SearchMode::Flat);
    assert_eq!(ids(&result), vec![tab("w1:t8"), workspace("w3")]);
    let codex = result
        .iter()
        .find(|row| row.target == tab("w1:t8"))
        .unwrap();
    assert_eq!(codex.label_matches, vec![0, 1, 2, 3, 4]);
    assert_eq!(codex.ancestors[0].matches, vec![0, 1, 2]);
}
#[test]
fn query_term_order_is_independent() {
    assert_eq!(
        ids(&rows("cod color", SearchMode::Flat)),
        ids(&rows("color cod", SearchMode::Flat))
    );
}
#[test]
fn duplicate_display_names_keep_distinct_ids() {
    let result = rows("codex", SearchMode::Flat);
    assert!(ids(&result).contains(&tab("w1:t8")));
    assert!(ids(&result).contains(&tab("w2:t7")));
}

#[test]
fn one_term_matches_across_parent_and_child_without_spaces() {
    let result = rows("pickercod", SearchMode::Flat);
    assert_eq!(ids(&result), vec![tab("w1:t8"), tab("w1:t4")]);
    let codex = result
        .iter()
        .find(|row| row.target == tab("w1:t8"))
        .unwrap();
    assert_eq!(codex.ancestors[0].matches, (5..11).collect::<Vec<_>>());
    assert_eq!(codex.label_matches, vec![0, 1, 2]);
    assert_eq!(
        ids(&rows("pickercod", SearchMode::Hierarchical)),
        vec![workspace("w1"), tab("w1:t4"), tab("w1:t8")]
    );
}

#[test]
fn unicode_match_indices_address_visible_codepoints() {
    let mut snapshot = parse_snapshot(include_bytes!("fixtures/snapshot.json")).unwrap();
    snapshot.workspaces[0].label = "日本ColorPic".into();
    let result = Search::default().rows(&snapshot, "colorpic", SearchMode::Hierarchical);
    assert_eq!(result[0].label_matches, vec![2, 3, 4, 5, 6, 7, 8, 9]);
}

fn assert_single_tab_workspace(mode: SearchMode) {
    for query in ["", "  ", "exact", "terminal", "exact terminal"] {
        let result = rows(query, mode);
        let combined: Vec<_> = result
            .iter()
            .filter(|row| row.workspace_id == "w3")
            .collect();
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].target, workspace("w3"));
        assert_eq!(combined[0].label, "Exact Codex");
        assert_eq!(combined[0].inline_tab.as_deref(), Some("Terminal"));
        assert!(combined[0].ancestors.is_empty());
    }
    let result = rows("exact terminal", mode);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].label_matches, vec![0, 1, 2, 3, 4]);
    assert_eq!(result[0].inline_tab_matches, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    assert!(!result[0].active);
    assert_eq!(ids(&rows("codexterminal", mode)), vec![workspace("w3")]);
    assert!(rows("exact nonexistent", mode).is_empty());
}

#[test]
fn hierarchical_single_tab_workspace_is_one_searchable_row() {
    assert_single_tab_workspace(SearchMode::Hierarchical);
}

#[test]
fn flat_single_tab_workspace_is_one_searchable_row() {
    assert_single_tab_workspace(SearchMode::Flat);
}

#[test]
fn filtered_multi_tab_workspace_does_not_become_inline() {
    let hierarchical = rows("server", SearchMode::Hierarchical);
    assert_eq!(ids(&hierarchical), vec![workspace("w2"), tab("w2:t2")]);
    assert!(hierarchical.iter().all(|row| row.inline_tab.is_none()));
    let flat = rows("server", SearchMode::Flat);
    assert_eq!(ids(&flat), vec![tab("w2:t2")]);
    assert_eq!(flat[0].inline_tab, None);
}

fn worktree_rows(query: &str, mode: SearchMode) -> Vec<Row> {
    let snapshot = parse_snapshot(include_bytes!("fixtures/worktrees.json")).unwrap();
    Search::default().rows(&snapshot, query, mode)
}

#[test]
fn worktrees_are_nested_once_after_root_tabs_in_ui_order() {
    assert_eq!(
        ids(&worktree_rows("", SearchMode::Hierarchical)),
        vec![
            workspace("w1"),
            tab("w1:t1"),
            tab("w1:t2"),
            workspace("w2"),
            tab("w2:t1"),
            tab("w2:t2"),
            workspace("w3"),
            workspace("w4"),
            workspace("w5"),
            tab("w5:t1"),
            tab("w5:t2"),
            workspace("w0")
        ]
    );
}

#[test]
fn root_search_reveals_worktrees_and_their_tabs() {
    assert_eq!(
        worktree_rows("herdr-picker", SearchMode::Hierarchical).len(),
        11
    );
}

#[test]
fn worktree_search_retains_selectable_root_and_inline_tab() {
    let result = worktree_rows("just", SearchMode::Hierarchical);
    assert_eq!(ids(&result), vec![workspace("w1"), workspace("w4")]);
    assert_eq!(result[1].inline_tab.as_deref(), Some("Terminal"));
}

#[test]
fn three_level_search_keeps_only_matching_branch_and_ancestors() {
    assert_eq!(
        ids(&worktree_rows(
            "herdr-picker multimachine codex",
            SearchMode::Hierarchical
        )),
        vec![workspace("w1"), workspace("w2"), tab("w2:t1")]
    );
    assert_eq!(
        ids(&worktree_rows(
            "herdr-picker just terminal",
            SearchMode::Hierarchical
        )),
        vec![workspace("w1"), workspace("w4")]
    );
    assert!(worktree_rows("herdr-picker just nonexistent", SearchMode::Hierarchical).is_empty());
    assert_eq!(
        ids(&worktree_rows("pickermultimachine", SearchMode::Flat)),
        vec![workspace("w2"), tab("w2:t1"), tab("w2:t2")]
    );
}

#[test]
fn nested_rows_have_full_ancestor_context_and_depth() {
    let result = worktree_rows("herdr-picker multimachine codex", SearchMode::Hierarchical);
    assert_eq!(
        result.iter().map(|row| row.depth).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        result[2]
            .ancestors
            .iter()
            .map(|a| a.label.as_str())
            .collect::<Vec<_>>(),
        vec!["herdr-picker", "multimachine"]
    );
    assert_eq!(result[0].label_matches, (0..12).collect::<Vec<_>>());
    assert_eq!(result[1].label_matches, (0..12).collect::<Vec<_>>());
    assert_eq!(result[2].label_matches, (0..5).collect::<Vec<_>>());
}

#[test]
fn flat_nested_tab_matches_and_highlights_all_three_names() {
    let result = worktree_rows("herdr-picker multimachine codex", SearchMode::Flat);
    assert_eq!(ids(&result), vec![tab("w2:t1")]);
    assert_eq!(result[0].ancestors[0].label, "herdr-picker");
    assert_eq!(result[0].ancestors[1].label, "multimachine");
    assert_eq!(result[0].ancestors[0].matches, (0..12).collect::<Vec<_>>());
    assert_eq!(result[0].ancestors[1].matches, (0..12).collect::<Vec<_>>());
    assert_eq!(result[0].label_matches, (0..5).collect::<Vec<_>>());
}

#[test]
fn flat_worktree_results_are_independent_and_preserve_inline_tab_search() {
    let result = worktree_rows("herdr-picker just terminal", SearchMode::Flat);
    assert_eq!(ids(&result), vec![workspace("w4")]);
    assert_eq!(result[0].ancestors[0].label, "herdr-picker");
    assert_eq!(result[0].inline_tab.as_deref(), Some("Terminal"));
    assert_eq!(result[0].inline_tab_matches, (0..8).collect::<Vec<_>>());
}

#[test]
fn equal_name_scores_highlight_the_deepest_matching_label() {
    let mut snapshot = parse_snapshot(include_bytes!("fixtures/worktrees.json")).unwrap();
    snapshot.workspaces[1].label = "Codex".into();
    snapshot.workspaces[2].label = "Codex".into();
    let result = Search::default().rows(&snapshot, "codex", SearchMode::Flat);
    let tab = result
        .iter()
        .find(|row| row.target == Target::Tab("w2:t1".into()))
        .unwrap();
    assert!(
        tab.ancestors
            .iter()
            .all(|ancestor| ancestor.matches.is_empty())
    );
    assert_eq!(tab.label_matches, (0..5).collect::<Vec<_>>());
}

#[test]
fn single_root_tab_is_not_search_context_for_other_worktrees() {
    let mut snapshot = parse_snapshot(include_bytes!("fixtures/worktrees.json")).unwrap();
    snapshot.tabs.retain(|tab| tab.tab_id != "w1:t2");
    let result = Search::default().rows(&snapshot, "terminal", SearchMode::Hierarchical);
    assert_eq!(result[0].inline_tab.as_deref(), Some("Terminal"));
    assert!(
        result
            .iter()
            .all(|row| row.target != Target::Tab("w2:t1".into()))
    );
    assert!(
        result
            .iter()
            .all(|row| row.target != Target::Tab("w5:t1".into()))
    );
}

#[test]
fn nested_rows_and_breadcrumbs_retain_destination_kinds() {
    let result = worktree_rows("herdr-picker multimachine codex", SearchMode::Hierarchical);
    assert_eq!(
        result.iter().map(|row| row.kind).collect::<Vec<_>>(),
        vec![RowKind::RootWorkspace, RowKind::Worktree, RowKind::Tab]
    );
    assert_eq!(
        result[2]
            .ancestors
            .iter()
            .map(|ancestor| ancestor.kind)
            .collect::<Vec<_>>(),
        vec![RowKind::RootWorkspace, RowKind::Worktree]
    );
    let flat = worktree_rows("herdr-picker multimachine codex", SearchMode::Flat);
    assert_eq!(flat[0].kind, RowKind::Tab);
    assert_eq!(flat[0].ancestors[0].kind, RowKind::RootWorkspace);
    assert_eq!(flat[0].ancestors[1].kind, RowKind::Worktree);
}

#[test]
fn ungrouped_linked_worktree_uses_root_kind() {
    let mut snapshot = parse_snapshot(include_bytes!("fixtures/worktrees.json")).unwrap();
    snapshot
        .workspaces
        .retain(|workspace| workspace.workspace_id != "w1");
    snapshot.tabs.retain(|tab| tab.workspace_id != "w1");
    let result = Search::default().rows(&snapshot, "multimachine codex", SearchMode::Flat);
    assert_eq!(result[0].kind, RowKind::Tab);
    assert_eq!(result[0].ancestors.len(), 1);
    assert_eq!(result[0].ancestors[0].kind, RowKind::RootWorkspace);
}

#[test]
fn space_free_three_level_search_highlights_each_label() {
    let query = "herdr-pickermultimachinecodex";
    let result = worktree_rows(query, SearchMode::Flat);
    assert_eq!(ids(&result), vec![tab("w2:t1")]);
    assert_eq!(result[0].ancestors[0].matches, (0..12).collect::<Vec<_>>());
    assert_eq!(result[0].ancestors[1].matches, (0..12).collect::<Vec<_>>());
    assert_eq!(result[0].label_matches, (0..5).collect::<Vec<_>>());
    assert_eq!(
        ids(&worktree_rows(query, SearchMode::Hierarchical)),
        vec![workspace("w1"), workspace("w2"), tab("w2:t1")]
    );
}

#[test]
fn space_free_fuzzy_search_can_skip_an_intermediate_label() {
    assert_eq!(
        ids(&worktree_rows("hdrpkrmltmchn", SearchMode::Flat)),
        vec![workspace("w2"), tab("w2:t1"), tab("w2:t2")]
    );
    assert_eq!(
        ids(&worktree_rows("herdr-pickercodex", SearchMode::Flat)),
        vec![tab("w2:t1"), tab("w5:t1")]
    );
}

#[test]
fn space_free_search_cannot_reverse_labels_or_combine_siblings() {
    assert!(rows("codexpicker", SearchMode::Flat).is_empty());
    assert!(rows("servercodex", SearchMode::Hierarchical).is_empty());
    assert!(worktree_rows("multimachinejust", SearchMode::Flat).is_empty());
    assert!(rows("tcgcolorpicker", SearchMode::Flat).is_empty());
}

#[test]
fn space_free_search_maps_unicode_indices_to_each_label() {
    let mut snapshot = parse_snapshot(include_bytes!("fixtures/snapshot.json")).unwrap();
    snapshot.workspaces[2].label = "日本Exact".into();
    snapshot
        .tabs
        .iter_mut()
        .find(|tab| tab.tab_id == "w3:t1")
        .unwrap()
        .label = "世界Terminal".into();
    let result = Search::default().rows(&snapshot, "exactterminal", SearchMode::Flat);
    assert_eq!(ids(&result), vec![workspace("w3")]);
    assert_eq!(result[0].label_matches, (2..7).collect::<Vec<_>>());
    assert_eq!(result[0].inline_tab_matches, (2..10).collect::<Vec<_>>());
}

#[test]
fn space_free_terms_keep_and_semantics_and_order_independence() {
    assert_eq!(
        ids(&rows("pickercod codex", SearchMode::Flat)),
        vec![tab("w1:t8")]
    );
    assert_eq!(
        ids(&rows("codex pickercod", SearchMode::Flat)),
        vec![tab("w1:t8")]
    );
    assert!(rows("pickercod nonexistent", SearchMode::Flat).is_empty());
}
