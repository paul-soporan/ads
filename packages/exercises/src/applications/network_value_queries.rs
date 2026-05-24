use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use ads::{DisjointSet, DisjointSetHandle, RedBlackTree};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};

use crate::applications::{CommandApplication, CommandApplicationLayout, parse_command_parts};
use crate::types::StatusMessage;

struct ComponentData {
    tree: RedBlackTree<i64>,
}

impl ComponentData {
    fn new() -> Self {
        Self {
            tree: RedBlackTree::new(),
        }
    }

    fn min_val(&self) -> Option<i64> {
        self.tree.min().map(|v| *v.value())
    }

    fn kth_val(&self, k: usize) -> Option<i64> {
        if k == 0 {
            return None;
        }
        self.tree.select(k - 1).map(|v| *v.value())
    }

    fn size(&self) -> usize {
        self.tree.size()
    }
}

enum NetworkCommand {
    Init(usize, Vec<i64>),
    Union(usize, usize),
    Insert(usize, i64),
    Min(usize),
    Kth(usize, usize),
    GlobalMin,
    Sample,
}

impl NetworkCommand {
    fn parse(input: &str) -> Result<Self, String> {
        let parts = parse_command_parts(input)?;

        match parts[0].to_ascii_uppercase().as_str() {
            "INIT" => {
                if parts.len() < 3 {
                    return Err("INIT format: INIT N v1 v2 ... vN".to_string());
                }
                let n = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "INIT: N must be a positive integer.".to_string())?;
                if parts.len() != n + 2 {
                    return Err(format!(
                        "INIT: expected {n} values, got {}.",
                        parts.len() - 2
                    ));
                }
                let values = parts[2..]
                    .iter()
                    .map(|s| s.parse::<i64>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| "INIT: all values must be integers.".to_string())?;
                Ok(Self::Init(n, values))
            }

            "1" | "UNION" => {
                if parts.len() != 3 {
                    return Err("Format: 1 x y   or   UNION x y".to_string());
                }
                let x = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "x must be a positive integer.".to_string())?;
                let y = parts[2]
                    .parse::<usize>()
                    .map_err(|_| "y must be a positive integer.".to_string())?;
                Ok(Self::Union(x, y))
            }

            "2" | "INSERT" | "ADD" => {
                if parts.len() != 3 {
                    return Err("Format: 2 x v   or   INSERT x v".to_string());
                }
                let x = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "x must be a positive integer.".to_string())?;
                let v = parts[2]
                    .parse::<i64>()
                    .map_err(|_| "v must be an integer.".to_string())?;
                Ok(Self::Insert(x, v))
            }

            "3" | "MIN" => {
                if parts.len() != 2 {
                    return Err("Format: 3 x   or   MIN x".to_string());
                }
                let x = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "x must be a positive integer.".to_string())?;
                Ok(Self::Min(x))
            }

            "4" | "KTH" => {
                if parts.len() != 3 {
                    return Err("Format: 4 x k   or   KTH x k".to_string());
                }
                let x = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "x must be a positive integer.".to_string())?;
                let k = parts[2]
                    .parse::<usize>()
                    .map_err(|_| "k must be a positive integer.".to_string())?;
                Ok(Self::Kth(x, k))
            }

            "5" | "GMIN" | "GLOBALMIN" => Ok(Self::GlobalMin),

            "SAMPLE" => Ok(Self::Sample),

            _ => Err(format!(
                "Unknown command '{}'. Use INIT, 1-5 (UNION/INSERT/MIN/KTH/GMIN), or SAMPLE.",
                parts[0]
            )),
        }
    }
}

/// Each component keeps a Red-Black Tree multiset of its values.
/// The DSU tracks component membership.
/// A lazy-deletion min-heap tracks the global minimum across all components.
pub struct NetworkValueQueryState {
    ds: DisjointSet<usize>,
    handles: Vec<DisjointSetHandle<usize>>,

    /// Keyed by the 0-based index of the component root node.
    component_data: HashMap<usize, ComponentData>,

    /// Min-heap of (min_value, root_index).
    /// Entries are lazily discarded when the root is no longer valid or its minimum has changed.
    global_min_heap: BinaryHeap<Reverse<(i64, usize)>>,

    pub n: usize,
    pub input: String,
    pub output: Vec<String>,
}

impl NetworkValueQueryState {
    pub fn new() -> Self {
        Self {
            ds: DisjointSet::new(),
            handles: Vec::new(),
            component_data: HashMap::new(),
            global_min_heap: BinaryHeap::new(),
            n: 0,
            input: String::new(),
            output: Vec::new(),
        }
    }

    pub fn execute_command(&mut self, raw: &str) -> StatusMessage {
        match NetworkCommand::parse(raw) {
            Ok(cmd) => self.apply(cmd),
            Err(e) => StatusMessage::error(e),
        }
    }

