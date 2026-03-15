use ads::RedBlackTree;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use crate::render::render_rb_tree_text_generic;
use crate::types::StatusMessage;


#[derive(Clone, Eq, PartialEq)]
pub struct LeaderboardEntry {
    pub player: String,
    pub score: i32,
}

impl Display for LeaderboardEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.player, self.score)
    }
}

impl Ord for LeaderboardEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.player.cmp(&other.player))
    }
}

impl PartialOrd for LeaderboardEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


pub enum LeaderboardCommand {
    Add { player: String, score: i32 },
    Update { player: String, delta: i32 },
    Remove { player: String },
    Top { k: usize },
}

impl LeaderboardCommand {
    pub fn parse(input: &str) -> Result<Self, String> {
        let parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            return Err("Command cannot be empty.".to_string());
        }

        match parts[0].to_ascii_uppercase().as_str() {
            "ADD" => {
                if parts.len() != 3 {
                    return Err("ADD format: ADD player score".to_string());
                }

                let score = parts[2]
                    .parse::<i32>()
                    .map_err(|_| "ADD expects score to be an integer.".to_string())?;
                Ok(Self::Add {
                    player: parts[1].to_string(),
                    score,
                })
            }
            "UPDATE" => {
                if parts.len() != 3 {
                    return Err("UPDATE format: UPDATE player delta".to_string());
                }

                let delta = parts[2]
                    .parse::<i32>()
                    .map_err(|_| "UPDATE expects delta to be an integer.".to_string())?;
                Ok(Self::Update {
                    player: parts[1].to_string(),
                    delta,
                })
            }
            "REMOVE" => {
                if parts.len() != 2 {
                    return Err("REMOVE format: REMOVE player".to_string());
                }
                Ok(Self::Remove {
                    player: parts[1].to_string(),
                })
            }
            "TOP" => {
                if parts.len() != 2 {
                    return Err("TOP format: TOP k".to_string());
                }

                let k = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "TOP expects k to be a non-negative integer.".to_string())?;
                Ok(Self::Top { k })
            }
            _ => Err("Unknown command. Use ADD, UPDATE, REMOVE, or TOP.".to_string()),
        }
    }
}


pub struct LeaderboardState {
    pub tree: RedBlackTree<LeaderboardEntry>,
    pub scores: HashMap<String, i32>,
    pub output: Vec<String>,
    pub input: String,
}

impl LeaderboardState {
    pub fn new() -> Self {
        Self {
            tree: RedBlackTree::new(),
            scores: HashMap::new(),
            output: Vec::new(),
            input: String::new(),
        }
    }

    pub fn execute_command(&mut self, raw_command: &str) -> StatusMessage {
        match LeaderboardCommand::parse(raw_command) {
            Ok(command) => self.apply(command),
            Err(message) => StatusMessage::error(message),
        }
    }

    fn apply(&mut self, command: LeaderboardCommand) -> StatusMessage {
        match command {
            LeaderboardCommand::Add { player, score } => {
                if self.scores.contains_key(&player) {
                    return StatusMessage::error(format!(
                        "ADD failed: player '{player}' already exists."
                    ));
                }

                let entry = LeaderboardEntry {
                    player: player.clone(),
                    score,
                };
                self.tree.insert(entry);
                self.scores.insert(player.clone(), score);
                StatusMessage::success(format!("Added {player} with score {score}."))
            }
            LeaderboardCommand::Update { player, delta } => {
                let Some(current_score) = self.scores.get(&player).copied() else {
                    return StatusMessage::error(format!(
                        "UPDATE failed: player '{player}' does not exist."
                    ));
                };

                let old_entry = LeaderboardEntry {
                    player: player.clone(),
                    score: current_score,
                };
                let _ = self.tree.delete_value(&old_entry);

                let new_score = current_score + delta;
                let new_entry = LeaderboardEntry {
                    player: player.clone(),
                    score: new_score,
                };
                self.tree.insert(new_entry);
                self.scores.insert(player.clone(), new_score);

                StatusMessage::success(format!("Updated {player}: {current_score} -> {new_score}."))
            }
            LeaderboardCommand::Remove { player } => {
                let Some(current_score) = self.scores.remove(&player) else {
                    return StatusMessage::error(format!(
                        "REMOVE failed: player '{player}' does not exist."
                    ));
                };

                let old_entry = LeaderboardEntry {
                    player: player.clone(),
                    score: current_score,
                };
                let _ = self.tree.delete_value(&old_entry);

                StatusMessage::success(format!("Removed {player} from the leaderboard."))
            }
            LeaderboardCommand::Top { k } => {
                let results = self.top_k(k);
                if results.is_empty() {
                    self.output.push("(no players)".to_string());
                } else {
                    self.output.extend(results);
                }
                self.output.push(String::new());

                StatusMessage::info(format!("TOP {k} executed."))
            }
        }
    }

    fn top_k(&self, k: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = self.tree.max();

        while let Some(node) = current {
            if result.len() >= k {
                break;
            }

            let entry = node.value().clone();
            result.push(format!("{} {}", entry.player, entry.score));
            current = self.tree.predecessor(&node);
        }

        result
    }

    pub fn output_text(&self) -> Text<'static> {
        if self.output.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Run TOP k to print results here.",
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

    pub fn tree_text(&self) -> Text<'static> {
        render_rb_tree_text_generic(&self.tree)
    }
}
