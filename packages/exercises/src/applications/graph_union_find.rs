use ads::DisjointSet;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};

use crate::applications::{CommandApplication, CommandApplicationLayout, parse_command_parts};
use crate::render::render_disjoint_set_forest_text;
use crate::types::StatusMessage;
use crate::utils::format_members;

enum GraphCommand {
    SetNodes(usize),
    AddEdge(usize, usize),
    Analyze,
    Sample,
}

impl GraphCommand {
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
                Ok(Self::SetNodes(n))
            }
            "ADD" => {
                if parts.len() != 3 {
                    return Err("ADD format: ADD <u> <v>".to_string());
                }
                let u = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "ADD expects u to be a non-negative integer.".to_string())?;
                let v = parts[2]
                    .parse::<usize>()
                    .map_err(|_| "ADD expects v to be a non-negative integer.".to_string())?;
                Ok(Self::AddEdge(u, v))
            }
            "ANALYZE" => Ok(Self::Analyze),
            "SAMPLE" => Ok(Self::Sample),
            _ => Err("Unknown command. Use SETN, ADD, ANALYZE, or SAMPLE.".to_string()),
        }
    }
}

#[derive(Clone)]
struct GraphAnalysis {
    component_count: usize,
    components: Vec<(usize, Vec<usize>)>,
    parent_by_node: Vec<Option<usize>>,
    cycle_edges: Vec<(usize, usize)>,
}

pub struct GraphUnionFindState {
    pub n: usize,
    pub edges: Vec<(usize, usize)>,
    pub output: Vec<String>,
    pub input: String,
    last: Option<GraphAnalysis>,
}

impl GraphUnionFindState {
    pub fn new() -> Self {
        Self {
            n: 0,
            edges: Vec::new(),
            output: Vec::new(),
            input: String::new(),
            last: None,
        }
    }

    pub fn execute_command(&mut self, raw: &str) -> StatusMessage {
        match GraphCommand::parse(raw) {
            Ok(command) => self.apply(command),
            Err(message) => StatusMessage::error(message),
        }
    }

    fn apply(&mut self, command: GraphCommand) -> StatusMessage {
        match command {
            GraphCommand::SetNodes(n) => {
                self.n = n;
                self.edges.clear();
                self.last = None;
                StatusMessage::success(format!(
                    "Graph reset with {n} nodes (0..{}).",
                    n.saturating_sub(1)
                ))
            }
            GraphCommand::AddEdge(u, v) => {
                if self.n == 0 {
                    return StatusMessage::error(
                        "Set node count first using SETN <n>.".to_string(),
                    );
                }
                if u >= self.n || v >= self.n {
                    return StatusMessage::error(format!(
                        "Edge out of range: nodes must be in 0..{}.",
                        self.n - 1
                    ));
                }
                self.edges.push((u, v));
                self.last = None;
                StatusMessage::info(format!("Added edge ({u}, {v})."))
            }
            GraphCommand::Analyze => self.analyze(),
            GraphCommand::Sample => {
                self.n = 8;
                self.edges = vec![(0, 1), (1, 2), (2, 0), (3, 4), (5, 6)];
                self.last = None;
                self.push_sample_to_output();
                StatusMessage::success(
                    "Loaded sample graph: 8 nodes, 5 edges, one cycle in {0,1,2}.".to_string(),
                )
            }
        }
    }

    fn analyze(&mut self) -> StatusMessage {
        if self.n == 0 {
            return StatusMessage::error(
                "Cannot analyze an empty graph. Use SETN first.".to_string(),
            );
        }

        let analysis = self.preview_analysis();
        let Some(analysis) = analysis else {
            return StatusMessage::error(
                "Cannot analyze an empty graph. Use SETN first.".to_string(),
            );
        };

        self.push_analysis_to_output(&analysis);
        self.last = Some(analysis.clone());

        StatusMessage::success(format!(
            "Analyze complete: {} components, cycle {}.",
            analysis.component_count,
            if analysis.cycle_edges.is_empty() {
                "not detected"
            } else {
                "detected"
            }
        ))
    }

