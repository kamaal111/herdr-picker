use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use herdr_picker::{
    api::parse_snapshot,
    app::{Action, App},
    config::SearchMode,
    model::Target,
};
use ratatui::layout::Rect;

fn app() -> App {
    App::new(
        parse_snapshot(include_bytes!("fixtures/snapshot.json")).unwrap(),
        SearchMode::Hierarchical,
    )
}
fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn mouse(kind: MouseEventKind, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column: 5,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

#[test]
fn initial_selection_uses_first_other_workspace_when_no_mru_available() {
    let mut app = app();
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Workspace("w1".into()))
    );
}
#[test]
fn initial_order_is_current_workspace_then_ui_order() {
    assert_eq!(app().rows[0].target, Target::Workspace("w2".into()));
}
#[test]
fn search_selects_first_matching_parent() {
    let mut app = app();
    app.set_query("codex".into());
    assert_eq!(app.selected, Some(0));
    assert_eq!(app.offset, 0);
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Workspace("w2".into()))
    );
}
#[test]
fn navigation_includes_workspace_and_tab() {
    let mut app = app();
    app.set_query("colorpicker".into());
    app.handle(key(KeyCode::Down));
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Tab("w1:t4".into()))
    );
    app.handle(key(KeyCode::Up));
    assert_eq!(app.selected, Some(0));
}
#[test]
fn control_n_and_p_navigate() {
    let mut app = app();
    app.set_query("colorpicker".into());
    app.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('n'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(app.selected, Some(1));
    app.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('p'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(app.selected, Some(0));
}
#[test]
fn empty_results_ignore_enter_and_navigation() {
    let mut app = app();
    app.set_query("nonexistenttttt".into());
    app.navigate(1);
    app.navigate(-1);
    assert_eq!(app.selected, None);
    assert_eq!(app.handle(key(KeyCode::Enter)), Action::Continue);
}
#[test]
fn escape_closes_immediately_without_clearing_query() {
    let mut app = app();
    app.set_query("color".into());
    assert_eq!(app.handle(key(KeyCode::Esc)), Action::Close);
    assert_eq!(app.query, "color");
}
#[test]
fn typing_filters_immediately_without_switching() {
    let mut app = app();
    assert_eq!(app.handle(key(KeyCode::Char('c'))), Action::Continue);
    assert_eq!(app.query, "c");
    assert_eq!(app.selected, Some(0));
}
#[test]
fn backspace_updates_query_and_selection() {
    let mut app = app();
    app.set_query("color".into());
    app.navigate(1);
    app.handle(key(KeyCode::Backspace));
    assert_eq!(app.query, "colo");
    assert_eq!(app.selected, Some(0));
}
#[test]
fn search_reset_and_resize_keep_selection_visible() {
    let mut app = app();
    app.navigate(100);
    app.ensure_visible(2);
    assert_eq!(app.offset, 6);
    app.set_query("cod".into());
    app.ensure_visible(1);
    assert_eq!(app.offset, 0);
}
#[test]
fn left_and_right_do_not_change_tree_state() {
    let mut app = app();
    let selected = app.selected;
    app.handle(key(KeyCode::Left));
    app.handle(key(KeyCode::Right));
    assert_eq!(app.selected, selected);
    assert_eq!(app.rows.len(), 8);
}
#[test]
fn mouse_click_switches_the_visible_row() {
    let mut app = app();
    app.results_area = Rect::new(0, 2, 40, 5);
    assert_eq!(
        app.handle(mouse(MouseEventKind::Down(MouseButton::Left), 3)),
        Action::Switch(Target::Tab("w2:t7".into()))
    );
}
#[test]
fn mouse_wheel_scrolls_and_keyboard_continues_from_selection() {
    let mut app = app();
    app.results_area = Rect::new(0, 2, 40, 5);
    app.set_query("".into());
    app.handle(mouse(MouseEventKind::ScrollDown, 3));
    app.ensure_visible(2);
    assert_eq!(app.selected, Some(3));
    assert_eq!(app.offset, 2);
    app.handle(key(KeyCode::Up));
    assert_eq!(app.selected, Some(2));
}
#[test]
fn reopening_has_no_previous_query() {
    assert_eq!(app().query, "");
}

#[test]
fn single_workspace_fallback_selects_its_workspace_row() {
    let snapshot = herdr_picker::model::Snapshot {
        workspaces: vec![herdr_picker::model::Workspace {
            workspace_id: "w1".into(),
            label: "Only".into(),
            number: 1,
            active_tab_id: "w1:t1".into(),
            worktree: None,
        }],
        tabs: vec![herdr_picker::model::Tab {
            tab_id: "w1:t1".into(),
            workspace_id: "w1".into(),
            label: "Terminal".into(),
            number: 1,
        }],
        focused_workspace_id: Some("w1".into()),
        focused_tab_id: Some("w1:t1".into()),
    };
    let mut app = App::new(snapshot, SearchMode::Hierarchical);
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Workspace("w1".into()))
    );
}

