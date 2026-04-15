use ads::{BTree, BinarySearchTree, BinomialHeap, RedBlackTree};
use ratatui::prelude::{Line, Text};

use crate::menu::{HEAP_INTERACTIVE_ACTIONS, INTERACTIVE_ACTIONS};
use crate::render::{
    binomial_heap_degrees_str, binomial_heap_total, bst_depth, btree_depth, btree_key_count,
    rb_black_height, rb_depth, render_binomial_heap_text, render_bst_tree_text, render_btree_text,
    render_rb_tree_text,
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
                TreeKind::BTree => format!("B-Tree · {title}"),
                TreeKind::BinomialHeap => format!("Binomial Heap · {title}"),
            },
            hint: hint.to_string(),
            buffer: String::new(),
        }
    }
}

pub enum InteractiveTree {
    Bst(BinarySearchTree<i32>),
    Rb(RedBlackTree<i32>),
    BTree(BTree<i32>),
    BinomialHeap(BinomialHeap<i32>),
}

impl InteractiveTree {
    pub fn new(kind: TreeKind) -> Self {
        match kind {
            TreeKind::Bst => Self::Bst(BinarySearchTree::new()),
            TreeKind::Rb => Self::Rb(RedBlackTree::new()),
            TreeKind::BTree => Self::BTree(BTree::new(2)),
            TreeKind::BinomialHeap => Self::BinomialHeap(BinomialHeap::new()),
        }
    }

    pub fn kind(&self) -> TreeKind {
        match self {
            Self::Bst(_) => TreeKind::Bst,
            Self::Rb(_) => TreeKind::Rb,
            Self::BTree(_) => TreeKind::BTree,
            Self::BinomialHeap(_) => TreeKind::BinomialHeap,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Bst(_) => "Binary Search Tree",
            Self::Rb(_) => "Red-Black Tree",
            Self::BTree(_) => "B-Tree (t = 2, internal=yellow, leaf=cyan)",
            Self::BinomialHeap(_) => "Binomial Heap  (root=magenta, internal=green, leaf=cyan)",
        }
    }

    pub fn screen_title(&self) -> &'static str {
        match self {
            Self::Bst(_) => "BST Interactive",
            Self::Rb(_) => "Red-Black Interactive",
            Self::BTree(_) => "B-Tree Interactive",
            Self::BinomialHeap(_) => "Binomial Heap Interactive",
        }
    }

    pub fn tree_text(&self) -> Text<'static> {
        match self {
            Self::Bst(tree) => render_bst_tree_text(tree),
            Self::Rb(tree) => render_rb_tree_text(tree),
            Self::BTree(tree) => render_btree_text(tree),
            Self::BinomialHeap(heap) => render_binomial_heap_text(heap),
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
            Self::BTree(tree) => (
                tree.min().map(|handle| *handle.value()),
                tree.max().map(|handle| *handle.value()),
            ),
            Self::BinomialHeap(heap) => (heap.min().map(|v| *v.value()), None),
        }
    }

    /// Returns the list of action labels shown in the sidebar for this tree kind.
    pub fn action_list(&self) -> &'static [&'static str] {
        match self {
            Self::BinomialHeap(_) => &HEAP_INTERACTIVE_ACTIONS,
            _ => &INTERACTIVE_ACTIONS,
        }
    }

    /// Number of actions available for this tree kind.
    pub fn action_count(&self) -> usize {
        self.action_list().len()
    }

    /// Context-sensitive one-line help text for the footer bar.
    pub fn help_text(&self) -> &'static str {
        match self {
            Self::BinomialHeap(_) => "↑/↓ move • Enter run • 1-5 shortcuts • Esc back • q quit",
            _ => "↑/↓ move • Enter run • 1-6 shortcuts • Esc back • q quit",
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
            Self::BTree(tree) => {
                let root_view = tree.root_view();
                let (min_value, max_value) = self.min_max();
                Text::from(vec![
                    Line::from(format!("Degree (t): {}", tree.degree())),
                    Line::from(format!("Height: {}", btree_depth(&root_view))),
                    Line::from(format!("Total keys: {}", btree_key_count(&root_view))),
                    Line::from(format!("Min: {}", format_option(min_value))),
                    Line::from(format!("Max: {}", format_option(max_value))),
                ])
            }
            Self::BinomialHeap(heap) => Text::from(vec![
                Line::from(format!("Total elements: {}", binomial_heap_total(heap))),
                Line::from(format!("Trees: {}", binomial_heap_degrees_str(heap))),
                Line::from(format!(
                    "Min: {}",
                    format_option(heap.min().map(|v| *v.value()))
                )),
            ]),
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
            (Self::BTree(tree), InputAction::Insert) => {
                tree.insert(value);
                StatusMessage::success(format!("Inserted {value} into the B-Tree."))
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
            (Self::BTree(tree), InputAction::Delete) => match tree.delete_value(&value) {
                Some(deleted) => {
                    StatusMessage::success(format!("Deleted {deleted} from the B-Tree."))
                }
                None => StatusMessage::error(format!("Value {value} was not found in the B-Tree.")),
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
            (Self::BTree(tree), InputAction::Search) => {
                if tree.contains(&value) {
                    StatusMessage::success(format!("Value {value} exists in the B-Tree."))
                } else {
                    StatusMessage::error(format!("Value {value} does not exist in the B-Tree."))
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
            (Self::BTree(tree), InputAction::PredSucc) => {
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
            (Self::BinomialHeap(heap), InputAction::Insert) => {
                heap.insert(value);
                StatusMessage::success(format!("Inserted {value} into the binomial heap."))
            }
            (Self::BinomialHeap(heap), InputAction::Delete) => match heap.search(&value) {
                Some(handle) => {
                    heap.delete(handle);
                    StatusMessage::success(format!("Deleted {value} from the binomial heap."))
                }
                None => {
                    StatusMessage::error(format!("Value {value} not found in the binomial heap."))
                }
            },
            (Self::BinomialHeap(_), _) => {
                StatusMessage::error("Operation not supported for binomial heap.")
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
        let n = self.tree.action_count();
        self.selected_action = (self.selected_action + 1) % n;
    }

    pub fn previous_action(&mut self) {
        let n = self.tree.action_count();
        self.selected_action = if self.selected_action == 0 {
            n - 1
        } else {
            self.selected_action - 1
        };
    }
}