    fn check_node(&self, x: usize) -> Result<(), StatusMessage> {
        if self.n == 0 {
            return Err(StatusMessage::error("Not initialized. Use INIT first."));
        }
        if x < 1 || x > self.n {
            return Err(StatusMessage::error(format!(
                "Node must be in 1..{}.",
                self.n
            )));
        }
        Ok(())
    }

    fn apply(&mut self, cmd: NetworkCommand) -> StatusMessage {
        match cmd {
            NetworkCommand::Init(n, values) => {
                self.ds = DisjointSet::new();
                self.handles.clear();
                self.component_data.clear();
                self.global_min_heap = BinaryHeap::new();
                self.n = n;

                for (i, &val) in values.iter().enumerate() {
                    let handle = self.ds.make_set(i);
                    self.handles.push(handle);
                    let mut data = ComponentData::new();
                    data.tree.insert(val);
                    self.global_min_heap.push(Reverse((val, i)));
                    self.component_data.insert(i, data);
                }

                StatusMessage::success(format!("Initialized {n} nodes."))
            }

            NetworkCommand::Union(x, y) => {
                if let Err(e) = self.check_node(x) {
                    return e;
                }
                if let Err(e) = self.check_node(y) {
                    return e;
                }
                let x0 = x - 1;
                let y0 = y - 1;

                let root_x = *self.ds.find(&self.handles[x0]).value();
                let root_y = *self.ds.find(&self.handles[y0]).value();

                if root_x == root_y {
                    return StatusMessage::info(format!(
                        "Nodes {x} and {y} are already in the same component."
                    ));
                }

                let size_x = self.component_data[&root_x].size();
                let size_y = self.component_data[&root_y].size();

                self.ds.union(&self.handles[x0], &self.handles[y0]);
                let new_root = *self.ds.find(&self.handles[x0]).value();
                let old_root = if new_root == root_x { root_y } else { root_x };

                let new_root_size = if new_root == root_x { size_x } else { size_y };
                let old_root_size = if new_root == root_x { size_y } else { size_x };

                if old_root_size <= new_root_size {
                    let mut old_tree = self.component_data.remove(&old_root).unwrap().tree;
                    let new_data = self.component_data.get_mut(&new_root).unwrap();
                    while let Some(view) = old_tree.min() {
                        let val = old_tree.delete(view).unwrap();
                        new_data.tree.insert(val);
                    }
                } else {
                    let mut smaller_tree = self.component_data.remove(&new_root).unwrap().tree;
                    let mut larger_data = self.component_data.remove(&old_root).unwrap();
                    while let Some(view) = smaller_tree.min() {
                        let val = smaller_tree.delete(view).unwrap();
                        larger_data.tree.insert(val);
                    }
                    self.component_data.insert(new_root, larger_data);
                }

                if let Some(min_val) = self.component_data[&new_root].min_val() {
                    self.global_min_heap.push(Reverse((min_val, new_root)));
                }

                StatusMessage::info(format!("Merged components of nodes {x} and {y}."))
            }

            NetworkCommand::Insert(x, v) => {
                if let Err(e) = self.check_node(x) {
                    return e;
                }
                let root = *self.ds.find(&self.handles[x - 1]).value();
                let data = self.component_data.get_mut(&root).unwrap();
                let old_min = data.min_val();
                data.tree.insert(v);

                let is_new_min = old_min.is_none_or(|m| v <= m);
                if is_new_min {
                    self.global_min_heap.push(Reverse((v, root)));
                }

                StatusMessage::info(format!("Inserted value {v} into component of node {x}."))
            }

            NetworkCommand::Min(x) => {
                if let Err(e) = self.check_node(x) {
                    return e;
                }
                let root = *self.ds.find(&self.handles[x - 1]).value();
                match self.component_data[&root].min_val() {
                    Some(v) => {
                        self.output.push(v.to_string());
                        StatusMessage::info(format!("Min of component({x}): {v}"))
                    }
                    None => StatusMessage::error("Component is empty."),
                }
            }

            NetworkCommand::Kth(x, k) => {
                if let Err(e) = self.check_node(x) {
                    return e;
                }
                let root = *self.ds.find(&self.handles[x - 1]).value();
                match self.component_data[&root].kth_val(k) {
                    Some(v) => {
                        self.output.push(v.to_string());
                        StatusMessage::info(format!("{k}-th smallest of component({x}): {v}"))
                    }
                    None => {
                        self.output.push("-1".to_string());
                        let sz = self.component_data[&root].size();
                        StatusMessage::info(format!(
                            "Component({x}) has {sz} element(s); k={k} is out of range → -1"
                        ))
                    }
                }
            }

            NetworkCommand::GlobalMin => {
                if self.n == 0 {
                    return StatusMessage::error("Not initialized. Use INIT first.");
                }
                match self.query_global_min() {
                    Some(v) => {
                        self.output.push(v.to_string());
                        StatusMessage::info(format!("Global minimum: {v}"))
                    }
                    None => StatusMessage::error("No components."),
                }
            }

            NetworkCommand::Sample => {
                let _ = self.apply(NetworkCommand::Init(5, vec![7, 3, 9, 1, 5]));
                self.output.clear();
                self.output
                    .push("Sample: N=5  A=[7, 3, 9, 1, 5]".to_string());
                self.output
                    .push("Enter operations below. Expected output:".to_string());
                self.output.push("  3 2      → 3".to_string());
                self.output.push("  1 1 2    → (merge)".to_string());
                self.output.push("  3 1      → 3".to_string());
                self.output.push("  4 1 2    → 7".to_string());
                self.output
                    .push("  2 1 4    → (insert 4 into comp of node 1)".to_string());
                self.output.push("  3 2      → 3".to_string());
                self.output.push("  1 4 5    → (merge)".to_string());
                self.output.push("  5        → 1".to_string());
                self.output.push("  1 1 4    → (merge all)".to_string());
                self.output.push("  5        → 1".to_string());
                self.output.push(String::new());
                StatusMessage::success(
                    "Sample loaded. Enter operations one by one to see results.".to_string(),
                )
            }
        }
    }

