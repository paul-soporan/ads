use ratatui::prelude::{Line, Text};

use crate::types::StatusMessage;
use crate::utils::{lines_to_text, placeholder_text};

pub struct CommandApplicationLayout {
    pub left_width_percent: u16,
    pub input_height: u16,
    pub stats_height: u16,
    pub output_title: &'static str,
    pub input_title: &'static str,
    pub stats_title: &'static str,
    pub state_title: &'static str,
}

impl CommandApplicationLayout {
    pub const fn new(
        left_width_percent: u16,
        input_height: u16,
        stats_height: u16,
        output_title: &'static str,
        input_title: &'static str,
        stats_title: &'static str,
        state_title: &'static str,
    ) -> Self {
        Self {
            left_width_percent,
            input_height,
            stats_height,
            output_title,
            input_title,
            stats_title,
            state_title,
        }
    }
}

pub trait CommandApplication {
    fn execute_command(&mut self, raw: &str) -> StatusMessage;
    fn input_buffer(&self) -> &str;
    fn input_buffer_mut(&mut self) -> &mut String;
    fn output_text(&self) -> Text<'static>;
    fn stats_text(&self) -> Text<'static>;
    fn state_text(&self) -> Text<'static>;
    fn input_hint_lines(&self) -> Vec<Line<'static>>;
    fn layout(&self) -> CommandApplicationLayout;

    fn submit_input(&mut self) -> StatusMessage {
        let command = self.input_buffer().trim().to_string();
        if command.is_empty() {
            return StatusMessage::error("Please enter a command.");
        }

        let status = self.execute_command(command.as_str());
        self.input_buffer_mut().clear();
        status
    }
}

pub fn parse_command_parts(input: &str) -> Result<Vec<&str>, String> {
    let parts = input.split_whitespace().collect::<Vec<_>>();
    if parts.is_empty() {
        return Err("Command cannot be empty.".to_string());
    }

    Ok(parts)
}

/// Convert a list of logged output strings into a `Text` widget value, or show
/// `placeholder` (in dimmed italic) when the list is empty.
pub fn output_lines_to_text(lines: &[String], placeholder: &'static str) -> Text<'static> {
    if lines.is_empty() {
        placeholder_text(placeholder)
    } else {
        lines_to_text(lines)
    }
}
