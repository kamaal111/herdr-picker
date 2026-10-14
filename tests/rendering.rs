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
    assert!(rendered.contains("Search workspaces and tabs"));
    assert!(rendered.contains("9 results"));
    assert!(rendered.contains("ColorPicker"));
    assert!(rendered.contains("Esc Close"));
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
