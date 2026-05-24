use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};

pub fn digit_to_index(ch: char, max: usize) -> Option<usize> {
    ch.to_digit(10)
        .and_then(|value| usize::try_from(value).ok())
        .and_then(|value| value.checked_sub(1))
        .filter(|index| *index < max)
}

pub fn format_option(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "None".to_string())
}

/// Convert a list of plain strings into a `Text` widget value.
pub fn lines_to_text(lines: &[String]) -> Text<'static> {
    Text::from(
        lines
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect::<Vec<_>>(),
    )
}

/// Return a dimmed italic placeholder text when there is no output yet.
pub fn placeholder_text(msg: &'static str) -> Text<'static> {
    Text::from(vec![Line::from(Span::styled(
        msg,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    ))])
}

/// Build a set-membership display like `{1, 2, 3}`.
pub fn format_members(members: &[usize]) -> String {
    format!(
        "{{{}}}",
        members
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
