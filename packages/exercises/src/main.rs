use ads::{BinarySearchTree, BstNodeHandle, NodeColor, RbNodeHandle, RedBlackTree};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Line, Modifier, Span, Style, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    fmt::{self, Display, Formatter},
    io,
    time::Duration,
};

const MAIN_MENU_ITEMS: [&str; 4] = [
    "Predefined Showcase",
    "Interactive Mode",
    "Applications",
    "Exit",
];
const DATA_STRUCTURE_ITEMS: [&str; 3] = ["Binary Search Tree (BST)", "Red-Black Tree", "Go Back"];
const APPLICATION_ITEMS: [&str; 3] = [
    "Dynamic Leaderboard",
    "Dynamic Median of a Data Stream",
    "Go Back",
];
const INTERACTIVE_ACTIONS: [&str; 6] = [
    "Insert",
    "Delete",
    "Search",
    "Min / Max",
    "Predecessor / Successor",
    "Back",
];

const BST_SHOWCASE_OPS: [Op; 16] = [
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

const RB_SHOWCASE_OPS: [Op; 12] = [
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

fn main() -> io::Result<()> {
    let terminal_guard = TerminalGuard::enter()?;

    let result = {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let mut app = App::new();
        app.run(&mut terminal)
    };

    drop(terminal_guard);
    result
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

struct App {
    screen: Screen,
    prompt: Option<PromptState>,
    status: StatusMessage,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            screen: Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)),
            prompt: None,
            status: StatusMessage::info("Welcome to ADS Explorer!"),
            should_quit: false,
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key);
                    }
                }
            }
        }

        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(area);

        let header = Paragraph::new(self.screen.title())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("ADS Explorer"))
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(header, chunks[0]);

        match &self.screen {
            Screen::MainMenu(menu)
            | Screen::ShowcaseMenu(menu)
            | Screen::InteractiveMenu(menu)
            | Screen::ApplicationsMenu(menu) => self.render_menu(frame, chunks[1], menu),
            Screen::Showcase(showcase) => self.render_showcase(frame, chunks[1], showcase),
            Screen::Interactive(interactive) => {
                self.render_interactive(frame, chunks[1], interactive)
            }
            Screen::Leaderboard(leaderboard) => {
                self.render_leaderboard(frame, chunks[1], leaderboard)
            }
            Screen::MedianStream(median) => self.render_median_stream(frame, chunks[1], median),
        }

        self.render_footer(frame, chunks[2]);

        if let Some(prompt) = &self.prompt {
            self.render_prompt(frame, area, prompt);
        }
    }

    fn render_menu(&self, frame: &mut Frame, area: Rect, menu: &MenuState) {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area);

        let items = menu
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| ListItem::new(format!("{}. {item}", index + 1)))
            .collect::<Vec<_>>();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("› ");

        let mut state = ListState::default();
        state.select(Some(menu.selected));
        frame.render_stateful_widget(list, columns[1], &mut state);
    }

    fn render_showcase(&self, frame: &mut Frame, area: Rect, showcase: &ShowcaseState) {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
            .split(area);

        let tree_text = showcase.tree_text();
        let tree_panel = Paragraph::new(tree_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(showcase.tree_title())
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(tree_panel, body[0]);

        let side = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),
                Constraint::Length(5),
                Constraint::Min(6),
            ])
            .split(body[1]);

        let stats = Paragraph::new(showcase.stats_text())
            .block(Block::default().borders(Borders::ALL).title("Stats"));
        frame.render_widget(stats, side[0]);

        let current_action = Paragraph::new(showcase.current_action_text())
            .block(Block::default().borders(Borders::ALL).title("Current Step"))
            .wrap(Wrap { trim: false });
        frame.render_widget(current_action, side[1]);

        let history = List::new(showcase.history_items()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Applied Operations"),
        );
        frame.render_widget(history, side[2]);
    }

    fn render_interactive(&self, frame: &mut Frame, area: Rect, interactive: &InteractiveState) {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
            .split(area);

        let tree_panel = Paragraph::new(interactive.tree.tree_text())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(interactive.tree.title())
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(tree_panel, body[0]);

        let side = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(8)])
            .split(body[1]);

        let stats = Paragraph::new(interactive.tree.stats_text())
            .block(Block::default().borders(Borders::ALL).title("Tree Details"))
            .wrap(Wrap { trim: false });
        frame.render_widget(stats, side[0]);

        let action_items = INTERACTIVE_ACTIONS
            .iter()
            .enumerate()
            .map(|(index, item)| ListItem::new(format!("{}. {item}", index + 1)))
            .collect::<Vec<_>>();

        let actions = List::new(action_items)
            .block(Block::default().borders(Borders::ALL).title("Actions"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("› ");

        let mut state = ListState::default();
        state.select(Some(interactive.selected_action));
        frame.render_stateful_widget(actions, side[1], &mut state);
    }

    fn render_leaderboard(&self, frame: &mut Frame, area: Rect, leaderboard: &LeaderboardState) {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(4)])
            .split(body[0]);

        let output = Paragraph::new(leaderboard.output_text())
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .wrap(Wrap { trim: false });
        frame.render_widget(output, left[0]);

        let prompt = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "> ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(leaderboard.input.as_str()),
            ]),
            Line::from(Span::styled(
                "Format: ADD player score | UPDATE player delta | REMOVE player | TOP k",
                Style::default().fg(Color::DarkGray),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Input"),
        )
        .wrap(Wrap { trim: false });
        frame.render_widget(prompt, left[1]);

        let tree_panel = Paragraph::new(leaderboard.tree_text())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Red-Black Tree")
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(tree_panel, body[1]);
    }

    fn render_median_stream(&self, frame: &mut Frame, area: Rect, median: &MedianStreamState) {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(6)])
            .split(body[0]);

        let info_panel = Paragraph::new(median.info_text())
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .wrap(Wrap { trim: false });
        frame.render_widget(info_panel, left[0]);

        let prompt = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "> ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(median.input.as_str()),
            ]),
            Line::from(Span::styled(
                "Format: ADD x | REMOVE x | MEDIAN",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Lower median is returned for an even count of elements.",
                Style::default().fg(Color::DarkGray),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Input"),
        )
        .wrap(Wrap { trim: false });
        frame.render_widget(prompt, left[1]);

        let side = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(8)])
            .split(body[1]);

        let stats = Paragraph::new(median.stats_text())
            .block(Block::default().borders(Borders::ALL).title("Stats"));
        frame.render_widget(stats, side[0]);

        let tree_panel = Paragraph::new(render_rb_tree_text(&median.tree))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Order-Statistic Tree")
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(tree_panel, side[1]);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let help = if self.prompt.is_some() {
            "Type a number, Enter to submit, Esc to cancel."
        } else {
            self.screen.help_text()
        };

        let footer = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(help, Style::default().fg(Color::Blue))),
            self.status.to_line(),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: false });

        frame.render_widget(footer, area);
    }

    fn render_prompt(&self, frame: &mut Frame, area: Rect, prompt: &PromptState) {
        let popup = centered_rect(60, 24, area);
        let input = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled("Value: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(prompt.buffer.as_str()),
            ]),
            Line::from(Span::styled(
                prompt.hint.as_str(),
                Style::default().fg(Color::DarkGray),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(prompt.title.as_str())
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Left);

        frame.render_widget(Clear, popup);
        frame.render_widget(input, popup);
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            self.should_quit = true;
            return;
        }

        if self.prompt.is_some() {
            self.handle_prompt_key(key);
            return;
        }

        if matches!(key.code, KeyCode::Char('q')) {
            self.should_quit = true;
            return;
        }

        let mut next_screen = None;
        let mut next_prompt = None;
        let mut next_status = None;

        match &mut self.screen {
            Screen::MainMenu(menu) => match key.code {
                KeyCode::Up => menu.previous(),
                KeyCode::Down => menu.next(),
                KeyCode::Enter => match menu.selected {
                    0 => {
                        next_screen =
                            Some(Screen::ShowcaseMenu(MenuState::new(&DATA_STRUCTURE_ITEMS)))
                    }
                    1 => {
                        next_screen = Some(Screen::InteractiveMenu(MenuState::new(
                            &DATA_STRUCTURE_ITEMS,
                        )))
                    }
                    2 => {
                        next_screen =
                            Some(Screen::ApplicationsMenu(MenuState::new(&APPLICATION_ITEMS)))
                    }
                    3 => self.should_quit = true,
                    _ => {}
                },
                KeyCode::Char(digit) => {
                    if let Some(selection) = digit_to_index(digit, menu.items.len()) {
                        menu.selected = selection;
                        match selection {
                            0 => {
                                next_screen = Some(Screen::ShowcaseMenu(MenuState::new(
                                    &DATA_STRUCTURE_ITEMS,
                                )))
                            }
                            1 => {
                                next_screen = Some(Screen::InteractiveMenu(MenuState::new(
                                    &DATA_STRUCTURE_ITEMS,
                                )))
                            }
                            2 => {
                                next_screen = Some(Screen::ApplicationsMenu(MenuState::new(
                                    &APPLICATION_ITEMS,
                                )))
                            }
                            3 => self.should_quit = true,
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Screen::ShowcaseMenu(menu) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)))
                }
                KeyCode::Up => menu.previous(),
                KeyCode::Down => menu.next(),
                KeyCode::Enter => {
                    next_screen = Some(match menu.selected {
                        0 => Screen::Showcase(ShowcaseState::new(TreeKind::Bst)),
                        1 => Screen::Showcase(ShowcaseState::new(TreeKind::Rb)),
                        _ => Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)),
                    });
                }
                KeyCode::Char(digit) => {
                    if let Some(selection) = digit_to_index(digit, menu.items.len()) {
                        menu.selected = selection;
                        next_screen = Some(match selection {
                            0 => Screen::Showcase(ShowcaseState::new(TreeKind::Bst)),
                            1 => Screen::Showcase(ShowcaseState::new(TreeKind::Rb)),
                            _ => Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)),
                        });
                    }
                }
                _ => {}
            },
            Screen::InteractiveMenu(menu) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)))
                }
                KeyCode::Up => menu.previous(),
                KeyCode::Down => menu.next(),
                KeyCode::Enter => {
                    next_screen = Some(match menu.selected {
                        0 => Screen::Interactive(InteractiveState::new(TreeKind::Bst)),
                        1 => Screen::Interactive(InteractiveState::new(TreeKind::Rb)),
                        _ => Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)),
                    });
                }
                KeyCode::Char(digit) => {
                    if let Some(selection) = digit_to_index(digit, menu.items.len()) {
                        menu.selected = selection;
                        next_screen = Some(match selection {
                            0 => Screen::Interactive(InteractiveState::new(TreeKind::Bst)),
                            1 => Screen::Interactive(InteractiveState::new(TreeKind::Rb)),
                            _ => Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)),
                        });
                    }
                }
                _ => {}
            },
            Screen::ApplicationsMenu(menu) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)))
                }
                KeyCode::Up => menu.previous(),
                KeyCode::Down => menu.next(),
                KeyCode::Enter => {
                    next_screen = Some(match menu.selected {
                        0 => Screen::Leaderboard(LeaderboardState::new()),
                        1 => Screen::MedianStream(MedianStreamState::new()),
                        _ => Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)),
                    });
                }
                KeyCode::Char(digit) => {
                    if let Some(selection) = digit_to_index(digit, menu.items.len()) {
                        menu.selected = selection;
                        next_screen = Some(match selection {
                            0 => Screen::Leaderboard(LeaderboardState::new()),
                            1 => Screen::MedianStream(MedianStreamState::new()),
                            _ => Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)),
                        });
                    }
                }
                _ => {}
            },
            Screen::Showcase(showcase) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::ShowcaseMenu(MenuState::new(&DATA_STRUCTURE_ITEMS)));
                }
                KeyCode::Right | KeyCode::Char('n') => showcase.next_step(),
                KeyCode::Left | KeyCode::Char('p') => showcase.previous_step(),
                _ => {}
            },
            Screen::Interactive(interactive) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::InteractiveMenu(MenuState::new(
                        &DATA_STRUCTURE_ITEMS,
                    )));
                }
                KeyCode::Up => interactive.previous_action(),
                KeyCode::Down => interactive.next_action(),
                KeyCode::Enter => {
                    let action = interactive.selected_action;
                    match action {
                        0 => {
                            next_prompt = Some(PromptState::new(
                                interactive.tree.kind(),
                                InputAction::Insert,
                                "Insert Value",
                                "Enter an integer to insert.",
                            ));
                        }
                        1 => {
                            next_prompt = Some(PromptState::new(
                                interactive.tree.kind(),
                                InputAction::Delete,
                                "Delete Value",
                                "Enter an integer to delete.",
                            ));
                        }
                        2 => {
                            next_prompt = Some(PromptState::new(
                                interactive.tree.kind(),
                                InputAction::Search,
                                "Search Value",
                                "Enter an integer to search for.",
                            ));
                        }
                        3 => {
                            let (min_value, max_value) = interactive.tree.min_max();
                            next_status = Some(StatusMessage::info(format!(
                                "Min: {} • Max: {}",
                                format_option(min_value),
                                format_option(max_value)
                            )));
                        }
                        4 => {
                            next_prompt = Some(PromptState::new(
                                interactive.tree.kind(),
                                InputAction::PredSucc,
                                "Base Value",
                                "Enter an integer to inspect neighbors.",
                            ));
                        }
                        5 => {
                            next_screen = Some(Screen::InteractiveMenu(MenuState::new(
                                &DATA_STRUCTURE_ITEMS,
                            )));
                        }
                        _ => {}
                    }
                }
                KeyCode::Char(digit) => {
                    if let Some(selection) = digit_to_index(digit, INTERACTIVE_ACTIONS.len()) {
                        interactive.selected_action = selection;
                    }
                }
                _ => {}
            },
            Screen::MedianStream(median) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::ApplicationsMenu(MenuState::new(&APPLICATION_ITEMS)))
                }
                KeyCode::Backspace => {
                    median.input.pop();
                }
                KeyCode::Enter => {
                    let command = median.input.trim().to_string();
                    if command.is_empty() {
                        next_status = Some(StatusMessage::error("Please enter a command."));
                    } else {
                        next_status = Some(median.execute_command(command.as_str()));
                        median.input.clear();
                    }
                }
                KeyCode::Char(ch) => {
                    if !ch.is_control() {
                        median.input.push(ch);
                    }
                }
                _ => {}
            },
            Screen::Leaderboard(leaderboard) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::ApplicationsMenu(MenuState::new(&APPLICATION_ITEMS)))
                }
                KeyCode::Backspace => {
                    leaderboard.input.pop();
                }
                KeyCode::Enter => {
                    let command = leaderboard.input.trim().to_string();
                    if command.is_empty() {
                        next_status = Some(StatusMessage::error("Please enter a command."));
                    } else {
                        next_status = Some(leaderboard.execute_command(command.as_str()));
                        leaderboard.input.clear();
                    }
                }
                KeyCode::Char(ch) => {
                    if !ch.is_control() {
                        leaderboard.input.push(ch);
                    }
                }
                _ => {}
            },
        }

        if let Some(screen) = next_screen {
            self.screen = screen;
            self.status = StatusMessage::info("Switched screen.");
        }

        if let Some(prompt) = next_prompt {
            self.prompt = Some(prompt);
        }

        if let Some(status) = next_status {
            self.status = status;
        }
    }

    fn handle_prompt_key(&mut self, key: KeyEvent) {
        let Some(prompt) = &mut self.prompt else {
            return;
        };

        match key.code {
            KeyCode::Esc => {
                self.prompt = None;
                self.status = StatusMessage::info("Cancelled input.");
            }
            KeyCode::Backspace => {
                prompt.buffer.pop();
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() || (ch == '-' && prompt.buffer.is_empty()) => {
                prompt.buffer.push(ch);
            }
            KeyCode::Enter => {
                let value = match prompt.buffer.parse::<i32>() {
                    Ok(value) => value,
                    Err(_) => {
                        self.status = StatusMessage::error("Please enter a valid 32-bit integer.");
                        return;
                    }
                };

                let mut next_status = None;
                if let Screen::Interactive(interactive) = &mut self.screen {
                    next_status = Some(interactive.tree.apply_input(prompt.action, value));
                }

                self.prompt = None;
                if let Some(status) = next_status {
                    self.status = status;
                }
            }
            _ => {}
        }
    }
}