    fn preview_analysis(&self) -> Option<GraphAnalysis> {
        if self.n == 0 {
            return None;
        }

        let mut uf = DisjointSet::new();
        let handles = (0..self.n).map(|i| uf.make_set(i)).collect::<Vec<_>>();
        let mut cycle_edges = Vec::new();

        for &(u, v) in &self.edges {
            if uf.same_set(&handles[u], &handles[v]) {
                cycle_edges.push((u, v));
            } else {
                uf.union(&handles[u], &handles[v]);
            }
        }

        let parent_by_node: Vec<Option<usize>> = handles
            .iter()
            .map(|h| h.parent().map(|p| *p.value()))
            .collect();
        let components: Vec<(usize, Vec<usize>)> = uf
            .components()
            .into_iter()
            .map(|(root, members)| (*root.value(), members.iter().map(|m| *m.value()).collect()))
            .collect();

        Some(GraphAnalysis {
            component_count: components.len(),
            components,
            parent_by_node,
            cycle_edges,
        })
    }

    fn push_analysis_to_output(&mut self, analysis: &GraphAnalysis) {
        self.output.push("Graph Analysis".to_string());
        self.output
            .push(format!("  Nodes: {} | Edges: {}", self.n, self.edges.len()));
        self.output.push(format!(
            "  Connected Components: {}",
            analysis.component_count
        ));

        if analysis.cycle_edges.is_empty() {
            self.output
                .push("  Cycle Detection: No cycle found".to_string());
        } else {
            self.output.push(format!(
                "  Cycle Detection: Yes ({} cycle-causing edge{})",
                analysis.cycle_edges.len(),
                if analysis.cycle_edges.len() == 1 {
                    ""
                } else {
                    "s"
                }
            ));
        }

        self.output.push("  Components:".to_string());

        for (idx, (_, component)) in analysis.components.iter().enumerate() {
            self.output.push(format!(
                "    C{} • {} members • {}",
                idx + 1,
                component.len(),
                format_members(component)
            ));
        }

        if !analysis.cycle_edges.is_empty() {
            self.output.push("  Cycle-causing edges:".to_string());
            for (idx, (u, v)) in analysis.cycle_edges.iter().enumerate() {
                self.output.push(format!("    {}. ({u}, {v})", idx + 1));
            }
        }

        self.output.push(String::new());
    }

    fn push_sample_to_output(&mut self) {
        self.output.push("Sample Scenario Loaded".to_string());
        self.output.push("  Graph: 8 nodes (0..7)".to_string());
        self.output
            .push("  Edges: (0,1), (1,2), (2,0), (3,4), (5,6)".to_string());
        self.output
            .push("  Tip: run ANALYZE to print component and cycle details.".to_string());
        self.output.push(String::new());
    }

    pub fn output_text(&self) -> Text<'static> {
        if self.output.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Use SETN/ADD or SAMPLE, then run ANALYZE.",
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
            } else if trimmed.contains("Cycle Detection:") {
                let style = if trimmed.contains("No cycle") {
                    Style::default().fg(Color::LightGreen)
                } else {
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD)
                };
                lines.push(Line::from(Span::styled(line.clone(), style)));
            } else if trimmed == "Components:" {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if trimmed == "Cycle-causing edges:" {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
            } else if trimmed.starts_with('C')
                && trimmed[1..]
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                && line.contains("members")
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
            } else if trimmed.starts_with(|c: char| c.is_ascii_digit()) && line.contains('(') {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::LightRed),
                )));
            } else if trimmed.starts_with("Connected Components:") || trimmed.starts_with("Nodes:")
            {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::White),
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
        let edge_count = self.edges.len();
        let live = self.preview_analysis();
        let component_count = live
            .as_ref()
            .map(|result| result.component_count.to_string())
            .unwrap_or_else(|| "0".to_string());
        let has_cycle = live
            .as_ref()
            .map(|result| {
                if result.cycle_edges.is_empty() {
                    "No"
                } else {
                    "Yes"
                }
            })
            .unwrap_or("No");

        Text::from(vec![
            Line::from(format!("Nodes       : {}", self.n)),
            Line::from(format!("Edges       : {edge_count}")),
            Line::from(format!("Components  : {component_count}")),
            Line::from(format!("Cycle       : {has_cycle}")),
        ])
    }

    pub fn state_text(&self) -> Text<'static> {
        if self.n == 0 {
            return Text::from(vec![Line::from(Span::styled(
                "Set nodes first with SETN <n>.",
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

        render_disjoint_set_forest_text(&analysis.components, &analysis.parent_by_node, "C")
    }
}

impl CommandApplication for GraphUnionFindState {
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
            "SETN n | ADD u v | ANALYZE | SAMPLE",
            Style::default().fg(Color::DarkGray),
        ))]
    }

    fn layout(&self) -> CommandApplicationLayout {
        CommandApplicationLayout::new(62, 5, 6, "Output", "Command Input", "Stats", "State")
    }
}
