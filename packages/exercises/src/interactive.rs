use ads::{BTree, BinarySearchTree, BinomialHeap, RedBlackTree};
use ratatui::prelude::{Line, Text};

use crate::menu::{HEAP_INTERACTIVE_ACTIONS, INTERACTIVE_ACTIONS};
use crate::render::{
    binomial_heap_degrees_str, binomial_heap_total, bst_depth, btree_depth, btree_key_count,
    rb_black_height, rb_depth, render_binomial_heap_text, render_bst_tree_text, render_btree_text,
    render_rb_tree_text,
};
use crate::types::{InputAction, StatusMessage};
use crate::utils::format_option;

pub struct PromptState {
    pub action: InputAction,
    pub title: String,
    pub hint: &'static str,
    pub buffer: String,
}

impl PromptState {
    pub fn new(action: InputAction, title: impl Into<String>, hint: &'static str) -> Self {
        Self {
            action,
            title: title.into(),
            hint,
            buffer: String::new(),
        }
    }
}

pub enum TreeAction {
    NeedsInput {
        action: InputAction,
        title: String,
        hint: &'static str,
    },
    Completed(StatusMessage),
    Back,
}

pub trait VisualizableTree {
    fn title(&self) -> &'static str;
    fn screen_title(&self) -> &'static str;
    fn help_text(&self) -> &'static str;
    fn tree_text(&self) -> Text<'static>;
    fn stats_text(&self) -> Text<'static>;
    fn action_list(&self) -> &'static [&'static str];
    fn apply_input(&mut self, action: InputAction, value: i32) -> StatusMessage;
    fn handle_action(&mut self, action: usize) -> TreeAction;
    fn min_max(&self) -> (Option<i32>, Option<i32>);

    fn action_count(&self) -> usize {
        self.action_list().len()
    }
}

fn standard_tree_handle_action(
    tree: &dyn VisualizableTree,
    prefix: &'static str,
    action: usize,
) -> TreeAction {
    match action {
        0 => TreeAction::NeedsInput {
            action: InputAction::Insert,
            title: format!("{prefix} · Insert Value"),
            hint: "Enter an integer to insert.",
        },
        1 => TreeAction::NeedsInput {
            action: InputAction::Delete,
            title: format!("{prefix} · Delete Value"),
            hint: "Enter an integer to delete.",
        },
        2 => TreeAction::NeedsInput {
            action: InputAction::Search,
            title: format!("{prefix} · Search Value"),
            hint: "Enter an integer to search for.",
        },
        3 => {
            let (min_value, max_value) = tree.min_max();
            TreeAction::Completed(StatusMessage::info(format!(
                "Min: {} • Max: {}",
                format_option(min_value),
                format_option(max_value)
            )))
        }
        4 => TreeAction::NeedsInput {
            action: InputAction::PredSucc,
            title: format!("{prefix} · Base Value"),
            hint: "Enter an integer to inspect neighbors.",
        },
        _ => TreeAction::Back,
    }
}

// ---- BST ----

struct BstInteractive {
    tree: BinarySearchTree<i32>,
}

impl BstInteractive {
    fn new() -> Self {
        Self {
            tree: BinarySearchTree::new(),
        }
    }
}