enum Screen {
    MainMenu(MenuState),
    ShowcaseMenu(MenuState),
    InteractiveMenu(MenuState),
    ApplicationsMenu(MenuState),
    Showcase(ShowcaseState),
    Interactive(InteractiveState),
    Leaderboard(LeaderboardState),
    MedianStream(MedianStreamState),
}

impl Screen {
    fn title(&self) -> &'static str {
        match self {
            Self::MainMenu(_) => "Main Menu",
            Self::ShowcaseMenu(_) => "Predefined Showcase",
            Self::InteractiveMenu(_) => "Interactive Mode",
            Self::ApplicationsMenu(_) => "Applications",
            Self::Showcase(showcase) => showcase.screen_title(),
            Self::Interactive(interactive) => interactive.tree.screen_title(),
            Self::Leaderboard(_) => "Dynamic Leaderboard",
            Self::MedianStream(_) => "Dynamic Median of a Data Stream",
        }
    }

    fn help_text(&self) -> &'static str {
        match self {
            Self::MainMenu(_) => "↑/↓ move • Enter select • 1-4 shortcuts • q quit",
            Self::ShowcaseMenu(_) | Self::InteractiveMenu(_) => {
                "↑/↓ move • Enter select • 1-3 shortcuts • Esc back • q quit"
            }
            Self::ApplicationsMenu(_) => {
                "↑/↓ move • Enter select • 1-3 shortcuts • Esc back • q quit"
            }
            Self::Showcase(_) => "←/→ or p/n step • Esc back • q quit",
            Self::Interactive(_) => "↑/↓ move • Enter run • 1-6 shortcuts • Esc back • q quit",
            Self::Leaderboard(_) | Self::MedianStream(_) => {
                "Type command • Enter execute • Backspace edit • Esc back • q quit"
            }
        }
    }
}

