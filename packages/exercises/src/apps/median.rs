use ads::RedBlackTree;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};

use crate::render::render_rb_tree_text;
use crate::types::StatusMessage;


pub enum MedianCommand {
    Add(i32),
    Remove(i32),
    Median,
}

impl MedianCommand {
    pub fn parse(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Command cannot be empty.".to_string());
        }
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
    pub tree: RedBlackTree<i32>,
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
                self.tree.insert(x);
                StatusMessage::info(format!("Inserted {x}. Size: {}", self.tree.size()))
            }
            Ok(MedianCommand::Remove(x)) => {
                let existed = self.tree.search(&x).is_some();
                if existed {
                    self.tree.delete_value(&x);
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
                            let median = *handle.value();
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
        if self.output.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Run MEDIAN to print results here.",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))]);
        }
        Text::from(
            self.output
                .iter()
                .map(|line| Line::from(line.clone()))
                .collect::<Vec<_>>(),
        )
    }

    pub fn stats_text(&self) -> Text<'static> {
        let n = self.tree.size();
        let median_str = if n == 0 {
            "—".to_string()
        } else {
            let rank = (n - 1) / 2;
            self.tree
                .select(rank)
                .map(|h| h.value().to_string())
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
