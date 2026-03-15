use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Line, Modifier, Span, Style, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{io, time::Duration};

use crate::apps::{LeaderboardState, MedianStreamState};
use crate::interactive::{InteractiveState, PromptState};
use crate::menu::{
    APPLICATION_ITEMS, DATA_STRUCTURE_ITEMS, INTERACTIVE_ACTIONS, MAIN_MENU_ITEMS, MenuState,
};
use crate::screen::Screen;
use crate::showcase::ShowcaseState;
use crate::types::{InputAction, StatusMessage, TreeKind};
use crate::utils::{centered_rect, digit_to_index, format_option};

pub struct App {
    pub screen: Screen,
    pub prompt: Option<PromptState>,
    pub status: StatusMessage,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS)),
            prompt: None,
            status: StatusMessage::info("Welcome to ADS Explorer!"),
            should_quit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
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

    pub fn render(&self, frame: &mut Frame) {
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

        let tree_panel = Paragraph::new(median.tree_text())
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

    pub fn handle_key(&mut self, key: KeyEvent) {
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