struct MenuState {
    items: &'static [&'static str],
    selected: usize,
}

impl MenuState {
    fn new(items: &'static [&'static str]) -> Self {
        Self { items, selected: 0 }
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len();
    }

    fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.items.len() - 1
        } else {
            self.selected - 1
        };
    }
}

#[derive(Clone, Copy)]
enum Op {
    Insert(i32),
    Delete(i32),
}

impl Op {
    fn to_line(self) -> Line<'static> {
        match self {
            Self::Insert(value) => Line::from(vec![
                Span::styled("Insert ", Style::default().fg(Color::Green)),
                Span::raw(value.to_string()),
            ]),
            Self::Delete(value) => Line::from(vec![
                Span::styled("Delete ", Style::default().fg(Color::Red)),
                Span::raw(value.to_string()),
            ]),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TreeKind {
    Bst,
    Rb,
}

struct ShowcaseState {
    kind: TreeKind,
    step: usize,
}

impl ShowcaseState {
    fn new(kind: TreeKind) -> Self {
        Self { kind, step: 0 }
    }

    fn screen_title(&self) -> &'static str {
        match self.kind {
            TreeKind::Bst => "BST Showcase",
            TreeKind::Rb => "Red-Black Tree Showcase",
        }
    }

    fn tree_title(&self) -> &'static str {
        match self.kind {
            TreeKind::Bst => "Binary Search Tree",
            TreeKind::Rb => "Red-Black Tree",
        }
    }

