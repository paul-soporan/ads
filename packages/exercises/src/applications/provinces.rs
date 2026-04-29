use ads::DisjointSet;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};

use crate::render::render_disjoint_set_forest_text;
use crate::types::StatusMessage;

enum ProvinceCommand {
    SetN(usize),
    Connect(usize, usize),
    Analyze,
    Sample,
}

impl ProvinceCommand {
    fn parse(input: &str) -> Result<Self, String> {
        let parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            return Err("Command cannot be empty.".to_string());
        }

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
            "CONNECT" => {
                if parts.len() != 3 {
                    return Err("CONNECT format: CONNECT <i> <j>".to_string());
                }
                let i = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "CONNECT expects i to be a non-negative integer.".to_string())?;
                let j = parts[2]
                    .parse::<usize>()
                    .map_err(|_| "CONNECT expects j to be a non-negative integer.".to_string())?;
                Ok(Self::Connect(i, j))
            }
            "ANALYZE" => Ok(Self::Analyze),
            "SAMPLE" => Ok(Self::Sample),
            _ => Err("Unknown command. Use SETN, CONNECT, ANALYZE, or SAMPLE.".to_string()),
        }
    }
}

#[derive(Clone)]
struct ProvinceAnalysis {
    province_count: usize,
    provinces: Vec<(usize, Vec<usize>)>,
    parent_by_node: Vec<Option<usize>>,
}

