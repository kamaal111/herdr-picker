use herdr_picker::{
    api::parse_snapshot,
    config::SearchMode,
    model::Target,
    search::{Row, Search},
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
    assert_eq!(result[0].parent.as_deref(), Some("TCG"));
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
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].label_matches, vec![0, 1, 2, 3, 4]);
    assert_eq!(result[0].parent_matches, vec![0, 1, 2]);
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
fn one_term_cannot_stitch_together_fragments_from_two_names() {
    assert!(rows("pickercod", SearchMode::Flat).is_empty());
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
        assert_eq!(combined[0].parent, None);
    }
    let result = rows("exact terminal", mode);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].label_matches, vec![0, 1, 2, 3, 4]);
    assert_eq!(result[0].inline_tab_matches, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    assert!(!result[0].active);
    assert!(rows("codexterminal", mode).is_empty());
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
