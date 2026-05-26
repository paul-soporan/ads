use ads::trees::red_black_tree::safe::RedBlackTree;
use ratatui::prelude::{Color, Line, Span, Style, Text};

use crate::applications::{
    CommandApplication, CommandApplicationLayout, output_lines_to_text, parse_command_parts,
};
use crate::render::render_rb_tree_text;
use crate::types::StatusMessage;

pub enum MedianCommand {
    Add(i32),
    Remove(i32),
    Median,
}

impl MedianCommand {
    pub fn parse(input: &str) -> Result<Self, String> {
        let parts = parse_command_parts(input)?;
        match parts[0].to_ascii_uppercase().as_str() {
            "ADD" => {
                if parts.len() != 2 {
                    return Err("ADD format: ADD <integer>".to_string());
                }
                let x = parts[1]
                    .parse::<i32>()
                    .map_err(|_| "ADD expects an integer argument.".to_string())?;
                Ok(Self::Add(x))
            }
            "REMOVE" => {
                if parts.len() != 2 {
                    return Err("REMOVE format: REMOVE <integer>".to_string());
                }
                let x = parts[1]
                    .parse::<i32>()
                    .map_err(|_| "REMOVE expects an integer argument.".to_string())?;
                Ok(Self::Remove(x))
            }
            "MEDIAN" => Ok(Self::Median),
            _ => Err("Unknown command. Use ADD, REMOVE, or MEDIAN.".to_string()),
        }
    }
}

pub struct MedianStreamState {
    pub tree: RedBlackTree<i32, ()>,
    pub output: Vec<String>,
    pub input: String,
}

impl MedianStreamState {
    pub fn new() -> Self {
        Self {
            tree: RedBlackTree::new(),
            output: Vec::new(),
            input: String::new(),
        }
    }

    pub fn execute_command(&mut self, raw: &str) -> StatusMessage {
        match MedianCommand::parse(raw) {
            Err(e) => StatusMessage::error(e),
            Ok(MedianCommand::Add(x)) => {
                self.tree.insert_entry(x, ());
                StatusMessage::info(format!("Inserted {x}. Size: {}", self.tree.size()))
            }
            Ok(MedianCommand::Remove(x)) => {
                let existed = self.tree.cursor(&x).is_some();
                if existed {
                    self.tree.remove_key(&x);
                    StatusMessage::info(format!("Removed {x}. Size: {}", self.tree.size()))
                } else {
                    StatusMessage::error(format!("{x} not found in stream."))
                }
            }
            Ok(MedianCommand::Median) => {
                let n = self.tree.size();
                if n == 0 {
                    StatusMessage::error("Stream is empty — no median.")
                } else {
                    let rank = (n - 1) / 2;
                    match self.tree.select(rank) {
                        Some(handle) => {
                            let median = *handle.key();
                            self.output.push(median.to_string());
                            StatusMessage::info(format!("Median: {median}"))
                        }
                        None => StatusMessage::error("Could not locate median node."),
                    }
                }
            }
        }
    }

    pub fn info_text(&self) -> Text<'static> {
        output_lines_to_text(&self.output, "Run MEDIAN to print results here.")
    }

    pub fn stats_text(&self) -> Text<'static> {
        let n = self.tree.size();
        let median_str = if n == 0 {
            "—".to_string()
        } else {
            let rank = (n - 1) / 2;
            self.tree
                .select(rank)
                .map(|h| h.key().to_string())
                .unwrap_or_else(|| "?".to_string())
        };
        Text::from(vec![
            Line::from(format!("Elements : {n}")),
            Line::from(format!("Median   : {median_str}")),
        ])
    }

    pub fn tree_text(&self) -> Text<'static> {
        render_rb_tree_text(&self.tree)
    }
}

impl CommandApplication for MedianStreamState {
    fn execute_command(&mut self, raw: &str) -> StatusMessage {
        Self::execute_command(self, raw)
    }

    fn input_buffer(&self) -> &str {
        self.input.as_str()
    }

    fn input_buffer_mut(&mut self) -> &mut String {
        &mut self.input
    }

    fn output_text(&self) -> Text<'static> {
        Self::info_text(self)
    }

    fn stats_text(&self) -> Text<'static> {
        Self::stats_text(self)
    }

    fn state_text(&self) -> Text<'static> {
        Self::tree_text(self)
    }

    fn input_hint_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(Span::styled(
                "Format: ADD x | REMOVE x | MEDIAN",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Lower median is returned for an even count of elements.",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    }

    fn layout(&self) -> CommandApplicationLayout {
        CommandApplicationLayout::new(
            62,
            6,
            6,
            "Output",
            "Command Input",
            "Stats",
            "Order-Statistic Tree",
        )
    }
}
