use ads::{
    BTree, BTreeNodeView, BinarySearchTree, BstNodeView, NodeColor, RbNodeView, RedBlackTree,
};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use std::{collections::BTreeMap, fmt::Display};

pub struct NodeLabel {
    pub text: String,
    pub style: Style,
}

pub trait RenderNode {
    fn left_child(&self) -> Option<Self>
    where
        Self: Sized;
    fn right_child(&self) -> Option<Self>
    where
        Self: Sized;
    fn label(&self) -> NodeLabel;
}

impl RenderNode for BstNodeView<i32> {
    fn left_child(&self) -> Option<Self> {
        self.left()
    }

    fn right_child(&self) -> Option<Self> {
        self.right()
    }

    fn label(&self) -> NodeLabel {
        NodeLabel {
            text: self.value().to_string(),
            style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl<T> RenderNode for RbNodeView<T>
where
    T: Display,
{
    fn left_child(&self) -> Option<Self> {
        self.left()
    }

    fn right_child(&self) -> Option<Self> {
        self.right()
    }

    fn label(&self) -> NodeLabel {
        let text = match self.color() {
            NodeColor::Red => format!("{}(R)", self.value()),
            NodeColor::Black => format!("{}(B)", self.value()),
        };

        let style = match self.color() {
            NodeColor::Red => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            NodeColor::Black => Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        };

        NodeLabel { text, style }
    }
}

#[derive(Clone)]
struct StyledCell {
    text: String,
    style: Style,
}

pub struct TreeCanvas {
    grid: BTreeMap<(usize, usize), StyledCell>,
}

impl TreeCanvas {
    pub fn new() -> Self {
        Self {
            grid: BTreeMap::new(),
        }
    }

    pub fn put(&mut self, row: usize, col: usize, text: impl Into<String>, style: Style) {
        self.grid.insert(
            (row, col),
            StyledCell {
                text: text.into(),
                style,
            },
        );
    }

    pub fn into_text(self) -> Text<'static> {
        if self.grid.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "(empty tree)",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))]);
        }

        let max_row = self.grid.keys().map(|(row, _)| *row).max().unwrap_or(0);
        let mut lines = Vec::with_capacity(max_row + 1);

        for row in 0..=max_row {
            let row_items = self
                .grid
                .iter()
                .filter(|((current_row, _), _)| *current_row == row)
                .collect::<Vec<_>>();

            let mut spans = Vec::new();
            let mut current_col = 0;

            for ((_, col), cell) in row_items {
                if *col > current_col {
                    spans.push(Span::raw(" ".repeat(*col - current_col)));
                }

                current_col = *col + cell.text.chars().count();
                spans.push(Span::styled(cell.text.clone(), cell.style));
            }

            lines.push(Line::from(spans));
        }

        Text::from(lines)
    }
}

pub fn build_layout<N: RenderNode>(
    node: &Option<N>,
    level: usize,
    cursor_x: &mut usize,
    canvas: &mut TreeCanvas,
) -> Option<(usize, usize)> {
    let node = node.as_ref()?;

    let left_position = build_layout(&node.left_child(), level + 1, cursor_x, canvas);
    let label = node.label();
    let visible_width = label.text.chars().count();
    let padding = 3;

    let center_x = *cursor_x + visible_width / 2;
    let row = level * 2;
    let start_col = center_x.saturating_sub(visible_width / 2);
    canvas.put(row, start_col, label.text, label.style);
    *cursor_x = start_col + visible_width + padding;

    let right_position = build_layout(&node.right_child(), level + 1, cursor_x, canvas);

    let edge_style = Style::default().fg(Color::DarkGray);
    if let Some((left_x, _)) = left_position {
        canvas.put(row + 1, (center_x + left_x) / 2, "╱", edge_style);
    }
    if let Some((right_x, _)) = right_position {
        canvas.put(row + 1, (center_x + right_x) / 2, "╲", edge_style);
    }

    Some((center_x, row))
}

fn render_tree_text<N: RenderNode>(root: &Option<N>) -> Text<'static> {
    let mut canvas = TreeCanvas::new();
    let mut cursor_x = 0;
    let _ = build_layout(root, 0, &mut cursor_x, &mut canvas);
    canvas.into_text()
}