    fn operations(&self) -> &'static [Op] {
        match self.kind {
            TreeKind::Bst => &BST_SHOWCASE_OPS,
            TreeKind::Rb => &RB_SHOWCASE_OPS,
        }
    }

    fn next_step(&mut self) {
        self.step = (self.step + 1).min(self.operations().len());
    }

    fn previous_step(&mut self) {
        self.step = self.step.saturating_sub(1);
    }

    fn current_action(&self) -> Option<Op> {
        self.step
            .checked_sub(1)
            .and_then(|index| self.operations().get(index).copied())
    }

    fn stats_text(&self) -> Text<'static> {
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
        }

        Text::from(lines)
    }

    fn current_action_text(&self) -> Text<'static> {
        match self.current_action() {
            Some(op) => Text::from(vec![Line::from("Last operation:"), op.to_line()]),
            None => Text::from(vec![
                Line::from("Last operation:"),
                Line::from("Initial empty state"),
            ]),
        }
    }

    fn history_items(&self) -> Vec<ListItem<'static>> {
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

    fn tree_text(&self) -> Text<'static> {
        match self.kind {
            TreeKind::Bst => render_bst_tree_text(&build_bst_showcase_tree(self.step)),
            TreeKind::Rb => render_rb_tree_text(&build_rb_showcase_tree(self.step)),
        }
    }
}