#[test]
fn empty_session_has_no_selection_and_can_close() {
    let snapshot = herdr_picker::model::Snapshot {
        workspaces: vec![],
        tabs: vec![],
        focused_workspace_id: None,
        focused_tab_id: None,
    };
    let mut app = App::new(snapshot, SearchMode::Flat);
    assert_eq!(app.selected, None);
    assert_eq!(app.handle(key(KeyCode::Enter)), Action::Continue);
    assert_eq!(app.handle(key(KeyCode::Esc)), Action::Close);
}

#[test]
fn single_tab_search_enter_and_click_focus_workspace() {
    let mut app = app();
    app.set_query("exact terminal".into());
    assert_eq!(app.rows.len(), 1);
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Workspace("w3".into()))
    );
    app.results_area = Rect::new(0, 2, 40, 5);
    assert_eq!(
        app.handle(mouse(MouseEventKind::Down(MouseButton::Left), 2)),
        Action::Switch(Target::Workspace("w3".into()))
    );
}

fn worktree_app() -> App {
    App::new(
        parse_snapshot(include_bytes!("fixtures/worktrees.json")).unwrap(),
        SearchMode::Hierarchical,
    )
}

#[test]
fn focused_worktree_group_starts_with_root_without_marking_root_active() {
    let app = worktree_app();
    assert_eq!(app.rows[0].target, Target::Workspace("w1".into()));
    assert!(!app.rows[0].active);
    let active: Vec<_> = app
        .rows
        .iter()
        .filter(|row| row.active)
        .map(|row| row.target.clone())
        .collect();
    assert_eq!(active, vec![Target::Workspace("w4".into())]);
}

#[test]
fn worktree_enter_focuses_actual_workspace_and_nested_tab_enter_focuses_tab() {
    let mut app = worktree_app();
    app.set_query("multimachine codex".into());
    app.handle(key(KeyCode::Down));
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Workspace("w2".into()))
    );
    app.handle(key(KeyCode::Down));
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Tab("w2:t1".into()))
    );
}

#[test]
fn nested_worktree_and_tab_mouse_clicks_focus_actual_ids() {
    let mut app = worktree_app();
    app.set_query("multimachine codex".into());
    app.results_area = Rect::new(0, 2, 60, 5);
    assert_eq!(
        app.handle(mouse(MouseEventKind::Down(MouseButton::Left), 3)),
        Action::Switch(Target::Workspace("w2".into()))
    );
    assert_eq!(
        app.handle(mouse(MouseEventKind::Down(MouseButton::Left), 4)),
        Action::Switch(Target::Tab("w2:t1".into()))
    );
}

#[test]
fn nested_rows_remain_reachable_after_scrolling_and_query_reset() {
    let mut app = worktree_app();
    assert_eq!(app.selected, Some(0));
    app.navigate(10);
    app.ensure_visible(2);
    assert_eq!(app.offset, 9);
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Tab("w5:t2".into()))
    );
    app.set_query("just terminal".into());
    assert_eq!(app.selected, Some(0));
    assert_eq!(app.offset, 0);
    app.navigate(1);
    assert_eq!(
        app.handle(key(KeyCode::Enter)),
        Action::Switch(Target::Workspace("w4".into()))
    );
}
