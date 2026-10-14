use herdr_picker::{
    api::parse_snapshot,
    app::App,
    config::SearchMode,
    ui::{label_spans, render},
};
use ratatui::{Terminal, backend::TestBackend};
use unicode_width::UnicodeWidthStr;

fn draw(width: u16, height: u16, query: &str) -> String {
    let mut app = App::new(
        parse_snapshot(include_bytes!("fixtures/snapshot.json")).unwrap(),
        SearchMode::Hierarchical,
    );
    app.set_query(query.into());
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}
#[test]
fn empty_result_text_is_visible() {
    assert!(draw(60, 18, "zzzzzzz").contains("No matching results"));
}
#[test]
fn normal_palette_contains_input_count_results_and_help() {
    let rendered = draw(80, 20, "");
    assert!(rendered.contains("Search workspaces, worktrees and tabs"));
    assert!(rendered.contains("8 results"));
    assert!(rendered.contains("ColorPicker"));
    assert!(rendered.contains("Navigate"));
    assert!(rendered.contains("↵ OK"));
    assert!(rendered.contains("Esc Cancel"));
    assert!(!rendered.contains("Enter Switch"));
    assert!(!rendered.contains("Esc Close"));
}
#[test]
fn tiny_terminal_has_escape_guidance() {
    assert_eq!(draw(6, 1, ""), "Esc cl");
}
#[test]
fn long_wide_labels_are_truncated_by_cell_width() {
    let spans = label_spans("世界Hello", &[0], 5);
    let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
    assert_eq!(text, "世界…");
    assert_eq!(text.width(), 5);
}
#[test]
fn matched_characters_are_underlined() {
    let spans = label_spans("Codex", &[0, 2], 20);
    assert!(
        spans[0]
            .style
            .add_modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
    assert!(
        spans[2]
            .style
            .add_modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
    assert_eq!(spans[1].style, ratatui::style::Style::default());
}
#[test]
fn control_characters_cannot_inject_terminal_sequences() {
    let spans = label_spans("a\n\u{1b}b", &[], 10);
    assert_eq!(
        spans.iter().map(|s| s.content.as_ref()).collect::<String>(),
        "a��b"
    );
}
#[test]
fn zero_width_labels_are_empty() {
    assert!(label_spans("Codex", &[], 0).is_empty());
}

#[test]
fn single_tab_workspace_renders_inline_without_duplicate() {
    let rendered = draw(80, 20, "exact terminal");
    assert!(rendered.contains("Exact Codex - Terminal"));
    assert_eq!(rendered.matches("Terminal").count(), 1);
    assert!(rendered.contains("1 results"));
}

fn inline_buffer(width: u16, mode: SearchMode) -> ratatui::buffer::Buffer {
    let mut snapshot = parse_snapshot(include_bytes!("fixtures/snapshot.json")).unwrap();
    snapshot.workspaces[2].label = "世界Exact".into();
    snapshot.focused_workspace_id = Some("w3".into());
    snapshot.focused_tab_id = Some("w3:t1".into());
    let mut app = App::new(snapshot, mode);
    app.set_query("世界 terminal".into());
    let mut terminal = Terminal::new(TestBackend::new(width, 10)).unwrap();
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn inline_row_highlights_both_names_and_keeps_active_selection() {
    let buffer = inline_buffer(40, SearchMode::Hierarchical);
    assert_eq!(buffer[(0, 2)].symbol(), "›");
    assert_eq!(buffer[(1, 2)].symbol(), "●");
    assert_eq!(buffer[(3, 2)].symbol(), "世");
    assert_eq!(buffer[(15, 2)].symbol(), "T");
    assert!(
        buffer[(3, 2)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
    assert!(
        buffer[(15, 2)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
    assert_eq!(
        buffer[(15, 2)].bg,
        herdr_picker::theme::Palette::catppuccin().selection_bg
    );
    assert_eq!(
        buffer[(15, 2)].fg,
        herdr_picker::theme::Palette::catppuccin().blue
    );
}

#[test]
fn flat_inline_row_uses_dash_separator() {
    let buffer = inline_buffer(40, SearchMode::Flat);
    let text: String = (0..40).map(|x| buffer[(x, 2)].symbol()).collect();
    assert!(text.contains("世 界 Exact - Terminal"));
}

#[test]
fn narrow_inline_row_truncates_unicode_by_cells() {
    let buffer = inline_buffer(14, SearchMode::Hierarchical);
    let text: String = (0..14).map(|x| buffer[(x, 2)].symbol()).collect();
    assert_eq!(text, "›● 世 界 … - Te…");
}

fn worktree_buffer(width: u16, mode: SearchMode, query: &str) -> ratatui::buffer::Buffer {
    let snapshot = parse_snapshot(include_bytes!("fixtures/worktrees.json")).unwrap();
    let mut app = App::new(snapshot, mode);
    app.set_query(query.into());
    let mut terminal = Terminal::new(TestBackend::new(width, 18)).unwrap();
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn root_worktree_and_tab_use_three_distinct_indentation_levels() {
    let buffer = worktree_buffer(
        80,
        SearchMode::Hierarchical,
        "herdr-picker multimachine codex",
    );
    assert_eq!(buffer[(3, 2)].symbol(), "h");
    assert_eq!(buffer[(5, 3)].symbol(), "m");
    assert_eq!(buffer[(7, 4)].symbol(), "C");
    assert!(
        buffer[(3, 2)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
    assert!(
        buffer[(5, 3)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
    assert!(
        buffer[(7, 4)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
}

#[test]
fn flat_nested_tab_displays_full_highlighted_breadcrumb() {
    let buffer = worktree_buffer(80, SearchMode::Flat, "herdr-picker multimachine codex");
    let text: String = (0..80).map(|x| buffer[(x, 2)].symbol()).collect();
    assert!(text.contains("herdr-picker / multimachine / Codex"));
    assert!(
        buffer[(3, 2)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
    assert!(
        buffer[(18, 2)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
    assert!(
        buffer[(33, 2)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
}

#[test]
fn nested_single_tab_worktree_stays_inline_and_active() {
    let buffer = worktree_buffer(80, SearchMode::Hierarchical, "just terminal");
    let text: String = (0..80).map(|x| buffer[(x, 3)].symbol()).collect();
    assert!(text.contains("just - Terminal"));
    assert_eq!(buffer[(1, 2)].symbol(), " ");
    assert_eq!(buffer[(1, 3)].symbol(), "●");
    assert_eq!(buffer[(5, 3)].symbol(), "j");
}

#[test]
fn narrow_nested_tab_keeps_indentation_and_truncates_label() {
    let buffer = worktree_buffer(10, SearchMode::Hierarchical, "multimachine codex");
    let text: String = (0..10).map(|x| buffer[(x, 4)].symbol()).collect();
    assert_eq!(text, "       Co…");
    let flat = worktree_buffer(14, SearchMode::Flat, "herdr-picker multimachine codex");
    let text: String = (0..14).map(|x| flat[(x, 2)].symbol()).collect();
    assert_eq!(text, "›  … / m… / C…");
}

#[test]
fn nested_unicode_breadcrumb_truncates_by_terminal_cells() {
    let mut snapshot = parse_snapshot(include_bytes!("fixtures/worktrees.json")).unwrap();
    snapshot.workspaces[1].label = "世界herdr-picker".into();
    let mut app = App::new(snapshot, SearchMode::Flat);
    app.set_query("世界 multimachine codex".into());
    let mut terminal = Terminal::new(TestBackend::new(18, 10)).unwrap();
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
    let buffer = terminal.backend().buffer();
    let text: String = (0..18).map(|x| buffer[(x, 2)].symbol()).collect();
    assert_eq!(text, "›  世 … / mu… / Co…");
    assert!(
        buffer[(3, 2)]
            .modifier
            .contains(ratatui::style::Modifier::UNDERLINED)
    );
}

fn themed_draw(width: u16, height: u16, query: &str) -> (App, ratatui::buffer::Buffer) {
    let mut app = App::new(
        parse_snapshot(include_bytes!("fixtures/snapshot.json")).unwrap(),
        SearchMode::Hierarchical,
    );
    app.theme = herdr_picker::theme::Theme::parse("[theme]\nname = 'dracula'").unwrap();
    app.set_query(query.into());
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
    (app, terminal.backend().buffer().clone())
}

#[test]
fn dracula_colors_fill_panel_selection_hints_and_buttons() {
    let (app, buffer) = themed_draw(80, 20, "color");
    let palette = app.theme.palette();
    assert_eq!(buffer[(79, 10)].bg, palette.panel_bg);
    assert_eq!(buffer[(0, 0)].fg, palette.text);
    assert_eq!(buffer[(0, 2)].bg, palette.selection_bg);
    assert_eq!(buffer[(0, 2)].fg, palette.text);
    assert_eq!(buffer[(1, 17)].fg, palette.overlay1);
    assert_eq!(buffer[(30, 19)].bg, palette.accent);
    assert_eq!(buffer[(33, 19)].symbol(), "O");
    assert_eq!(buffer[(38, 19)].bg, palette.surface0);
    assert_eq!(buffer[(38, 19)].fg, palette.text);
    let footer: String = (30..50).map(|x| buffer[(x, 19)].symbol()).collect();
    assert_eq!(footer, " ↵ OK    Esc Cancel ");
    let padding: String = (0..80).map(|x| buffer[(x, 18)].symbol()).collect();
    assert_eq!(padding, " ".repeat(80));
    assert_eq!(app.ok_area.x, 30);
    assert_eq!(app.cancel_area.right(), 50);
    assert_eq!(buffer[(29, 19)].bg, palette.panel_bg);
    assert_eq!(buffer[(50, 19)].bg, palette.panel_bg);
}

#[test]
fn empty_results_use_disabled_ok_style() {
    let (app, buffer) = themed_draw(80, 20, "nonexistenttttt");
    let palette = app.theme.palette();
    assert_eq!(buffer[(30, 19)].bg, palette.surface0);
    assert_eq!(buffer[(30, 19)].fg, palette.overlay0);
    assert_eq!(buffer[(38, 19)].fg, palette.text);
}

#[test]
fn compact_footer_shows_cancel_and_tiny_resize_clears_hits() {
    let (mut app, buffer) = themed_draw(14, 10, "");
    assert_eq!(app.ok_area, ratatui::layout::Rect::default());
    assert_eq!(app.cancel_area, ratatui::layout::Rect::new(3, 9, 8, 1));
    let footer: String = (0..14).map(|x| buffer[(x, 9)].symbol()).collect();
    assert_eq!(footer, "    Cancel    ");
    let mut terminal = Terminal::new(TestBackend::new(6, 1)).unwrap();
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
    assert_eq!(app.cancel_area, ratatui::layout::Rect::default());
    assert_eq!(app.results_area, ratatui::layout::Rect::default());
}

#[test]
fn loading_panel_uses_shared_footer_with_disabled_ok() {
    let palette = herdr_picker::theme::Palette::dracula();
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    let mut cancel = ratatui::layout::Rect::default();
    terminal
        .draw(|frame| cancel = herdr_picker::ui::render_loading(frame, palette))
        .unwrap();
    assert_eq!(cancel, ratatui::layout::Rect::new(38, 19, 12, 1));
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(79, 10)].bg, palette.panel_bg);
    assert_eq!(buffer[(30, 19)].fg, palette.overlay0);
    assert_eq!(buffer[(38, 19)].bg, palette.surface0);
    let padding: String = (0..80).map(|x| buffer[(x, 18)].symbol()).collect();
    assert_eq!(padding, " ".repeat(80));
}

fn colored_worktree_app(mode: SearchMode, query: &str) -> App {
    let snapshot = parse_snapshot(include_bytes!("fixtures/worktrees.json")).unwrap();
    let mut app = App::new(snapshot, mode);
    app.theme = herdr_picker::theme::Theme::parse("[theme]\nname = 'dracula'").unwrap();
    app.set_query(query.into());
    app
}

fn app_buffer(app: &mut App, width: u16) -> ratatui::buffer::Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, 18)).unwrap();
    terminal.draw(|frame| render(frame, app)).unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn dracula_hierarchy_colors_selected_root_worktree_and_tab_names() {
    let mut app = colored_worktree_app(SearchMode::Hierarchical, "herdr-picker multimachine codex");
    app.selected = Some(1);
    let buffer = app_buffer(&mut app, 80);
    assert_eq!(buffer[(3, 2)].fg, ratatui::style::Color::Rgb(189, 147, 249));
    assert_eq!(buffer[(5, 3)].fg, ratatui::style::Color::Rgb(80, 250, 123));
    assert_eq!(buffer[(7, 4)].fg, ratatui::style::Color::Rgb(139, 233, 253));
    assert_eq!(buffer[(5, 3)].bg, app.theme.palette().selection_bg);
    assert_eq!(buffer[(3, 2)].bg, app.theme.palette().panel_bg);
    assert_eq!(buffer[(0, 3)].symbol(), "›");
    assert!(
        buffer[(5, 3)]
            .modifier
            .contains(ratatui::style::Modifier::BOLD | ratatui::style::Modifier::UNDERLINED)
    );
}

#[test]
fn dracula_flat_breadcrumb_keeps_each_type_color_on_selected_row() {
    let mut app = colored_worktree_app(SearchMode::Flat, "herdr-picker multimachine codex");
    let buffer = app_buffer(&mut app, 80);
    assert_eq!(buffer[(3, 2)].fg, ratatui::style::Color::Rgb(189, 147, 249));
    assert_eq!(buffer[(18, 2)].fg, ratatui::style::Color::Rgb(80, 250, 123));
    assert_eq!(
        buffer[(33, 2)].fg,
        ratatui::style::Color::Rgb(139, 233, 253)
    );
    assert_eq!(buffer[(16, 2)].symbol(), "/");
    assert_eq!(buffer[(16, 2)].fg, app.theme.palette().text);
    assert_eq!(buffer[(33, 2)].bg, app.theme.palette().selection_bg);
}

#[test]
fn dracula_inline_worktree_and_tab_have_separate_colors() {
    let mut app = colored_worktree_app(SearchMode::Hierarchical, "just terminal");
    let buffer = app_buffer(&mut app, 80);
    assert_eq!(buffer[(5, 3)].symbol(), "j");
    assert_eq!(buffer[(12, 3)].symbol(), "T");
    assert_eq!(buffer[(5, 3)].fg, app.theme.palette().green);
    assert_eq!(buffer[(12, 3)].fg, app.theme.palette().blue);
    assert_eq!(buffer[(10, 3)].fg, app.theme.palette().text);
}

#[test]
fn truncated_breadcrumb_ellipses_keep_each_type_color() {
    let mut app = colored_worktree_app(SearchMode::Flat, "herdr-picker multimachine codex");
    let buffer = app_buffer(&mut app, 14);
    assert_eq!(buffer[(3, 2)].symbol(), "…");
    assert_eq!(buffer[(8, 2)].symbol(), "…");
    assert_eq!(buffer[(13, 2)].symbol(), "…");
    assert_eq!(buffer[(3, 2)].fg, app.theme.palette().accent);
    assert_eq!(buffer[(8, 2)].fg, app.theme.palette().green);
    assert_eq!(buffer[(13, 2)].fg, app.theme.palette().blue);
}

#[test]
fn appearance_updates_apply_custom_type_colors_without_changing_navigation() {
    let mut app = colored_worktree_app(SearchMode::Flat, "herdr-picker multimachine codex");
    app.theme = herdr_picker::theme::Theme::parse("[theme]\nname = 'dracula'\nauto_switch = true\nlight_name = 'one-light'\n[theme.custom]\naccent = '#123456'\ngreen = '#654321'\nblue = '#abcdef'\n[theme.custom.light]\ngreen = '#102030'").unwrap();
    let dark = app_buffer(&mut app, 80);
    assert_eq!(dark[(3, 2)].fg, ratatui::style::Color::Rgb(18, 52, 86));
    assert_eq!(dark[(18, 2)].fg, ratatui::style::Color::Rgb(101, 67, 33));
    assert_eq!(dark[(33, 2)].fg, ratatui::style::Color::Rgb(171, 205, 239));
    let selected = app.selected;
    let offset = app.offset;
    app.handle(terminal_input::event::Event::ThemeModeChanged(
        terminal_input::event::ThemeMode::Light,
    ));
    let light = app_buffer(&mut app, 80);
    assert_eq!(light[(3, 2)].fg, ratatui::style::Color::Rgb(18, 52, 86));
    assert_eq!(light[(18, 2)].fg, ratatui::style::Color::Rgb(16, 32, 48));
    assert_eq!(light[(33, 2)].fg, ratatui::style::Color::Rgb(171, 205, 239));
    assert_eq!(
        light[(33, 2)].bg,
        herdr_picker::theme::Palette::one_light().selection_bg
    );
    assert_eq!(app.query, "herdr-picker multimachine codex");
    assert_eq!(app.selected, selected);
    assert_eq!(app.offset, offset);
}

#[test]
fn dracula_single_tab_root_colors_workspace_and_inline_tab_separately() {
    let (app, buffer) = themed_draw(80, 20, "exact terminal");
    assert_eq!(buffer[(3, 2)].symbol(), "E");
    assert_eq!(buffer[(17, 2)].symbol(), "T");
    assert_eq!(buffer[(3, 2)].fg, app.theme.palette().accent);
    assert_eq!(buffer[(17, 2)].fg, app.theme.palette().blue);
    assert_eq!(buffer[(15, 2)].fg, app.theme.palette().text);
}