struct InteractiveState {
    tree: InteractiveTree,
    selected_action: usize,
}

impl InteractiveState {
    fn new(kind: TreeKind) -> Self {
        Self {
            tree: InteractiveTree::new(kind),
            selected_action: 0,
        }
    }

    fn next_action(&mut self) {
        self.selected_action = (self.selected_action + 1) % INTERACTIVE_ACTIONS.len();
    }

    fn previous_action(&mut self) {
        self.selected_action = if self.selected_action == 0 {
            INTERACTIVE_ACTIONS.len() - 1
        } else {
            self.selected_action - 1
        };
    }
}

enum InteractiveTree {
    Bst(BinarySearchTree<i32>),
    Rb(RedBlackTree<i32>),
}

impl InteractiveTree {
    fn new(kind: TreeKind) -> Self {
        match kind {
            TreeKind::Bst => Self::Bst(BinarySearchTree::new()),
            TreeKind::Rb => Self::Rb(RedBlackTree::new()),
        }
    }

    fn kind(&self) -> TreeKind {
        match self {
            Self::Bst(_) => TreeKind::Bst,
            Self::Rb(_) => TreeKind::Rb,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::Bst(_) => "Binary Search Tree",
            Self::Rb(_) => "Red-Black Tree",
        }
    }

    fn screen_title(&self) -> &'static str {
        match self {
            Self::Bst(_) => "BST Interactive",
            Self::Rb(_) => "Red-Black Interactive",
        }
    }

    fn tree_text(&self) -> Text<'static> {
        match self {
            Self::Bst(tree) => render_bst_tree_text(tree),
            Self::Rb(tree) => render_rb_tree_text(tree),
        }
    }

    fn min_max(&self) -> (Option<i32>, Option<i32>) {
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

    fn stats_text(&self) -> Text<'static> {
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

    fn apply_input(&mut self, action: InputAction, value: i32) -> StatusMessage {
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

#[derive(Clone, Eq, PartialEq)]
struct LeaderboardEntry {
    player: String,
    score: i32,
}

impl Display for LeaderboardEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.player, self.score)
    }
}

