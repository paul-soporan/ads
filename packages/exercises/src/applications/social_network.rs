use ads::contiguous::disjoint_set::safe::DisjointSet;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};

use crate::applications::{CommandApplication, CommandApplicationLayout, parse_command_parts};
use crate::render::render_disjoint_set_forest_text;
use crate::types::StatusMessage;
use crate::utils::format_members;

enum SocialCommand {
    SetN(usize),
    Friend(usize, usize),
    Analyze,
    Sample,
}

impl SocialCommand {
    fn parse(input: &str) -> Result<Self, String> {
        let parts = parse_command_parts(input)?;

        match parts[0].to_ascii_uppercase().as_str() {
            "SETN" => {
                if parts.len() != 2 {
                    return Err("SETN format: SETN <n>".to_string());
                }
                let n = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "SETN expects a non-negative integer.".to_string())?;
                Ok(Self::SetN(n))
            }
            "FRIEND" => {
                if parts.len() != 3 {
                    return Err("FRIEND format: FRIEND <u> <v>".to_string());
                }
                let u = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "FRIEND expects u to be a non-negative integer.".to_string())?;
                let v = parts[2]
                    .parse::<usize>()
                    .map_err(|_| "FRIEND expects v to be a non-negative integer.".to_string())?;
                Ok(Self::Friend(u, v))
            }
            "ANALYZE" => Ok(Self::Analyze),
            "SAMPLE" => Ok(Self::Sample),
            _ => Err("Unknown command. Use SETN, FRIEND, ANALYZE, or SAMPLE.".to_string()),
        }
    }
}

#[derive(Clone)]
struct SocialAnalysis {
    groups: Vec<(usize, Vec<usize>)>,
    parent_by_node: Vec<Option<usize>>,
}

pub struct SocialNetworkState {
    pub n: usize,
    pub friendships: Vec<(usize, usize)>,
    pub output: Vec<String>,
    pub input: String,
    last: Option<SocialAnalysis>,
}

impl SocialNetworkState {
    pub fn new() -> Self {
        Self {
            n: 0,
            friendships: Vec::new(),
            output: Vec::new(),
            input: String::new(),
            last: None,
        }
    }

    pub fn execute_command(&mut self, raw: &str) -> StatusMessage {
        match SocialCommand::parse(raw) {
            Ok(command) => self.apply(command),
            Err(message) => StatusMessage::error(message),
        }
    }

    fn apply(&mut self, command: SocialCommand) -> StatusMessage {
        match command {
            SocialCommand::SetN(n) => {
                self.n = n;
                self.friendships.clear();
                self.last = None;
                StatusMessage::success(format!(
                    "Initialized {n} users (0..{}).",
                    n.saturating_sub(1)
                ))
            }
            SocialCommand::Friend(u, v) => {
                if self.n == 0 {
                    return StatusMessage::error("Use SETN first.".to_string());
                }
                if u >= self.n || v >= self.n {
                    return StatusMessage::error(format!("Users must be in 0..{}.", self.n - 1));
                }
                self.friendships.push((u, v));
                self.last = None;
                StatusMessage::info(format!("Recorded friendship ({u}, {v})."))
            }
            SocialCommand::Analyze => self.analyze(),
            SocialCommand::Sample => {
                self.n = 7;
                self.friendships = vec![(0, 1), (1, 2), (3, 4), (5, 6)];
                self.last = None;
                self.push_sample_to_output();
                StatusMessage::success(
                    "Loaded sample: n=7, friendships=[(0,1),(1,2),(3,4),(5,6)].".to_string(),
                )
            }
        }
    }

    fn analyze(&mut self) -> StatusMessage {
        if self.n == 0 {
            return StatusMessage::error("No users available. Use SETN first.".to_string());
        }

        let analysis = self.preview_analysis();
        let Some(analysis) = analysis else {
            return StatusMessage::error("No users available. Use SETN first.".to_string());
        };

        self.push_analysis_to_output(&analysis);
        self.last = Some(analysis.clone());

        StatusMessage::success(format!(
            "Analyze complete: {} friend groups.",
            analysis.groups.len()
        ))
    }

    fn preview_analysis(&self) -> Option<SocialAnalysis> {
        if self.n == 0 {
            return None;
        }

        let mut uf = DisjointSet::new();
        for i in 0..self.n {
            uf.make_set(i);
        }

        for &(u, v) in &self.friendships {
            uf.union(&u, &v);
        }

        let parent_by_node: Vec<Option<usize>> = (0..self.n)
            .map(|value| {
                uf.view(&value)
                    .and_then(|view| view.parent_id().and_then(|pid| uf.root_value(pid)))
            })
            .collect();
        let groups: Vec<(usize, Vec<usize>)> = uf
            .components()
            .into_iter()
            .map(|(root, members)| {
                let root_value = uf.root_value(root).expect("root id should be valid");
                (root_value, members)
            })
            .collect();

        Some(SocialAnalysis {
            groups,
            parent_by_node,
        })
    }

