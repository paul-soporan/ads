use ads::{BinarySearchTree, RedBlackTree};
use ratatui::prelude::{Line, Text};

use crate::menu::INTERACTIVE_ACTIONS;
use crate::render::{
    bst_depth, rb_black_height, rb_depth, render_bst_tree_text, render_rb_tree_text,
};
use crate::types::{InputAction, StatusMessage, TreeKind};
use crate::utils::format_option;

pub struct PromptState {
    pub action: InputAction,
    pub title: String,
    pub hint: String,
    pub buffer: String,
}

impl PromptState {
    pub fn new(tree_kind: TreeKind, action: InputAction, title: &str, hint: &str) -> Self {
        Self {
            action,
            title: match tree_kind {
                TreeKind::Bst => format!("BST · {title}"),
                TreeKind::Rb => format!("Red-Black Tree · {title}"),
            },
            hint: hint.to_string(),
            buffer: String::new(),
        }
    }
}

pub enum InteractiveTree {
    Bst(BinarySearchTree<i32>),
    Rb(RedBlackTree<i32>),
}

impl InteractiveTree {
    pub fn new(kind: TreeKind) -> Self {
        match kind {
            TreeKind::Bst => Self::Bst(BinarySearchTree::new()),
            TreeKind::Rb => Self::Rb(RedBlackTree::new()),
        }
    }

    pub fn kind(&self) -> TreeKind {
        match self {
            Self::Bst(_) => TreeKind::Bst,
            Self::Rb(_) => TreeKind::Rb,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Bst(_) => "Binary Search Tree",
            Self::Rb(_) => "Red-Black Tree",
        }
    }

    pub fn screen_title(&self) -> &'static str {
        match self {
            Self::Bst(_) => "BST Interactive",
            Self::Rb(_) => "Red-Black Interactive",
        }
    }

    pub fn tree_text(&self) -> Text<'static> {
        match self {
            Self::Bst(tree) => render_bst_tree_text(tree),
            Self::Rb(tree) => render_rb_tree_text(tree),
        }
    }

    pub fn min_max(&self) -> (Option<i32>, Option<i32>) {
        match self {
            Self::Bst(tree) => (
                tree.min().map(|handle| *handle.value()),
                tree.max().map(|handle| *handle.value()),
            ),
            Self::Rb(tree) => (
                tree.min().map(|handle| *handle.value()),
                tree.max().map(|handle| *handle.value()),
            ),
        }
    }

    pub fn stats_text(&self) -> Text<'static> {
        match self {
            Self::Bst(tree) => {
                let root = tree.root();
                let (min_value, max_value) = self.min_max();
                Text::from(vec![
                    Line::from(format!("Depth: {}", bst_depth(&root))),
                    Line::from(format!(
                        "Root: {}",
                        format_option(root.map(|node| *node.value()))
                    )),
                    Line::from(format!("Min: {}", format_option(min_value))),
                    Line::from(format!("Max: {}", format_option(max_value))),
                ])
            }
            Self::Rb(tree) => {
                let root = tree.root();
                let (min_value, max_value) = self.min_max();
                Text::from(vec![
                    Line::from(format!("Depth: {}", rb_depth(&root))),
                    Line::from(format!("Black height: {}", rb_black_height(&root))),
                    Line::from(format!(
                        "Root: {}",
                        format_option(root.map(|node| *node.value()))
                    )),
                    Line::from(format!("Min: {}", format_option(min_value))),
                    Line::from(format!("Max: {}", format_option(max_value))),
                ])
            }
        }
    }

    pub fn apply_input(&mut self, action: InputAction, value: i32) -> StatusMessage {
        match (self, action) {
            (Self::Bst(tree), InputAction::Insert) => {
                tree.insert(value);
                StatusMessage::success(format!("Inserted {value} into the BST."))
            }
            (Self::Rb(tree), InputAction::Insert) => {
                tree.insert(value);
                StatusMessage::success(format!("Inserted {value} into the red-black tree."))
            }
            (Self::Bst(tree), InputAction::Delete) => match tree.delete_value(&value) {
                Some(deleted) => StatusMessage::success(format!("Deleted {deleted} from the BST.")),
                None => StatusMessage::error(format!("Value {value} was not found in the BST.")),
            },
            (Self::Rb(tree), InputAction::Delete) => match tree.delete_value(&value) {
                Some(deleted) => {
                    StatusMessage::success(format!("Deleted {deleted} from the red-black tree."))
                }
                None => StatusMessage::error(format!(
                    "Value {value} was not found in the red-black tree."
                )),
            },
            (Self::Bst(tree), InputAction::Search) => {
                if tree.contains(&value) {
                    StatusMessage::success(format!("Value {value} exists in the BST."))
                } else {
                    StatusMessage::error(format!("Value {value} does not exist in the BST."))
                }
            }
            (Self::Rb(tree), InputAction::Search) => {
                if tree.contains(&value) {
                    StatusMessage::success(format!("Value {value} exists in the red-black tree."))
                } else {
                    StatusMessage::error(format!(
                        "Value {value} does not exist in the red-black tree."
                    ))
                }
            }
            (Self::Bst(tree), InputAction::PredSucc) => {
                let predecessor = tree
                    .predecessor_of_value(&value)
                    .map(|handle| *handle.value());
                let successor = tree
                    .successor_of_value(&value)
                    .map(|handle| *handle.value());
                StatusMessage::info(format!(
                    "Predecessor: {} • Successor: {}",
                    format_option(predecessor),
                    format_option(successor)
                ))
            }
            (Self::Rb(tree), InputAction::PredSucc) => {
                let predecessor = tree
                    .predecessor_of_value(&value)
                    .map(|handle| *handle.value());
                let successor = tree
                    .successor_of_value(&value)
                    .map(|handle| *handle.value());
                StatusMessage::info(format!(
                    "Predecessor: {} • Successor: {}",
                    format_option(predecessor),
                    format_option(successor)
                ))
            }
        }
    }
}

pub struct InteractiveState {
    pub tree: InteractiveTree,
    pub selected_action: usize,
}

impl InteractiveState {
    pub fn new(kind: TreeKind) -> Self {
        Self {
            tree: InteractiveTree::new(kind),
            selected_action: 0,
        }
    }

    pub fn next_action(&mut self) {
        self.selected_action = (self.selected_action + 1) % INTERACTIVE_ACTIONS.len();
    }

    pub fn previous_action(&mut self) {
        self.selected_action = if self.selected_action == 0 {
            INTERACTIVE_ACTIONS.len() - 1
        } else {
            self.selected_action - 1
        };
    }
}
