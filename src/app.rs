use ratatui::layout::Rect;
use terminal_input::event::{
    Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::{
    config::SearchMode,
    model::{Snapshot, Target},
    ordering,
    search::{Row, Search},
    theme::Theme,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Close,
    Switch(Target),
}

pub struct App {
    pub snapshot: Snapshot,
    pub query: String,
    pub rows: Vec<Row>,
    pub selected: Option<usize>,
    pub offset: usize,
    pub results_area: Rect,
    pub mode: SearchMode,
    pub ok_area: Rect,
    pub cancel_area: Rect,
    pub theme: Theme,
    search: Search,
}

impl App {
    pub fn new(snapshot: Snapshot, mode: SearchMode) -> Self {
        let mut search = Search::default();
        let rows = search.rows(&snapshot, "", mode);
        let initial = ordering::initial_target(&snapshot);
        let selected = rows.iter().position(
            |r| matches!(&r.target, Target::Workspace(id) if Some(id.as_str()) == initial),
        );
        Self {
            snapshot,
            query: String::new(),
            rows,
            selected,
            offset: 0,
            results_area: Rect::default(),
            ok_area: Rect::default(),
            cancel_area: Rect::default(),
            theme: Theme::default(),
            mode,
            search,
        }
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.rows = self.search.rows(&self.snapshot, &self.query, self.mode);
        self.selected = (!self.rows.is_empty()).then_some(0);
        self.offset = 0;
    }

    pub fn navigate(&mut self, delta: isize) {
        if self.rows.is_empty() {
            self.selected = None;
            return;
        }
        let current = self.selected.unwrap_or(0);
        self.selected = Some(
            current
                .saturating_add_signed(delta)
                .min(self.rows.len() - 1),
        );
    }

    pub fn ensure_visible(&mut self, height: usize) {
        if let Some(selected) = self.selected {
            if selected < self.offset {
                self.offset = selected;
            }
            if height > 0 && selected >= self.offset + height {
                self.offset = selected + 1 - height;
            }
        }
        self.offset = self.offset.min(self.rows.len().saturating_sub(height));
    }

    fn activate(&self) -> Action {
        self.selected
            .and_then(|i| self.rows.get(i))
            .map(|r| Action::Switch(r.target.clone()))
            .unwrap_or(Action::Continue)
    }

    pub fn handle(&mut self, event: Event) -> Action {
        match event {
            Event::ThemeModeChanged(appearance) => self.theme.set_appearance(appearance),
            Event::Mouse(mouse)
                if mouse.kind == MouseEventKind::Down(MouseButton::Left)
                    && self.cancel_area.contains((mouse.column, mouse.row).into()) =>
            {
                return Action::Close;
            }
            Event::Mouse(mouse)
                if mouse.kind == MouseEventKind::Down(MouseButton::Left)
                    && self.ok_area.contains((mouse.column, mouse.row).into()) =>
            {
                return self.activate();
            }
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Esc => return Action::Close,
                    KeyCode::Char('c') if ctrl => return Action::Close,
                    KeyCode::Up | KeyCode::Char('p') if key.code == KeyCode::Up || ctrl => {
                        self.navigate(-1)
                    }
                    KeyCode::Down | KeyCode::Char('n') if key.code == KeyCode::Down || ctrl => {
                        self.navigate(1)
                    }
                    KeyCode::Enter => return self.activate(),
                    KeyCode::Backspace => {
                        let mut query = self.query.clone();
                        query.pop();
                        self.set_query(query);
                    }
                    KeyCode::Char(ch)
                        if !key.modifiers.intersects(
                            KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                        ) =>
                    {
                        self.set_query(format!("{}{ch}", self.query))
                    }
                    _ => {}
                }
            }
            Event::Paste(text) => self.set_query(format!(
                "{}{}",
                self.query,
                text.chars().filter(|c| !c.is_control()).collect::<String>()
            )),
            Event::Mouse(mouse) if self.results_area.contains((mouse.column, mouse.row).into()) => {
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        let index = self.offset + usize::from(mouse.row - self.results_area.y);
                        if index < self.rows.len() {
                            self.selected = Some(index);
                            return self.activate();
                        }
                    }
                    MouseEventKind::ScrollDown => self.navigate(3),
                    MouseEventKind::ScrollUp => self.navigate(-3),
                    _ => {}
                }
            }
            _ => {}
        }
        Action::Continue
    }
}
