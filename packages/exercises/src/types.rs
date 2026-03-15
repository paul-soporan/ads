use ratatui::prelude::{Color, Line, Modifier, Span, Style};

#[derive(Clone, Copy)]
pub enum Op {
    Insert(i32),
    Delete(i32),
}

impl Op {
    pub fn to_line(self) -> Line<'static> {
        match self {
            Self::Insert(value) => Line::from(vec![
                Span::styled("Insert ", Style::default().fg(Color::Green)),
                Span::raw(value.to_string()),
            ]),
            Self::Delete(value) => Line::from(vec![
                Span::styled("Delete ", Style::default().fg(Color::Red)),
                Span::raw(value.to_string()),
            ]),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TreeKind {
    Bst,
    Rb,
}

#[derive(Clone, Copy)]
pub enum InputAction {
    Insert,
    Delete,
    Search,
    PredSucc,
}

pub struct StatusMessage {
    pub text: String,
    pub style: Style,
}

impl StatusMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(Color::Cyan),
        }
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(Color::Green),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(Color::Red),
        }
    }

    pub fn to_line(&self) -> Line<'static> {
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(self.text.clone(), self.style),
        ])
    }
}