impl Ord for LeaderboardEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.player.cmp(&other.player))
    }
}

impl PartialOrd for LeaderboardEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct LeaderboardState {
    tree: RedBlackTree<LeaderboardEntry>,
    scores: HashMap<String, i32>,
    output: Vec<String>,
    input: String,
}

impl LeaderboardState {
    fn new() -> Self {
        Self {
            tree: RedBlackTree::new(),
            scores: HashMap::new(),
            output: Vec::new(),
            input: String::new(),
        }
    }

    fn execute_command(&mut self, raw_command: &str) -> StatusMessage {
        match LeaderboardCommand::parse(raw_command) {
            Ok(command) => self.apply(command),
            Err(message) => StatusMessage::error(message),
        }
    }

    fn apply(&mut self, command: LeaderboardCommand) -> StatusMessage {
        match command {
            LeaderboardCommand::Add { player, score } => {
                if self.scores.contains_key(&player) {
                    return StatusMessage::error(format!(
                        "ADD failed: player '{player}' already exists."
                    ));
                }

                let entry = LeaderboardEntry {
                    player: player.clone(),
                    score,
                };
                self.tree.insert(entry);
                self.scores.insert(player.clone(), score);
                StatusMessage::success(format!("Added {player} with score {score}."))
            }
            LeaderboardCommand::Update { player, delta } => {
                let Some(current_score) = self.scores.get(&player).copied() else {
                    return StatusMessage::error(format!(
                        "UPDATE failed: player '{player}' does not exist."
                    ));
                };

                let old_entry = LeaderboardEntry {
                    player: player.clone(),
                    score: current_score,
                };
                let _ = self.tree.delete_value(&old_entry);

                let new_score = current_score + delta;
                let new_entry = LeaderboardEntry {
                    player: player.clone(),
                    score: new_score,
                };
                self.tree.insert(new_entry);
                self.scores.insert(player.clone(), new_score);

                StatusMessage::success(format!("Updated {player}: {current_score} -> {new_score}."))
            }
            LeaderboardCommand::Remove { player } => {
                let Some(current_score) = self.scores.remove(&player) else {
                    return StatusMessage::error(format!(
                        "REMOVE failed: player '{player}' does not exist."
                    ));
                };

                let old_entry = LeaderboardEntry {
                    player: player.clone(),
                    score: current_score,
                };
                let _ = self.tree.delete_value(&old_entry);

                StatusMessage::success(format!("Removed {player} from the leaderboard."))
            }
            LeaderboardCommand::Top { k } => {
                let results = self.top_k(k);
                if results.is_empty() {
                    self.output.push("(no players)".to_string());
                } else {
                    self.output.extend(results);
                }
                self.output.push(String::new());

                StatusMessage::info(format!("TOP {k} executed."))
            }
        }
    }

    fn top_k(&self, k: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = self.tree.max();

        while let Some(node) = current {
            if result.len() >= k {
                break;
            }

            let entry = node.value().clone();
            result.push(format!("{} {}", entry.player, entry.score));
            current = self.tree.predecessor(&node);
        }

        result
    }