fn format_members(members: &[usize]) -> String {
    format!(
        "{{{}}}",
        members
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub struct ProvinceCounterState {
    pub matrix: Vec<Vec<u8>>,
    pub output: Vec<String>,
    pub input: String,
    last: Option<ProvinceAnalysis>,
}

impl ProvinceCounterState {
    pub fn new() -> Self {
        Self {
            matrix: Vec::new(),
            output: Vec::new(),
            input: String::new(),
            last: None,
        }
    }

    pub fn execute_command(&mut self, raw: &str) -> StatusMessage {
        match ProvinceCommand::parse(raw) {
            Ok(command) => self.apply(command),
            Err(message) => StatusMessage::error(message),
        }
    }

    fn apply(&mut self, command: ProvinceCommand) -> StatusMessage {
        match command {
            ProvinceCommand::SetN(n) => {
                self.matrix = vec![vec![0; n]; n];
                for i in 0..n {
                    self.matrix[i][i] = 1;
                }
                self.last = None;
                StatusMessage::success(format!("Initialized {n}x{n} matrix."))
            }
            ProvinceCommand::Connect(i, j) => {
                let n = self.matrix.len();
                if n == 0 {
                    return StatusMessage::error("Use SETN first.".to_string());
                }
                if i >= n || j >= n {
                    return StatusMessage::error(format!("Indices must be in 0..{}.", n - 1));
                }
                self.matrix[i][j] = 1;
                self.matrix[j][i] = 1;
                self.last = None;
                StatusMessage::info(format!("Connected city {i} with city {j}."))
            }
            ProvinceCommand::Analyze => self.analyze(),
            ProvinceCommand::Sample => {
                self.matrix = vec![
                    vec![1, 1, 0, 0, 0],
                    vec![1, 1, 0, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 0, 0, 1],
                ];
                self.last = None;
                self.push_sample_to_output();
                StatusMessage::success("Loaded sample matrix with 3 provinces.".to_string())
            }
        }
    }

    fn analyze(&mut self) -> StatusMessage {
        if self.matrix.is_empty() {
            return StatusMessage::error("Matrix is empty. Use SETN first.".to_string());
        }

        let analysis = self.preview_analysis();
        let Some(analysis) = analysis else {
            return StatusMessage::error("Matrix is empty. Use SETN first.".to_string());
        };

        self.push_analysis_to_output(&analysis);
        self.last = Some(analysis.clone());
        StatusMessage::success(format!(
            "Analyze complete: {} provinces.",
            analysis.province_count
        ))
    }

    fn preview_analysis(&self) -> Option<ProvinceAnalysis> {
        let n = self.matrix.len();
        if n == 0 {
            return None;
        }

        let mut uf = DisjointSet::new();
        let handles = (0..n).map(|i| uf.make_set(i)).collect::<Vec<_>>();

        for i in 0..n {
            for j in (i + 1)..n {
                if self.matrix[i][j] == 1 {
                    uf.union(&handles[i], &handles[j]);
                }
            }
        }

        let parent_by_node: Vec<Option<usize>> = handles
            .iter()
            .map(|h| h.parent().map(|p| *p.value()))
            .collect();
        let provinces: Vec<(usize, Vec<usize>)> = uf
            .components()
            .into_iter()
            .map(|(root, members)| (*root.value(), members.iter().map(|m| *m.value()).collect()))
            .collect();

        Some(ProvinceAnalysis {
            province_count: provinces.len(),
            provinces,
            parent_by_node,
        })
    }

    fn push_analysis_to_output(&mut self, analysis: &ProvinceAnalysis) {
        let n = self.matrix.len();
        let ones = self
            .matrix
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&v| v == 1)
            .count();
        let roads = if n == 0 {
            0
        } else {
            (ones.saturating_sub(n)) / 2
        };

        self.output.push("Province Analysis".to_string());
        self.output.push(format!("  Cities: {n} | Roads: {roads}"));
        self.output
            .push(format!("  Total Provinces: {}", analysis.province_count));
        self.output.push("  Provinces:".to_string());

        for (idx, (_, province)) in analysis.provinces.iter().enumerate() {
            self.output.push(format!(
                "    P{} • {} cities • {}",
                idx + 1,
                province.len(),
                format_members(province)
            ));
        }

        self.output.push(String::new());
    }

    fn push_sample_to_output(&mut self) {
        self.output.push("Sample Scenario Loaded".to_string());
        self.output.push("  Matrix size: 5x5".to_string());
        self.output
            .push("  Connected pairs: (0,1), (2,3); city 4 isolated".to_string());
        self.output.push("  Expected provinces: 3".to_string());
        self.output
            .push("  Tip: run ANALYZE to print province groups.".to_string());
        self.output.push(String::new());
    }

    pub fn output_text(&self) -> Text<'static> {
        if self.output.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Use SETN/CONNECT or SAMPLE, then run ANALYZE.",
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
            } else if trimmed == "Provinces:" {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if trimmed.starts_with('P')
                && trimmed[1..]
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                && line.contains("cities")
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
            } else if trimmed.starts_with("Total Provinces:") || trimmed.starts_with("Cities:") {
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
        let n = self.matrix.len();
        let ones = self
            .matrix
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&v| v == 1)
            .count();
        let undirected_edges = if n == 0 {
            0
        } else {
            (ones.saturating_sub(n)) / 2
        };
        let live = self.preview_analysis();
        let province_count = live
            .as_ref()
            .map(|x| x.province_count.to_string())
            .unwrap_or_else(|| "0".to_string());

        Text::from(vec![
            Line::from(format!("Cities      : {n}")),
            Line::from(format!("Roads       : {undirected_edges}")),
            Line::from(format!("Provinces   : {province_count}")),
        ])
    }

    pub fn state_text(&self) -> Text<'static> {
        if self.matrix.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Matrix is empty. Use SETN <n> or SAMPLE.",
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

        let mut lines = vec![Line::from(Span::styled(
            "isConnected Matrix",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];

        let n = self.matrix.len();
        let header = (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(" ");
        lines.push(Line::from(format!("    {header}")));
        for (i, row) in self.matrix.iter().enumerate() {
            let row_text = row
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            lines.push(Line::from(format!("{i:>2}: {row_text}")));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Disjoint-Set Forest",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));

        let forest =
            render_disjoint_set_forest_text(&analysis.provinces, &analysis.parent_by_node, "P");
        lines.extend(forest.lines);

        Text::from(lines)
    }
}
