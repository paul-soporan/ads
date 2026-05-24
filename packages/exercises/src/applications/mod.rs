pub mod core;
pub mod graph_union_find;
pub mod leaderboard;
pub mod median;
pub mod network_value_queries;
pub mod provinces;
pub mod social_network;

pub use core::{
    CommandApplication, CommandApplicationLayout, output_lines_to_text, parse_command_parts,
};
pub use graph_union_find::GraphUnionFindState;
pub use leaderboard::LeaderboardState;
pub use median::MedianStreamState;
pub use network_value_queries::NetworkValueQueryState;
pub use provinces::ProvinceCounterState;
pub use social_network::SocialNetworkState;

pub struct CommandApplicationDefinition {
    pub title: &'static str,
    pub factory: fn() -> Box<dyn CommandApplication>,
}

fn create_leaderboard() -> Box<dyn CommandApplication> {
    Box::new(LeaderboardState::new())
}

fn create_median_stream() -> Box<dyn CommandApplication> {
    Box::new(MedianStreamState::new())
}

fn create_graph_union_find() -> Box<dyn CommandApplication> {
    Box::new(GraphUnionFindState::new())
}

fn create_province_counter() -> Box<dyn CommandApplication> {
    Box::new(ProvinceCounterState::new())
}

fn create_social_network() -> Box<dyn CommandApplication> {
    Box::new(SocialNetworkState::new())
}

fn create_network_value_query() -> Box<dyn CommandApplication> {
    Box::new(NetworkValueQueryState::new())
}

pub const APPLICATION_MENU_ITEMS: [&str; 7] = {
    let mut items = [""; 7];
    let mut i = 0;
    while i < COMMAND_APPLICATION_DEFINITIONS.len() {
        items[i] = COMMAND_APPLICATION_DEFINITIONS[i].title;
        i += 1;
    }
    items[6] = "Go Back";
    items
};

pub const COMMAND_APPLICATION_DEFINITIONS: [CommandApplicationDefinition; 6] = [
    CommandApplicationDefinition {
        title: "Dynamic Leaderboard",
        factory: create_leaderboard,
    },
    CommandApplicationDefinition {
        title: "Dynamic Median of a Data Stream",
        factory: create_median_stream,
    },
    CommandApplicationDefinition {
        title: "Graph Components + Cycle Detection",
        factory: create_graph_union_find,
    },
    CommandApplicationDefinition {
        title: "Province Counter from isConnected Matrix",
        factory: create_province_counter,
    },
    CommandApplicationDefinition {
        title: "Social Network Friendship Groups",
        factory: create_social_network,
    },
    CommandApplicationDefinition {
        title: "Network Value Queries",
        factory: create_network_value_query,
    },
];
