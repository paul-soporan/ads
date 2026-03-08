use ads::{BinarySearchTree, BstNodeHandle, NodeColor, RbNodeHandle, RedBlackTree};
use colored::Colorize;
use std::collections::BTreeMap;
use std::io::{self, Write};

// --- Tree Introspection & Measurement ---

fn bst_depth<T>(node: &Option<BstNodeHandle<T>>) -> usize {
    match node {
        Some(n) => 1 + std::cmp::max(bst_depth(&n.left()), bst_depth(&n.right())),
        None => 0,
    }
}

fn rb_depth<T>(node: &Option<RbNodeHandle<T>>) -> usize {
    match node {
        Some(n) => 1 + std::cmp::max(rb_depth(&n.left()), rb_depth(&n.right())),
        None => 0,
    }
}

fn rb_black_height<T>(node: &Option<RbNodeHandle<T>>) -> usize {
    match node {
        Some(n) => {
            let left_bh = rb_black_height(&n.left());
            left_bh + if n.color() == NodeColor::Black { 1 } else { 0 }
        }
        None => 1,
    }
}

// --- Graphical Representation / Formatting ---

fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    io::stdout().flush().unwrap();
}

fn strip_ansi(s: &str) -> String {
    let mut res = String::new();
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            res.push(c);
        }
    }
    res
}

trait RenderNode {
    fn left_child(&self) -> Option<Self>
    where
        Self: Sized;
    fn right_child(&self) -> Option<Self>
    where
        Self: Sized;
    fn text(&self) -> String;
}

impl RenderNode for BstNodeHandle<i32> {
    fn left_child(&self) -> Option<Self> {
        self.left()
    }
    fn right_child(&self) -> Option<Self> {
        self.right()
    }
    fn text(&self) -> String {
        format!("{t}", t = (*self.value()).to_string().cyan().bold())
    }
}

impl RenderNode for RbNodeHandle<i32> {
    fn left_child(&self) -> Option<Self> {
        self.left()
    }
    fn right_child(&self) -> Option<Self> {
        self.right()
    }
    fn text(&self) -> String {
        let val = *self.value();
        if self.color() == NodeColor::Red {
            format!("{t}", t = format!("{val}(R)").red().bold())
        } else {
            format!("{t}", t = format!("{val}(B)").white().bold())
        }
    }
}

struct TreeCanvas {
    grid: BTreeMap<(usize, usize), String>,
}

impl TreeCanvas {
    fn new() -> Self {
        Self {
            grid: BTreeMap::new(),
        }
    }

    fn put(&mut self, row: usize, col: usize, text: String) {
        self.grid.insert((row, col), text);
    }

    fn render(&self) {
        if self.grid.is_empty() {
            return;
        }
        let max_row = self.grid.keys().map(|(r, _)| *r).max().unwrap();

        for r in 0..=max_row {
            let mut curr_col = 0;
            let mut row_items: Vec<_> =
                self.grid.iter().filter(|((row, _), _)| *row == r).collect();
            row_items.sort_by_key(|((_, col), _)| *col);

            let mut line = String::new();
            for ((_, col), text) in row_items {
                let text_len = strip_ansi(text).chars().count();
                if *col > curr_col {
                    line.push_str(&" ".repeat(*col - curr_col));
                    curr_col = *col;
                }
                line.push_str(text);
                curr_col += text_len;
            }
            println!("{line}");
        }
    }
}

fn build_layout<N: RenderNode>(
    node: &Option<N>,
    level: usize,
    x: &mut usize,
    canvas: &mut TreeCanvas,
) -> Option<(usize, usize)> {
    let n = match node {
        Some(n) => n,
        None => return None,
    };

    let left_pos = build_layout(&n.left_child(), level + 1, x, canvas);

    let text = n.text();
    let visible_len = strip_ansi(&text).chars().count();
    let pad = 3;

    let my_x = *x + visible_len / 2;
    let my_row = level * 2;

    let col_start = my_x.saturating_sub(visible_len / 2);
    canvas.put(my_row, col_start, text.clone());

    *x = col_start + visible_len + pad;

    let right_pos = build_layout(&n.right_child(), level + 1, x, canvas);

    if let Some((lx, _)) = left_pos {
        let edge_row = my_row + 1;
        let edge_col = (my_x + lx) / 2;
        canvas.put(edge_row, edge_col, format!("{t}", t = "╱".bright_black()));
    }
    if let Some((rx, _)) = right_pos {
        let edge_row = my_row + 1;
        let edge_col = (my_x + rx) / 2;
        canvas.put(edge_row, edge_col, format!("{t}", t = "╲".bright_black()));
    }

    Some((my_x, my_row))
}