impl VisualizableTree for BstInteractive {
    fn title(&self) -> &'static str {
        "Binary Search Tree"
    }

    fn screen_title(&self) -> &'static str {
        "BST Interactive"
    }

    fn help_text(&self) -> &'static str {
        "↑/↓ move • Enter run • 1-6 shortcuts • Esc back • q quit"
    }

    fn tree_text(&self) -> Text<'static> {
        render_bst_tree_text(&self.tree)
    }

    fn stats_text(&self) -> Text<'static> {
        let root = self.tree.root();
        let (min_value, max_value) = self.min_max();
        Text::from(vec![
            Line::from(format!("Depth: {}", bst_depth(&root))),
            Line::from(format!("Root: {}", format_option(root.map(|n| *n.value())))),
            Line::from(format!("Min: {}", format_option(min_value))),
            Line::from(format!("Max: {}", format_option(max_value))),
        ])
    }

    fn action_list(&self) -> &'static [&'static str] {
        &INTERACTIVE_ACTIONS
    }

    fn min_max(&self) -> (Option<i32>, Option<i32>) {
        (
            self.tree.min().map(|h| *h.value()),
            self.tree.max().map(|h| *h.value()),
        )
    }

    fn apply_input(&mut self, action: InputAction, value: i32) -> StatusMessage {
        match action {
            InputAction::Insert => {
                self.tree.insert(value);
                StatusMessage::success(format!("Inserted {value} into the BST."))
            }
            InputAction::Delete => match self.tree.delete_value(&value) {
                Some(deleted) => StatusMessage::success(format!("Deleted {deleted} from the BST.")),
                None => StatusMessage::error(format!("Value {value} was not found in the BST.")),
            },
            InputAction::Search => {
                if self.tree.contains(&value) {
                    StatusMessage::success(format!("Value {value} exists in the BST."))
                } else {
                    StatusMessage::error(format!("Value {value} does not exist in the BST."))
                }
            }
            InputAction::PredSucc => {
                let predecessor = self.tree.predecessor_of_value(&value).map(|h| *h.value());
                let successor = self.tree.successor_of_value(&value).map(|h| *h.value());
                StatusMessage::info(format!(
                    "Predecessor: {} • Successor: {}",
                    format_option(predecessor),
                    format_option(successor)
                ))
            }
        }
    }

    fn handle_action(&mut self, action: usize) -> TreeAction {
        standard_tree_handle_action(self, "BST", action)
    }
}

// ---- Red-Black Tree ----

struct RbInteractive {
    tree: RedBlackTree<i32>,
}

impl RbInteractive {
    fn new() -> Self {
        Self {
            tree: RedBlackTree::new(),
        }
    }
}

impl VisualizableTree for RbInteractive {
    fn title(&self) -> &'static str {
        "Red-Black Tree"
    }

    fn screen_title(&self) -> &'static str {
        "Red-Black Interactive"
    }

    fn help_text(&self) -> &'static str {
        "↑/↓ move • Enter run • 1-6 shortcuts • Esc back • q quit"
    }

    fn tree_text(&self) -> Text<'static> {
        render_rb_tree_text(&self.tree)
    }

    fn stats_text(&self) -> Text<'static> {
        let root = self.tree.root();
        let (min_value, max_value) = self.min_max();
        Text::from(vec![
            Line::from(format!("Depth: {}", rb_depth(&root))),
            Line::from(format!("Black height: {}", rb_black_height(&root))),
            Line::from(format!("Root: {}", format_option(root.map(|n| *n.value())))),
            Line::from(format!("Min: {}", format_option(min_value))),
            Line::from(format!("Max: {}", format_option(max_value))),
        ])
    }

    fn action_list(&self) -> &'static [&'static str] {
        &INTERACTIVE_ACTIONS
    }

    fn min_max(&self) -> (Option<i32>, Option<i32>) {
        (
            self.tree.min().map(|h| *h.value()),
            self.tree.max().map(|h| *h.value()),
        )
    }

    fn apply_input(&mut self, action: InputAction, value: i32) -> StatusMessage {
        match action {
            InputAction::Insert => {
                self.tree.insert(value);
                StatusMessage::success(format!("Inserted {value} into the red-black tree."))
            }
            InputAction::Delete => match self.tree.delete_value(&value) {
                Some(deleted) => {
                    StatusMessage::success(format!("Deleted {deleted} from the red-black tree."))
                }
                None => StatusMessage::error(format!(
                    "Value {value} was not found in the red-black tree."
                )),
            },
            InputAction::Search => {
                if self.tree.contains(&value) {
                    StatusMessage::success(format!("Value {value} exists in the red-black tree."))
                } else {
                    StatusMessage::error(format!(
                        "Value {value} does not exist in the red-black tree."
                    ))
                }
            }
            InputAction::PredSucc => {
                let predecessor = self.tree.predecessor_of_value(&value).map(|h| *h.value());
                let successor = self.tree.successor_of_value(&value).map(|h| *h.value());
                StatusMessage::info(format!(
                    "Predecessor: {} • Successor: {}",
                    format_option(predecessor),
                    format_option(successor)
                ))
            }
        }
    }

    fn handle_action(&mut self, action: usize) -> TreeAction {
        standard_tree_handle_action(self, "Red-Black Tree", action)
    }
}

