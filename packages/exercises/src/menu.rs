pub const MAIN_MENU_ITEMS: [&str; 4] = [
    "Predefined Showcase",
    "Interactive Mode",
    "Applications",
    "Exit",
];
pub const INTERACTIVE_ACTIONS: [&str; 6] = [
    "Insert",
    "Delete",
    "Search",
    "Min / Max",
    "Predecessor / Successor",
    "Back",
];
pub const HEAP_INTERACTIVE_ACTIONS: [&str; 5] =
    ["Insert", "Delete", "Extract Min", "Show Min", "Back"];

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
