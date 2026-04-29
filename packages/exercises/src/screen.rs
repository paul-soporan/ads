use crate::applications::{
    GraphUnionFindState, LeaderboardState, MedianStreamState, ProvinceCounterState,
    SocialNetworkState,
};
use crate::interactive::InteractiveState;
use crate::menu::MenuState;
use crate::showcase::ShowcaseState;

pub enum Screen {
    MainMenu(MenuState),
    ShowcaseMenu(MenuState),
    InteractiveMenu(MenuState),
    ApplicationsMenu(MenuState),
    Showcase(ShowcaseState),
    Interactive(InteractiveState),
    Leaderboard(LeaderboardState),
    MedianStream(MedianStreamState),
    GraphUnionFind(GraphUnionFindState),
    ProvinceCounter(ProvinceCounterState),
    SocialNetwork(SocialNetworkState),
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
            Self::Leaderboard(_) => "Dynamic Leaderboard",
            Self::MedianStream(_) => "Dynamic Median of a Data Stream",
            Self::GraphUnionFind(_) => "Graph Components + Cycle Detection",
            Self::ProvinceCounter(_) => "Province Counter from isConnected Matrix",
            Self::SocialNetwork(_) => "Social Network Friendship Groups",
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self {
            Self::MainMenu(_) => "↑/↓ move • Enter select • 1-4 shortcuts • q quit",
            Self::ShowcaseMenu(_) | Self::InteractiveMenu(_) => {
                "↑/↓ move • Enter select • 1-5 shortcuts • Esc back • q quit"
            }
            Self::ApplicationsMenu(_) => {
                "↑/↓ move • Enter select • 1-6 shortcuts • Esc back • q quit"
            }
            Self::Showcase(_) => "←/→ or p/n step • Esc back • q quit",
            Self::Interactive(state) => state.tree.help_text(),
            Self::Leaderboard(_)
            | Self::MedianStream(_)
            | Self::GraphUnionFind(_)
            | Self::ProvinceCounter(_)
            | Self::SocialNetwork(_) => {
                "Type command • Enter execute • Backspace edit • Esc back • q quit"
            }
        }
    }
}