// ---- B-Tree ----

struct BTreeInteractive {
    tree: BTree<i32>,
}

impl BTreeInteractive {
    fn new() -> Self {
        Self {
            tree: BTree::new(2),
        }
    }
}

impl VisualizableTree for BTreeInteractive {
    fn title(&self) -> &'static str {
        "B-Tree (t = 2, internal=yellow, leaf=cyan)"
    }

    fn screen_title(&self) -> &'static str {
        "B-Tree Interactive"
    }

    fn help_text(&self) -> &'static str {
        "↑/↓ move • Enter run • 1-6 shortcuts • Esc back • q quit"
    }

    fn tree_text(&self) -> Text<'static> {
        render_btree_text(&self.tree)
    }

    fn stats_text(&self) -> Text<'static> {
        let root_view = self.tree.root_view();
        let (min_value, max_value) = self.min_max();
        Text::from(vec![
            Line::from(format!("Degree (t): {}", self.tree.degree())),
            Line::from(format!("Height: {}", btree_depth(&root_view))),
            Line::from(format!("Total keys: {}", btree_key_count(&root_view))),
            Line::from(format!("Min: {}", format_option(min_value))),
            Line::from(format!("Max: {}", format_option(max_value))),
        ])
    }

    fn action_list(&self) -> &'static [&'static str] {
        &INTERACTIVE_ACTIONS
    }

    fn min_max(&self) -> (Option<i32>, Option<i32>) {
        (
            self.tree.min().map(|h| *h.value()),
            self.tree.max().map(|h| *h.value()),
        )
    }

    fn apply_input(&mut self, action: InputAction, value: i32) -> StatusMessage {
        match action {
            InputAction::Insert => {
                self.tree.insert(value);
                StatusMessage::success(format!("Inserted {value} into the B-Tree."))
            }
            InputAction::Delete => match self.tree.delete_value(&value) {
                Some(deleted) => {
                    StatusMessage::success(format!("Deleted {deleted} from the B-Tree."))
                }
                None => StatusMessage::error(format!("Value {value} was not found in the B-Tree.")),
            },
            InputAction::Search => {
                if self.tree.contains(&value) {
                    StatusMessage::success(format!("Value {value} exists in the B-Tree."))
                } else {
                    StatusMessage::error(format!("Value {value} does not exist in the B-Tree."))
                }
            }
            InputAction::PredSucc => {
                let predecessor = self.tree.predecessor_of_value(&value).map(|h| *h.value());
                let successor = self.tree.successor_of_value(&value).map(|h| *h.value());
                StatusMessage::info(format!(
                    "Predecessor: {} • Successor: {}",
                    format_option(predecessor),
                    format_option(successor)
                ))
            }
        }
    }

    fn handle_action(&mut self, action: usize) -> TreeAction {
        standard_tree_handle_action(self, "B-Tree", action)
    }
}

// ---- Binomial Heap ----

struct BinomialHeapInteractive {
    heap: BinomialHeap<i32>,
}

impl BinomialHeapInteractive {
    fn new() -> Self {
        Self {
            heap: BinomialHeap::new(),
        }
    }
}

