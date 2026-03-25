use ads::{BTree, BinarySearchTree, RedBlackTree};
use ratatui::prelude::{Line, Text};
use ratatui::widgets::ListItem;

use crate::render::{
    bst_depth, btree_depth, btree_key_count, rb_black_height, rb_depth, render_bst_tree_text,
    render_btree_text, render_rb_tree_text,
};
use crate::types::{Op, TreeKind};
use crate::utils::format_option;

pub const BST_SHOWCASE_OPS: [Op; 16] = [
    Op::Insert(50),
    Op::Insert(25),
    Op::Insert(75),
    Op::Insert(10),
    Op::Insert(30),
    Op::Insert(60),
    Op::Insert(90),
    Op::Insert(5),
    Op::Insert(15),
    Op::Insert(27),
    Op::Insert(35),
    Op::Insert(80),
    Op::Insert(100),
    Op::Delete(25),
    Op::Delete(50),
    Op::Delete(90),
];

pub const BTREE_SHOWCASE_OPS: [Op; 18] = [
    Op::Insert(10),
    Op::Insert(20),
    Op::Insert(30),
    Op::Insert(40),
    Op::Insert(50),
    Op::Insert(60),
    Op::Insert(70),
    Op::Insert(80),
    Op::Insert(15),
    Op::Insert(25),
    Op::Insert(45),
    Op::Insert(55),
    Op::Delete(40),
    Op::Delete(20),
    Op::Delete(10),
    Op::Delete(60),
    Op::Delete(55),
    Op::Delete(25),
];

pub const RB_SHOWCASE_OPS: [Op; 12] = [
    Op::Insert(10),
    Op::Insert(20),
    Op::Insert(30),
    Op::Insert(40),
    Op::Insert(50),
    Op::Insert(25),
    Op::Insert(15),
    Op::Insert(5),
    Op::Insert(3),
    Op::Delete(20),
    Op::Delete(30),
    Op::Delete(50),
];

pub struct ShowcaseState {
    pub kind: TreeKind,
    pub step: usize,
}

impl ShowcaseState {
    pub fn new(kind: TreeKind) -> Self {
        Self { kind, step: 0 }
    }

    pub fn screen_title(&self) -> &'static str {
        match self.kind {
            TreeKind::Bst => "BST Showcase",
            TreeKind::Rb => "Red-Black Tree Showcase",
            TreeKind::BTree => "B-Tree Showcase",
        }
    }

    pub fn tree_title(&self) -> &'static str {
        match self.kind {
            TreeKind::Bst => "Binary Search Tree",
            TreeKind::Rb => "Red-Black Tree",
            TreeKind::BTree => "B-Tree  (t = 2, internal=yellow, leaf=cyan)",
        }
    }

    pub fn operations(&self) -> &'static [Op] {
        match self.kind {
            TreeKind::Bst => &BST_SHOWCASE_OPS,
            TreeKind::Rb => &RB_SHOWCASE_OPS,
            TreeKind::BTree => &BTREE_SHOWCASE_OPS,
        }
    }

    pub fn next_step(&mut self) {
        self.step = (self.step + 1).min(self.operations().len());
    }

    pub fn previous_step(&mut self) {
        self.step = self.step.saturating_sub(1);
    }

    pub fn current_action(&self) -> Option<Op> {
        self.step
            .checked_sub(1)
            .and_then(|index| self.operations().get(index).copied())
    }

    pub fn stats_text(&self) -> Text<'static> {
        let mut lines = vec![Line::from(format!(
            "Step: {}/{}",
            self.step,
            self.operations().len()
        ))];

        match self.kind {
            TreeKind::Bst => {
                let tree = build_bst_showcase_tree(self.step);
                let root = tree.root();
                lines.push(Line::from(format!("Depth: {}", bst_depth(&root))));
                lines.push(Line::from(format!("Nodes: {}", self.step)));
                lines.push(Line::from(format!(
                    "Min: {}",
                    format_option(tree.min().map(|node| *node.value()))
                )));
                lines.push(Line::from(format!(
                    "Max: {}",
                    format_option(tree.max().map(|node| *node.value()))
                )));
            }
            TreeKind::Rb => {
                let tree = build_rb_showcase_tree(self.step);
                let root = tree.root();
                lines.push(Line::from(format!("Depth: {}", rb_depth(&root))));
                lines.push(Line::from(format!(
                    "Black height: {}",
                    rb_black_height(&root)
                )));
                lines.push(Line::from(format!(
                    "Min: {}",
                    format_option(tree.min().map(|node| *node.value()))
                )));
                lines.push(Line::from(format!(
                    "Max: {}",
                    format_option(tree.max().map(|node| *node.value()))
                )));
            }
            TreeKind::BTree => {
                let tree = build_btree_showcase_tree(self.step);
                let root_view = tree.root_view();
                lines.push(Line::from(format!("Degree (t): {}", tree.degree())));
                lines.push(Line::from(format!("Height: {}", btree_depth(&root_view))));
                lines.push(Line::from(format!(
                    "Total keys: {}",
                    btree_key_count(&root_view)
                )));
                lines.push(Line::from(format!(
                    "Min: {}",
                    format_option(tree.min().map(|handle| *handle.value()))
                )));
                lines.push(Line::from(format!(
                    "Max: {}",
                    format_option(tree.max().map(|handle| *handle.value()))
                )));
            }
        }

        Text::from(lines)
    }

    pub fn current_action_text(&self) -> Text<'static> {
        match self.current_action() {
            Some(op) => Text::from(vec![Line::from("Last operation:"), op.to_line()]),
            None => Text::from(vec![
                Line::from("Last operation:"),
                Line::from("Initial empty state"),
            ]),
        }
    }

    pub fn history_items(&self) -> Vec<ListItem<'static>> {
        self.operations()
            .iter()
            .take(self.step)
            .enumerate()
            .map(|(index, op)| {
                let label = match op {
                    Op::Insert(value) => format!("{:>2}. Insert {value}", index + 1),
                    Op::Delete(value) => format!("{:>2}. Delete {value}", index + 1),
                };
                ListItem::new(label)
            })
            .collect()
    }

    pub fn tree_text(&self) -> Text<'static> {
        match self.kind {
            TreeKind::Bst => render_bst_tree_text(&build_bst_showcase_tree(self.step)),
            TreeKind::Rb => render_rb_tree_text(&build_rb_showcase_tree(self.step)),
            TreeKind::BTree => render_btree_text(&build_btree_showcase_tree(self.step)),
        }
    }
}

pub fn build_bst_showcase_tree(step: usize) -> BinarySearchTree<i32> {
    let mut tree = BinarySearchTree::new();
    for operation in BST_SHOWCASE_OPS.iter().take(step) {
        match operation {
            Op::Insert(value) => tree.insert(*value),
            Op::Delete(value) => {
                let _ = tree.delete_value(value);
            }
        }
    }
    tree
}

pub fn build_rb_showcase_tree(step: usize) -> RedBlackTree<i32> {
    let mut tree = RedBlackTree::new();
    for operation in RB_SHOWCASE_OPS.iter().take(step) {
        match operation {
            Op::Insert(value) => tree.insert(*value),
            Op::Delete(value) => {
                let _ = tree.delete_value(value);
            }
        }
    }
    tree
}

pub fn build_btree_showcase_tree(step: usize) -> BTree<i32> {
    let mut tree = BTree::new(2);
    for operation in BTREE_SHOWCASE_OPS.iter().take(step) {
        match operation {
            Op::Insert(value) => tree.insert(*value),
            Op::Delete(value) => {
                let _ = tree.delete_value(value);
            }
        }
    }
    tree
}