    fn output_text(&self) -> Text<'static> {
        if self.output.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Run TOP k to print results here.",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))]);
        }

        Text::from(
            self.output
                .iter()
                .map(|line| Line::from(line.clone()))
                .collect::<Vec<_>>(),
        )
    }

    fn tree_text(&self) -> Text<'static> {
        render_rb_tree_text_generic(&self.tree)
    }
}

enum LeaderboardCommand {
    Add { player: String, score: i32 },
    Update { player: String, delta: i32 },
    Remove { player: String },
    Top { k: usize },
}

impl LeaderboardCommand {
    fn parse(input: &str) -> Result<Self, String> {
        let parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.is_empty() {
            return Err("Command cannot be empty.".to_string());
        }

        match parts[0].to_ascii_uppercase().as_str() {
            "ADD" => {
                if parts.len() != 3 {
                    return Err("ADD format: ADD player score".to_string());
                }

                let score = parts[2]
                    .parse::<i32>()
                    .map_err(|_| "ADD expects score to be an integer.".to_string())?;
                Ok(Self::Add {
                    player: parts[1].to_string(),
                    score,
                })
            }
            "UPDATE" => {
                if parts.len() != 3 {
                    return Err("UPDATE format: UPDATE player delta".to_string());
                }

                let delta = parts[2]
                    .parse::<i32>()
                    .map_err(|_| "UPDATE expects delta to be an integer.".to_string())?;
                Ok(Self::Update {
                    player: parts[1].to_string(),
                    delta,
                })
            }
            "REMOVE" => {
                if parts.len() != 2 {
                    return Err("REMOVE format: REMOVE player".to_string());
                }
                Ok(Self::Remove {
                    player: parts[1].to_string(),
                })
            }
            "TOP" => {
                if parts.len() != 2 {
                    return Err("TOP format: TOP k".to_string());
                }

                let k = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "TOP expects k to be a non-negative integer.".to_string())?;
                Ok(Self::Top { k })
            }
            _ => Err("Unknown command. Use ADD, UPDATE, REMOVE, or TOP.".to_string()),
        }
    }
}

struct MedianStreamState {
    tree: RedBlackTree<i32>,
    output: Vec<String>,
    pub input: String,
}

impl MedianStreamState {
    fn new() -> Self {
        Self {
            tree: RedBlackTree::new(),
            output: Vec::new(),
            input: String::new(),
        }
    }

    fn execute_command(&mut self, raw: &str) -> StatusMessage {
        match MedianCommand::parse(raw) {
            Err(e) => StatusMessage::error(e),
            Ok(MedianCommand::Add(x)) => {
                self.tree.insert(x);
                StatusMessage::info(format!("Inserted {x}. Size: {}", self.tree.size()))
            }
            Ok(MedianCommand::Remove(x)) => {
                let existed = self.tree.search(&x).is_some();
                if existed {
                    self.tree.delete_value(&x);
                    StatusMessage::info(format!("Removed {x}. Size: {}", self.tree.size()))
                } else {
                    StatusMessage::error(format!("{x} not found in stream."))
                }
            }
            Ok(MedianCommand::Median) => {
                let n = self.tree.size();
                if n == 0 {
                    StatusMessage::error("Stream is empty — no median.")
                } else {
                    let rank = (n - 1) / 2;
                    match self.tree.select(rank) {
                        Some(handle) => {
                            let median = *handle.value();
                            self.output.push(median.to_string());
                            StatusMessage::info(format!("Median: {median}"))
                        }
                        None => StatusMessage::error("Could not locate median node."),
                    }
                }
            }
        }
    }

    fn info_text(&self) -> Text<'static> {
        if self.output.is_empty() {
            return Text::from(vec![Line::from(Span::styled(
                "Run MEDIAN to print results here.",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))]);
        }
        Text::from(
            self.output
                .iter()
                .map(|line| Line::from(line.clone()))
                .collect::<Vec<_>>(),
        )
    }

    fn stats_text(&self) -> Text<'static> {
        let n = self.tree.size();
        let median_str = if n == 0 {
            "—".to_string()
        } else {
            let rank = (n - 1) / 2;
            self.tree
                .select(rank)
                .map(|h| h.value().to_string())
                .unwrap_or_else(|| "?".to_string())
        };
        Text::from(vec![
            Line::from(format!("Elements : {n}")),
            Line::from(format!("Median   : {median_str}")),
        ])
    }
}

enum MedianCommand {
    Add(i32),
    Remove(i32),
    Median,
}