    fn push_analysis_to_output(&mut self, analysis: &SocialAnalysis) {
        self.output.push("Friend Network Analysis".to_string());
        self.output.push(format!(
            "  Users: {} | Friendship events: {}",
            self.n,
            self.friendships.len()
        ));
        self.output
            .push(format!("  Total friend groups: {}", analysis.groups.len()));
        self.output.push("  Groups:".to_string());

        for (idx, (_, group)) in analysis.groups.iter().enumerate() {
            self.output.push(format!(
                "    G{} • {} users • {}",
                idx + 1,
                group.len(),
                format_members(group)
            ));
        }

        self.output.push(String::new());
    }

    fn push_sample_to_output(&mut self) {
        self.output.push("Sample Scenario Loaded".to_string());
        self.output.push("  Users: 7 (0..6)".to_string());
        self.output
            .push("  Friendships: (0,1), (1,2), (3,4), (5,6)".to_string());
        self.output
            .push("  Expected groups: {0,1,2}, {3,4}, {5,6}".to_string());
        self.output
            .push("  Tip: run ANALYZE to print groups and members.".to_string());
        self.output.push(String::new());
    }

    pub fn output_text(&self) -> Text<'static> {
        if self.output.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Use SETN/FRIEND or SAMPLE, then run ANALYZE.",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))]);
        }

        let palette = [
            Color::Yellow,
            Color::LightBlue,
            Color::LightGreen,
            Color::LightMagenta,
        ];

        let mut lines = Vec::with_capacity(self.output.len());
        for line in &self.output {
            let trimmed = line.trim_start();

            if line.ends_with("Analysis") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if line.ends_with("Loaded") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if trimmed == "Groups:" {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if trimmed.starts_with('G')
                && trimmed[1..]
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                && line.contains("users")
            {
                let idx = trimmed[1..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<usize>()
                    .unwrap_or(1)
                    .saturating_sub(1);
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(palette[idx % palette.len()])
                        .add_modifier(Modifier::BOLD),
                )));
            } else if trimmed.starts_with("Total friend groups:") || trimmed.starts_with("Users:") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::White),
                )));
            } else if trimmed.starts_with("Expected") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::LightCyan),
                )));
            } else if trimmed.starts_with("Tip:") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )));
            } else if line.starts_with("  ") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::Gray),
                )));
            } else {
                lines.push(Line::from(line.clone()));
            }
        }

        Text::from(lines)
    }

    pub fn stats_text(&self) -> Text<'static> {
        let live = self.preview_analysis();
        let groups = live
            .as_ref()
            .map(|x| x.groups.len().to_string())
            .unwrap_or_else(|| "0".to_string());

        let sizes = live
            .as_ref()
            .map(|x| {
                let mut sizes = x
                    .groups
                    .iter()
                    .map(|(_, members)| members.len())
                    .collect::<Vec<_>>();
                sizes.sort_by(|a, b| b.cmp(a));
                format!(
                    "[{}]",
                    sizes
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .unwrap_or_else(|| "[]".to_string());

        Text::from(vec![
            Line::from(format!("Users       : {}", self.n)),
            Line::from(format!("Events      : {}", self.friendships.len())),
            Line::from(format!("Groups      : {groups}")),
            Line::from(format!("Sizes       : {sizes}")),
        ])
    }

    pub fn state_text(&self) -> Text<'static> {
        if self.n == 0 {
            return Text::from(vec![Line::from(Span::styled(
                "No users yet. Use SETN <n> or SAMPLE.",
                Style::default().fg(Color::DarkGray),
            ))]);
        }

        let analysis = self.last.clone().or_else(|| self.preview_analysis());
        let Some(analysis) = analysis else {
            return Text::from(vec![Line::from(Span::styled(
                "No disjoint-set state available.",
                Style::default().fg(Color::DarkGray),
            ))]);
        };

        render_disjoint_set_forest_text(&analysis.groups, &analysis.parent_by_node, "G")
    }
}

impl CommandApplication for SocialNetworkState {
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
        Self::output_text(self)
    }

    fn stats_text(&self) -> Text<'static> {
        Self::stats_text(self)
    }

    fn state_text(&self) -> Text<'static> {
        Self::state_text(self)
    }

    fn input_hint_lines(&self) -> Vec<Line<'static>> {
        vec![Line::from(Span::styled(
            "SETN n | FRIEND u v | ANALYZE | SAMPLE",
            Style::default().fg(Color::DarkGray),
        ))]
    }

    fn layout(&self) -> CommandApplicationLayout {
        CommandApplicationLayout::new(62, 5, 6, "Output", "Command Input", "Stats", "State")
    }
}