fn display_bst(tree: &BinarySearchTree<i32>) {
    let root = tree.root();
    let depth = bst_depth(&root);
    let sep = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black();
    let title = format!("  Binary Search Tree State (Depth: {depth})")
        .magenta()
        .bold();

    println!("\n{sep}");
    println!("{title}");
    println!("{sep}");

    if root.is_none() {
        let empty_msg = "(Empty Tree)".italic().dimmed();
        println!("  {empty_msg}\n");
    } else {
        println!();
        let mut canvas = TreeCanvas::new();
        let mut x = 0;
        build_layout(&root, 0, &mut x, &mut canvas);
        canvas.render();
        println!();
    }
    println!("{sep}\n");
}

fn display_rb(tree: &RedBlackTree<i32>) {
    let root = tree.root();
    let depth = rb_depth(&root);
    let bh = rb_black_height(&root);
    let sep = "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black();
    let title = format!("  Red-Black Tree State (Depth: {depth}, Black Height: {bh})")
        .magenta()
        .bold();

    println!("\n{sep}");
    println!("{title}");
    println!("{sep}");

    if root.is_none() {
        let empty_msg = "(Empty Tree)".italic().dimmed();
        println!("  {empty_msg}\n");
    } else {
        println!();
        let mut canvas = TreeCanvas::new();
        let mut x = 0;
        build_layout(&root, 0, &mut x, &mut canvas);
        canvas.render();
        println!();
    }
    println!("{sep}\n");
}

// --- Menus & Input Logic ---

