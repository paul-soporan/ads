use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Line, Modifier, Span, Style, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{io, time::Duration};

use crate::applications::{
    APPLICATION_MENU_ITEMS, COMMAND_APPLICATION_DEFINITIONS, CommandApplication,
};
use crate::interactive::{
    DATA_STRUCTURE_MENU_ITEMS, InteractiveState, PromptState, TREE_DEFINITIONS, TreeAction,
};
use crate::menu::{MAIN_MENU_ITEMS, MenuState};
use crate::screen::{CommandApplicationScreen, Screen};
use crate::showcase::{SHOWCASE_FACTORIES, ShowcaseState};
use crate::types::StatusMessage;
use crate::utils::{centered_rect, digit_to_index};

pub struct App {
    pub screen: Screen,
    pub prompt: Option<PromptState>,
    pub status: StatusMessage,
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
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
            Screen::CommandApplication(screen) => {
                self.render_command_application(frame, chunks[1], screen.application.as_ref())
            }
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

        let action_items = interactive
            .tree
            .action_list()
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

    fn render_command_application(
        &self,
        frame: &mut Frame,
        area: Rect,
        application: &dyn CommandApplication,
    ) {
        let layout = application.layout();
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(layout.left_width_percent),
                Constraint::Percentage(100 - layout.left_width_percent),
            ])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(layout.input_height)])
            .split(body[0]);

        let output = Paragraph::new(application.output_text())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(layout.output_title),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(output, left[0]);

        let mut prompt_lines = vec![Line::from(vec![
            Span::styled(
                "> ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(application.input_buffer()),
        ])];
        prompt_lines.extend(application.input_hint_lines());

        let prompt = Paragraph::new(Text::from(prompt_lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(layout.input_title),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(prompt, left[1]);

        let side = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(layout.stats_height), Constraint::Min(8)])
            .split(body[1]);

        let stats = Paragraph::new(application.stats_text()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(layout.stats_title),
        );
        frame.render_widget(stats, side[0]);

        let state = Paragraph::new(application.state_text())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(layout.state_title)
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(state, side[1]);
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
                prompt.hint,
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
                KeyCode::Enter => {
                    let (screen, quit) = Self::navigate_from_main(menu.selected);
                    next_screen = screen;
                    if quit {
                        self.should_quit = true;
                    }
                }
                KeyCode::Char(digit) => {
                    if let Some(selection) = digit_to_index(digit, menu.items.len()) {
                        menu.selected = selection;
                        let (screen, quit) = Self::navigate_from_main(selection);
                        next_screen = screen;
                        if quit {
                            self.should_quit = true;
                        }
                    }
                }
                _ => {}
            },
            Screen::ShowcaseMenu(menu) => {
                next_screen = Self::handle_sub_menu_key(key, menu, Self::showcase_screen);
            }
            Screen::InteractiveMenu(menu) => {
                next_screen = Self::handle_sub_menu_key(key, menu, Self::interactive_screen);
            }
            Screen::ApplicationsMenu(menu) => {
                next_screen = Self::handle_sub_menu_key(key, menu, Self::application_screen);
            }
            Screen::Showcase(showcase) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::ShowcaseMenu(MenuState::new(
                        &DATA_STRUCTURE_MENU_ITEMS,
                    )));
                }
                KeyCode::Right | KeyCode::Char('n') => showcase.next_step(),
                KeyCode::Left | KeyCode::Char('p') => showcase.previous_step(),
                _ => {}
            },
            Screen::Interactive(interactive) => match key.code {
                KeyCode::Esc => {
                    next_screen = Some(Screen::InteractiveMenu(MenuState::new(
                        &DATA_STRUCTURE_MENU_ITEMS,
                    )));
                }
                KeyCode::Up => interactive.previous_action(),
                KeyCode::Down => interactive.next_action(),
                KeyCode::Enter => {
                    let action = interactive.selected_action;
                    match interactive.tree.handle_action(action) {
                        TreeAction::NeedsInput {
                            action,
                            title,
                            hint,
                        } => {
                            next_prompt = Some(PromptState::new(action, title, hint));
                        }
                        TreeAction::Completed(status) => {
                            next_status = Some(status);
                        }
                        TreeAction::Back => {
                            next_screen = Some(Screen::InteractiveMenu(MenuState::new(
                                &DATA_STRUCTURE_MENU_ITEMS,
                            )));
                        }
                    }
                }
                KeyCode::Char(digit) => {
                    if let Some(selection) = digit_to_index(digit, interactive.tree.action_count())
                    {
                        interactive.selected_action = selection;
                    }
                }
                _ => {}
            },
            Screen::CommandApplication(screen) => Self::handle_command_application_key(
                key,
                screen.application.as_mut(),
                &mut next_screen,
                &mut next_status,
            ),
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

    fn handle_command_application_key(
        key: KeyEvent,
        application: &mut dyn CommandApplication,
        next_screen: &mut Option<Screen>,
        next_status: &mut Option<StatusMessage>,
    ) {
        match key.code {
            KeyCode::Esc => {
                *next_screen = Some(Screen::ApplicationsMenu(MenuState::new(
                    &APPLICATION_MENU_ITEMS,
                )));
            }
            KeyCode::Backspace => {
                application.input_buffer_mut().pop();
            }
            KeyCode::Enter => {
                *next_status = Some(application.submit_input());
            }
            KeyCode::Char(ch) => {
                if !ch.is_control() {
                    application.input_buffer_mut().push(ch);
                }
            }
            _ => {}
        }
    }

    fn handle_sub_menu_key(
        key: KeyEvent,
        menu: &mut MenuState,
        navigate: fn(usize) -> Option<Screen>,
    ) -> Option<Screen> {
        match key.code {
            KeyCode::Esc => Some(Screen::MainMenu(MenuState::new(&MAIN_MENU_ITEMS))),
            KeyCode::Up => {
                menu.previous();
                None
            }
            KeyCode::Down => {
                menu.next();
                None
            }
            KeyCode::Enter => navigate(menu.selected),
            KeyCode::Char(digit) => digit_to_index(digit, menu.items.len()).and_then(|selection| {
                menu.selected = selection;
                navigate(selection)
            }),
            _ => None,
        }
    }

    fn navigate_from_main(selection: usize) -> (Option<Screen>, bool) {
        match selection {
            0 => (
                Some(Screen::ShowcaseMenu(MenuState::new(
                    &DATA_STRUCTURE_MENU_ITEMS,
                ))),
                false,
            ),
            1 => (
                Some(Screen::InteractiveMenu(MenuState::new(
                    &DATA_STRUCTURE_MENU_ITEMS,
                ))),
                false,
            ),
            2 => (
                Some(Screen::ApplicationsMenu(MenuState::new(
                    &APPLICATION_MENU_ITEMS,
                ))),
                false,
            ),
            3 => (None, true),
            _ => (None, false),
        }
    }

    fn showcase_screen(selection: usize) -> Option<Screen> {
        Some(Screen::Showcase(SHOWCASE_FACTORIES.get(selection)?()))
    }

    fn interactive_screen(selection: usize) -> Option<Screen> {
        let definition = TREE_DEFINITIONS.get(selection)?;
        Some(Screen::Interactive(InteractiveState::new_from_factory(
            definition.factory,
        )))
    }

    fn application_screen(selection: usize) -> Option<Screen> {
        let definition = COMMAND_APPLICATION_DEFINITIONS.get(selection)?;
        Some(Screen::CommandApplication(CommandApplicationScreen::new(
            definition.title,
            (definition.factory)(),
        )))
    }
}