impl VisualizableTree for BinomialHeapInteractive {
    fn title(&self) -> &'static str {
        "Binomial Heap  (root=magenta, internal=green, leaf=cyan)"
    }

    fn screen_title(&self) -> &'static str {
        "Binomial Heap Interactive"
    }

    fn help_text(&self) -> &'static str {
        "↑/↓ move • Enter run • 1-5 shortcuts • Esc back • q quit"
    }

    fn tree_text(&self) -> Text<'static> {
        render_binomial_heap_text(&self.heap)
    }

    fn stats_text(&self) -> Text<'static> {
        Text::from(vec![
            Line::from(format!(
                "Total elements: {}",
                binomial_heap_total(&self.heap)
            )),
            Line::from(format!("Trees: {}", binomial_heap_degrees_str(&self.heap))),
            Line::from(format!(
                "Min: {}",
                format_option(self.heap.min().map(|v| *v.value()))
            )),
        ])
    }

    fn action_list(&self) -> &'static [&'static str] {
        &HEAP_INTERACTIVE_ACTIONS
    }

    fn min_max(&self) -> (Option<i32>, Option<i32>) {
        (self.heap.min().map(|v| *v.value()), None)
    }

    fn apply_input(&mut self, action: InputAction, value: i32) -> StatusMessage {
        match action {
            InputAction::Insert => {
                self.heap.insert(value);
                StatusMessage::success(format!("Inserted {value} into the binomial heap."))
            }
            InputAction::Delete => match self.heap.search(&value) {
                Some(handle) => {
                    self.heap.delete(handle);
                    StatusMessage::success(format!("Deleted {value} from the binomial heap."))
                }
                None => {
                    StatusMessage::error(format!("Value {value} not found in the binomial heap."))
                }
            },
            _ => StatusMessage::error("Operation not supported for binomial heap."),
        }
    }

    fn handle_action(&mut self, action: usize) -> TreeAction {
        match action {
            0 => TreeAction::NeedsInput {
                action: InputAction::Insert,
                title: "Binomial Heap · Insert Value".to_string(),
                hint: "Enter an integer to insert.",
            },
            1 => TreeAction::NeedsInput {
                action: InputAction::Delete,
                title: "Binomial Heap · Delete Value".to_string(),
                hint: "Enter an integer to delete.",
            },
            2 => TreeAction::Completed(match self.heap.extract_min() {
                Some(v) => StatusMessage::success(format!("Extracted minimum: {v}")),
                None => StatusMessage::error("Heap is empty."),
            }),
            3 => {
                let (min_val, _) = self.min_max();
                TreeAction::Completed(StatusMessage::info(format!(
                    "Min: {}",
                    format_option(min_val)
                )))
            }
            _ => TreeAction::Back,
        }
    }
}

pub struct TreeDefinition {
    pub title: &'static str,
    pub factory: fn() -> Box<dyn VisualizableTree>,
}

fn bst_tree_factory() -> Box<dyn VisualizableTree> {
    Box::new(BstInteractive::new())
}

fn rb_tree_factory() -> Box<dyn VisualizableTree> {
    Box::new(RbInteractive::new())
}

fn btree_tree_factory() -> Box<dyn VisualizableTree> {
    Box::new(BTreeInteractive::new())
}

fn binomial_tree_factory() -> Box<dyn VisualizableTree> {
    Box::new(BinomialHeapInteractive::new())
}

pub const TREE_DEFINITIONS: [TreeDefinition; 4] = [
    TreeDefinition {
        title: "Binary Search Tree (BST)",
        factory: bst_tree_factory,
    },
    TreeDefinition {
        title: "Red-Black Tree",
        factory: rb_tree_factory,
    },
    TreeDefinition {
        title: "B-Tree (order 2)",
        factory: btree_tree_factory,
    },
    TreeDefinition {
        title: "Binomial Heap",
        factory: binomial_tree_factory,
    },
];

pub const DATA_STRUCTURE_MENU_ITEMS: [&str; 5] = {
    let mut items = [""; 5];
    let mut i = 0;
    while i < TREE_DEFINITIONS.len() {
        items[i] = TREE_DEFINITIONS[i].title;
        i += 1;
    }
    items[4] = "Go Back";
    items
};

pub struct InteractiveState {
    pub tree: Box<dyn VisualizableTree>,
    pub selected_action: usize,
}

impl InteractiveState {
    pub fn new_from_factory(factory: fn() -> Box<dyn VisualizableTree>) -> Self {
        Self {
            tree: factory(),
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