pub fn render_bst_tree_text(tree: &BinarySearchTree<i32>) -> Text<'static> {
    render_tree_text(&tree.root())
}

pub fn render_rb_tree_text(tree: &RedBlackTree<i32>) -> Text<'static> {
    render_rb_tree_text_generic(tree)
}

pub fn render_rb_tree_text_generic<T>(tree: &RedBlackTree<T>) -> Text<'static>
where
    T: Ord + Display,
{
    render_tree_text(&tree.root())
}

pub fn bst_depth<T>(node: &Option<BstNodeView<T>>) -> usize {
    match node {
        Some(node) => 1 + usize::max(bst_depth(&node.left()), bst_depth(&node.right())),
        None => 0,
    }
}

pub fn rb_depth<T>(node: &Option<RbNodeView<T>>) -> usize {
    match node {
        Some(node) => 1 + usize::max(rb_depth(&node.left()), rb_depth(&node.right())),
        None => 0,
    }
}

pub fn rb_black_height<T>(node: &Option<RbNodeView<T>>) -> usize {
    match node {
        Some(node) => {
            let left_height = rb_black_height(&node.left());
            left_height + usize::from(node.color() == NodeColor::Black)
        }
        None => 1,
    }
}

fn build_btree_layout<T>(
    view: &BTreeNodeView<T>,
    level: usize,
    cursor_x: &mut usize,
    canvas: &mut TreeCanvas,
) -> usize
where
    T: Clone + std::fmt::Display,
{
    let padding = 2;
    let keys: Vec<String> = view.keys().iter().map(|k| k.to_string()).collect();
    let label = format!("[{}]", keys.join("|"));
    let label_width = label.chars().count();
    let children = view.children();
    let row = level * 2;

    if children.is_empty() {
        let center_x = *cursor_x + label_width / 2;
        let style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        canvas.put(row, *cursor_x, label, style);
        *cursor_x += label_width + padding;
        return center_x;
    }

    let mut child_centers: Vec<usize> = Vec::with_capacity(children.len());
    for child in &children {
        let cc = build_btree_layout(child, level + 1, cursor_x, canvas);
        child_centers.push(cc);
    }

    let first_cc = child_centers[0];
    let last_cc = *child_centers.last().unwrap();
    let center_x = (first_cc + last_cc) / 2;
    let start_col = center_x.saturating_sub(label_width / 2);

    let style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    canvas.put(row, start_col, label, style);

    let edge_style = Style::default().fg(Color::DarkGray);
    for &cc in &child_centers {
        canvas.put(row + 1, cc, "│", edge_style);
    }

    center_x
}

pub fn render_btree_text(tree: &BTree<i32>) -> Text<'static> {
    render_btree_text_generic(tree)
}

pub fn render_btree_text_generic<T>(tree: &BTree<T>) -> Text<'static>
where
    T: Ord + Clone + std::fmt::Display,
{
    match tree.root_view() {
        None => Text::from(vec![Line::from(Span::styled(
            "(empty tree)",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ))]),
        Some(root) => {
            let mut canvas = TreeCanvas::new();
            let mut cursor_x = 0;
            build_btree_layout(&root, 0, &mut cursor_x, &mut canvas);
            canvas.into_text()
        }
    }
}

pub fn btree_depth<T: Clone>(view: &Option<BTreeNodeView<T>>) -> usize {
    match view {
        None => 0,
        Some(v) => {
            if v.is_leaf() {
                1
            } else {
                1 + v
                    .children()
                    .iter()
                    .map(|c| btree_depth(&Some(c.clone())))
                    .max()
                    .unwrap_or(0)
            }
        }
    }
}

pub fn btree_key_count<T: Clone>(view: &Option<BTreeNodeView<T>>) -> usize {
    match view {
        None => 0,
        Some(v) => {
            v.keys().len()
                + v.children()
                    .iter()
                    .map(|c| btree_key_count(&Some(c.clone())))
                    .sum::<usize>()
        }
    }
}