    /// Returns the global minimum using the lazy-deletion min-heap.
    /// Stale entries (merged roots or outdated minimums) are popped and discarded.
    fn query_global_min(&mut self) -> Option<i64> {
        loop {
            let top = *self.global_min_heap.peek()?;
            let Reverse((min_val, root_idx)) = top;

            let valid = self
                .component_data
                .get(&root_idx)
                .and_then(|d| d.min_val())
                .map(|actual| actual == min_val)
                .unwrap_or(false);

            if valid {
                return Some(min_val);
            }
            self.global_min_heap.pop();
        }
    }

    pub fn output_text(&self) -> Text<'static> {
        if self.output.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Use INIT N v1…vN or SAMPLE, then enter operations 1-5.",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))]);
        }

        let mut lines = Vec::with_capacity(self.output.len());
        for line in &self.output {
            let trimmed = line.trim_start();
            if trimmed.starts_with("Sample:") || trimmed.starts_with("Enter") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if trimmed.starts_with("→") || trimmed.contains("→") {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::DarkGray),
                )));
            } else if !trimmed.is_empty()
                && trimmed
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if trimmed == "-1" {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::LightRed),
                )));
            } else {
                lines.push(Line::from(line.clone()));
            }
        }
        Text::from(lines)
    }

    pub fn stats_text(&self) -> Text<'static> {
        let num_components = self.component_data.len();
        let global_min = self
            .component_data
            .values()
            .filter_map(|d| d.min_val())
            .min();
        let total_values: usize = self.component_data.values().map(|d| d.size()).sum();

        Text::from(vec![
            Line::from(format!("Nodes      : {}", self.n)),
            Line::from(format!("Components : {num_components}")),
            Line::from(format!(
                "Global Min : {}",
                global_min
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "—".to_string())
            )),
            Line::from(format!("Total Vals : {total_values}")),
        ])
    }

    pub fn state_text(&self) -> Text<'static> {
        if self.n == 0 {
            return Text::from(vec![Line::from(Span::styled(
                "No nodes yet. Use INIT N v1 ... vN or SAMPLE.",
                Style::default().fg(Color::DarkGray),
            ))]);
        }

        let mut comp_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..self.n {
            let root = *self.ds.find(&self.handles[i]).value();
            comp_map.entry(root).or_default().push(i + 1);
        }

        let mut roots: Vec<usize> = comp_map.keys().copied().collect();
        roots.sort();

        let palette = [
            Color::Yellow,
            Color::LightBlue,
            Color::LightGreen,
            Color::LightMagenta,
        ];

        let mut lines: Vec<Line> = Vec::new();
        for (idx, root) in roots.iter().enumerate() {
            let members = &comp_map[root];
            let data = &self.component_data[root];
            let size = data.size();
            let min_str = data
                .min_val()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "—".to_string());

            let show = size.min(8);
            let vals: Vec<String> = (1..=show)
                .filter_map(|k| data.kth_val(k))
                .map(|v| v.to_string())
                .collect();
            let vals_str = if size > 8 {
                format!("[{}, …]", vals.join(", "))
            } else {
                format!("[{}]", vals.join(", "))
            };

            let nodes_str = format!(
                "{{{}}}",
                members
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            lines.push(Line::from(Span::styled(
                format!("  nodes:{nodes_str}  min:{min_str}  vals:{vals_str}"),
                Style::default().fg(palette[idx % palette.len()]),
            )));
        }

        Text::from(lines)
    }
}

impl CommandApplication for NetworkValueQueryState {
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
            "INIT N v… | 1 x y | 2 x v | 3 x | 4 x k | 5 | SAMPLE",
            Style::default().fg(Color::DarkGray),
        ))]
    }

    fn layout(&self) -> CommandApplicationLayout {
        CommandApplicationLayout::new(55, 5, 6, "Output", "Command Input", "Stats", "Components")
    }
}

impl Default for NetworkValueQueryState {
    fn default() -> Self {
        Self::new()
    }
}
