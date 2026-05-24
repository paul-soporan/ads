use crate::applications::CommandApplication;
use crate::interactive::InteractiveState;
use crate::menu::MenuState;
use crate::showcase::ShowcaseState;

pub struct CommandApplicationScreen {
    pub title: &'static str,
    pub application: Box<dyn CommandApplication>,
}

impl CommandApplicationScreen {
    pub fn new(title: &'static str, application: Box<dyn CommandApplication>) -> Self {
        Self { title, application }
    }
}

pub enum Screen {
    MainMenu(MenuState),
    ShowcaseMenu(MenuState),
    InteractiveMenu(MenuState),
    ApplicationsMenu(MenuState),
    Showcase(ShowcaseState),
    Interactive(InteractiveState),
    CommandApplication(CommandApplicationScreen),
}

impl Screen {
    pub fn title(&self) -> &'static str {
        match self {
            Self::MainMenu(_) => "Main Menu",
            Self::ShowcaseMenu(_) => "Predefined Showcase",
            Self::InteractiveMenu(_) => "Interactive Mode",
            Self::ApplicationsMenu(_) => "Applications",
            Self::Showcase(showcase) => showcase.screen_title(),
            Self::Interactive(interactive) => interactive.tree.screen_title(),
            Self::CommandApplication(screen) => screen.title,
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self {
            Self::MainMenu(_) => "↑/↓ move • Enter select • 1-4 shortcuts • q quit",
            Self::ShowcaseMenu(_) | Self::InteractiveMenu(_) => {
                "↑/↓ move • Enter select • 1-5 shortcuts • Esc back • q quit"
            }
            Self::ApplicationsMenu(_) => {
                "↑/↓ move • Enter select • 1-7 shortcuts • Esc back • q quit"
            }
            Self::Showcase(_) => "←/→ or p/n step • Esc back • q quit",
            Self::Interactive(state) => state.tree.help_text(),
            Self::CommandApplication(_) => {
                "Type command • Enter execute • Backspace edit • Esc back • q quit"
            }
        }
    }
}