fn read_input(prompt: &str) -> String {
    let colored_prompt = prompt.yellow();
    print!("{colored_prompt}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn read_i32(prompt: &str) -> Option<i32> {
    read_input(prompt).parse::<i32>().ok()
}

#[derive(Clone)]
enum Op {
    Insert(i32),
    Delete(i32),
}

fn predefined_showcase_bst() {
    let ops = [
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
    let mut step = 0;
    let total = ops.len();
    loop {
        clear_screen();
        let header = format!("❱❱❱ BST Showcase (Step {step}/{total}) ❰❰❰")
            .cyan()
            .bold();
        println!("{header}");

        let mut tree = BinarySearchTree::new();
        for op in ops.iter().take(step) {
            match op {
                Op::Insert(v) => tree.insert(*v),
                Op::Delete(v) => {
                    tree.delete_value(v);
                }
            }
        }

        if step > 0 {
            match &ops[step - 1] {
                Op::Insert(v) => {
                    let msg = format!("Insert {v}").green();
                    println!("Last Action: {msg}");
                }
                Op::Delete(v) => {
                    let msg = format!("Delete {v}").red();
                    println!("Last Action: {msg}");
                }
            }
        } else {
            let msg = "None (Initial State)".dimmed();
            println!("Last Action: {msg}");
        }

        display_bst(&tree);

        let controls = "[N]ext step  │  [P]revious step  │  [Q]uit showcase".bright_blue();
        println!("{controls}");

        let choice = read_input("Action ❯ ").to_lowercase();
        match choice.as_str() {
            "n" => {
                if step < ops.len() {
                    step += 1;
                }
            }
            "p" => {
                if step > 0 {
                    step -= 1;
                }
            }
            "q" => break,
            _ => {}
        }
    }
}

fn predefined_showcase_rb() {
    let ops = [
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
    let mut step = 0;
    let total = ops.len();
    loop {
        clear_screen();
        let header = format!("❱❱❱ Red-Black Tree Showcase (Step {step}/{total}) ❰❰❰")
            .cyan()
            .bold();
        println!("{header}");

        let mut tree = RedBlackTree::new();
        for op in ops.iter().take(step) {
            match op {
                Op::Insert(v) => tree.insert(*v),
                Op::Delete(v) => {
                    tree.delete_value(v);
                }
            }
        }

        if step > 0 {
            match &ops[step - 1] {
                Op::Insert(v) => {
                    let msg = format!("Insert {v}").green();
                    println!("Last Action: {msg}");
                }
                Op::Delete(v) => {
                    let msg = format!("Delete {v}").red();
                    println!("Last Action: {msg}");
                }
            }
        } else {
            let msg = "None (Initial State)".dimmed();
            println!("Last Action: {msg}");
        }

        display_rb(&tree);

        let controls = "[N]ext step  │  [P]revious step  │  [Q]uit showcase".bright_blue();
        println!("{controls}");

        let choice = read_input("Action ❯ ").to_lowercase();
        match choice.as_str() {
            "n" => {
                if step < ops.len() {
                    step += 1;
                }
            }
            "p" => {
                if step > 0 {
                    step -= 1;
                }
            }
            "q" => break,
            _ => {}
        }
    }
}

fn interactive_bst() {
    let mut tree = BinarySearchTree::new();
    loop {
        clear_screen();
        display_bst(&tree);

        let actions = "Actions: [1] Insert │ [2] Delete │ [3] Search │ [4] Min/Max │ [5] Pred/Succ │ [6] Back".bright_blue();
        println!("{actions}");

        match read_input("Select action ❯ ").as_str() {
            "1" => {
                if let Some(val) = read_i32("Value to insert ❯ ") {
                    tree.insert(val);
                }
            }
            "2" => {
                if let Some(val) = read_i32("Value to delete ❯ ") {
                    if let Some(deleted) = tree.delete_value(&val) {
                        let msg = format!("Deleted: {deleted}").green();
                        println!("{msg}");
                    } else {
                        let msg = "Value not found.".red();
                        println!("{msg}");
                    }
                    read_input("Press Enter to continue...");
                }
            }
            "3" => {
                if let Some(val) = read_i32("Value to search ❯ ") {
                    if tree.contains(&val) {
                        let msg = format!("Value {val} is in the tree.").green();
                        println!("{msg}");
                    } else {
                        let msg = format!("Value {val} not found.").red();
                        println!("{msg}");
                    }
                    read_input("Press Enter to continue...");
                }
            }
            "4" => {
                let min_val = tree.min().map(|h| *h.value());
                let max_val = tree.max().map(|h| *h.value());
                let min_msg = format!("Min: {min_val:?}").cyan();
                let max_msg = format!("Max: {max_val:?}").cyan();
                println!("{min_msg}");
                println!("{max_msg}");
                read_input("Press Enter to continue...");
            }
            "5" => {
                if let Some(val) = read_i32("Base value ❯ ") {
                    let pred = tree.predecessor_of_value(&val).map(|h| *h.value());
                    let succ = tree.successor_of_value(&val).map(|h| *h.value());
                    let pred_msg = format!("Predecessor: {pred:?}").cyan();
                    let succ_msg = format!("Successor: {succ:?}").cyan();
                    println!("{pred_msg}");
                    println!("{succ_msg}");
                    read_input("Press Enter to continue...");
                }
            }
            "6" => break,
            _ => {
                let msg = "Invalid selection. Please try again.".red();
                println!("{msg}");
                read_input("Press Enter to continue...");
            }
        }
    }
}

fn interactive_rb() {
    let mut tree = RedBlackTree::new();
    loop {
        clear_screen();
        display_rb(&tree);

        let actions = "Actions: [1] Insert │ [2] Delete │ [3] Search │ [4] Min/Max │ [5] Pred/Succ │ [6] Back".bright_blue();
        println!("{actions}");

        match read_input("Select action ❯ ").as_str() {
            "1" => {
                if let Some(val) = read_i32("Value to insert ❯ ") {
                    tree.insert(val);
                }
            }
            "2" => {
                if let Some(val) = read_i32("Value to delete ❯ ") {
                    if let Some(deleted) = tree.delete_value(&val) {
                        let msg = format!("Deleted: {deleted}").green();
                        println!("{msg}");
                    } else {
                        let msg = "Value not found.".red();
                        println!("{msg}");
                    }
                    read_input("Press Enter to continue...");
                }
            }
            "3" => {
                if let Some(val) = read_i32("Value to search ❯ ") {
                    if tree.contains(&val) {
                        let msg = format!("Value {val} is in the tree.").green();
                        println!("{msg}");
                    } else {
                        let msg = format!("Value {val} not found.").red();
                        println!("{msg}");
                    }
                    read_input("Press Enter to continue...");
                }
            }
            "4" => {
                let min_val = tree.min().map(|h| *h.value());
                let max_val = tree.max().map(|h| *h.value());
                let min_msg = format!("Min: {min_val:?}").cyan();
                let max_msg = format!("Max: {max_val:?}").cyan();
                println!("{min_msg}");
                println!("{max_msg}");
                read_input("Press Enter to continue...");
            }
            "5" => {
                if let Some(val) = read_i32("Base value ❯ ") {
                    let pred = tree.predecessor_of_value(&val).map(|h| *h.value());
                    let succ = tree.successor_of_value(&val).map(|h| *h.value());
                    let pred_msg = format!("Predecessor: {pred:?}").cyan();
                    let succ_msg = format!("Successor: {succ:?}").cyan();
                    println!("{pred_msg}");
                    println!("{succ_msg}");
                    read_input("Press Enter to continue...");
                }
            }
            "6" => break,
            _ => {
                let msg = "Invalid selection. Please try again.".red();
                println!("{msg}");
                read_input("Press Enter to continue...");
            }
        }
    }
}

fn main() {
    loop {
        clear_screen();
        let title =
            "╭─────────────────────────╮\n│        MAIN MENU        │\n╰─────────────────────────╯"
                .cyan()
                .bold();
        println!("{title}");
        println!("  1. Predefined Showcase");
        println!("  2. Interactive Mode");
        println!("  3. Exit\n");

        match read_input("Select mode ❯ ").as_str() {
            "1" => loop {
                clear_screen();
                let subtitle = "╭─────────────────────────╮\n│   PREDEFINED SHOWCASE   │\n╰─────────────────────────╯".cyan().bold();
                println!("{subtitle}");
                println!("  1. Binary Search Tree (BST)");
                println!("  2. Red-Black Tree");
                println!("  3. Go Back\n");

                match read_input("Select Data Structure ❯ ").as_str() {
                    "1" => {
                        predefined_showcase_bst();
                        break;
                    }
                    "2" => {
                        predefined_showcase_rb();
                        break;
                    }
                    "3" => break,
                    _ => {
                        let msg = "Invalid selection.".red();
                        println!("{msg}");
                        read_input("Press Enter to continue...");
                    }
                }
            },
            "2" => loop {
                clear_screen();
                let subtitle = "╭─────────────────────────╮\n│    INTERACTIVE MODE     │\n╰─────────────────────────╯".cyan().bold();
                println!("{subtitle}");
                println!("  1. Binary Search Tree (BST)");
                println!("  2. Red-Black Tree");
                println!("  3. Go Back\n");

                match read_input("Select Data Structure ❯ ").as_str() {
                    "1" => {
                        interactive_bst();
                        break;
                    }
                    "2" => {
                        interactive_rb();
                        break;
                    }
                    "3" => break,
                    _ => {
                        let msg = "Invalid selection.".red();
                        println!("{msg}");
                        read_input("Press Enter to continue...");
                    }
                }
            },
            "3" => {
                let msg = "Exiting program. Goodbye!".green();
                println!("\n{msg}");
                break;
            }
            _ => {
                let msg = "Invalid selection. Please try again.".red();
                println!("{msg}");
                read_input("Press Enter to continue...");
            }
        }
    }
}