impl MedianCommand {
    fn parse(input: &str) -> Result<Self, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Command cannot be empty.".to_string());
        }
        match parts[0].to_ascii_uppercase().as_str() {
            "ADD" => {
                if parts.len() != 2 {
                    return Err("ADD format: ADD <integer>".to_string());
                }
                let x = parts[1]
                    .parse::<i32>()
                    .map_err(|_| "ADD expects an integer argument.".to_string())?;
                Ok(Self::Add(x))
            }
            "REMOVE" => {
                if parts.len() != 2 {
                    return Err("REMOVE format: REMOVE <integer>".to_string());
                }
                let x = parts[1]
                    .parse::<i32>()
                    .map_err(|_| "REMOVE expects an integer argument.".to_string())?;
                Ok(Self::Remove(x))
            }
            "MEDIAN" => Ok(Self::Median),
            _ => Err("Unknown command. Use ADD, REMOVE, or MEDIAN.".to_string()),
        }
    }
}

struct PromptState {
    action: InputAction,
    title: String,
    hint: String,
    buffer: String,
}

impl PromptState {
    fn new(tree_kind: TreeKind, action: InputAction, title: &str, hint: &str) -> Self {
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

#[derive(Clone, Copy)]
enum InputAction {
    Insert,
    Delete,
    Search,
    PredSucc,
}

struct StatusMessage {
    text: String,
    style: Style,
}

impl StatusMessage {
    fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(Color::Cyan),
        }
    }

    fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(Color::Green),
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(Color::Red),
        }
    }

    fn to_line(&self) -> Line<'static> {
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(self.text.clone(), self.style),
        ])
    }
}

fn digit_to_index(ch: char, max: usize) -> Option<usize> {
    ch.to_digit(10)
        .and_then(|value| usize::try_from(value).ok())
        .and_then(|value| value.checked_sub(1))
        .filter(|index| *index < max)
}

fn format_option(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "None".to_string())
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn build_bst_showcase_tree(step: usize) -> BinarySearchTree<i32> {
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

fn build_rb_showcase_tree(step: usize) -> RedBlackTree<i32> {
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

fn bst_depth<T>(node: &Option<BstNodeHandle<T>>) -> usize {
    match node {
        Some(node) => 1 + usize::max(bst_depth(&node.left()), bst_depth(&node.right())),
        None => 0,
    }
}

fn rb_depth<T>(node: &Option<RbNodeHandle<T>>) -> usize {
    match node {
        Some(node) => 1 + usize::max(rb_depth(&node.left()), rb_depth(&node.right())),
        None => 0,
    }
}

fn rb_black_height<T>(node: &Option<RbNodeHandle<T>>) -> usize {
    match node {
        Some(node) => {
            let left_height = rb_black_height(&node.left());
            left_height + usize::from(node.color() == NodeColor::Black)
        }
        None => 1,
    }
}

struct NodeLabel {
    text: String,
    style: Style,
}

trait RenderNode {
    fn left_child(&self) -> Option<Self>
    where
        Self: Sized;
    fn right_child(&self) -> Option<Self>
    where
        Self: Sized;
    fn label(&self) -> NodeLabel;
}

impl RenderNode for BstNodeHandle<i32> {
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

impl<T> RenderNode for RbNodeHandle<T>
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

struct TreeCanvas {
    grid: BTreeMap<(usize, usize), StyledCell>,
}

impl TreeCanvas {
    fn new() -> Self {
        Self {
            grid: BTreeMap::new(),
        }
    }

    fn put(&mut self, row: usize, col: usize, text: impl Into<String>, style: Style) {
        self.grid.insert(
            (row, col),
            StyledCell {
                text: text.into(),
                style,
            },
        );
    }

    fn into_text(self) -> Text<'static> {
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

fn build_layout<N: RenderNode>(
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

fn render_bst_tree_text(tree: &BinarySearchTree<i32>) -> Text<'static> {
    render_tree_text(&tree.root())
}

fn render_rb_tree_text(tree: &RedBlackTree<i32>) -> Text<'static> {
    render_rb_tree_text_generic(tree)
}

fn render_rb_tree_text_generic<T>(tree: &RedBlackTree<T>) -> Text<'static>
where
    T: Ord + Display,
{
    render_tree_text(&tree.root())
}

fn render_tree_text<N: RenderNode>(root: &Option<N>) -> Text<'static> {
    let mut canvas = TreeCanvas::new();
    let mut cursor_x = 0;
    let _ = build_layout(root, 0, &mut cursor_x, &mut canvas);
    canvas.into_text()
}
