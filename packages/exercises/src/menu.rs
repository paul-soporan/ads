pub const MAIN_MENU_ITEMS: [&str; 4] = [
    "Predefined Showcase",
    "Interactive Mode",
    "Applications",
    "Exit",
];
pub const DATA_STRUCTURE_ITEMS: [&str; 3] =
    ["Binary Search Tree (BST)", "Red-Black Tree", "Go Back"];
pub const APPLICATION_ITEMS: [&str; 3] = [
    "Dynamic Leaderboard",
    "Dynamic Median of a Data Stream",
    "Go Back",
];
pub const INTERACTIVE_ACTIONS: [&str; 6] = [
    "Insert",
    "Delete",
    "Search",
    "Min / Max",
    "Predecessor / Successor",
    "Back",
];

pub struct MenuState {
    pub items: &'static [&'static str],
    pub selected: usize,
}

impl MenuState {
    pub fn new(items: &'static [&'static str]) -> Self {
        Self { items, selected: 0 }
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len();
    }

    pub fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.items.len() - 1
        } else {
            self.selected - 1
        };
    }
}
