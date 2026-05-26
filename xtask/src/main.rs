mod aggregate;
mod bench_runner;
mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, CliCommand};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::Bench(args) => commands::bench::run(args),
        CliCommand::Aggregate(args) => commands::aggregate::run(args),
        CliCommand::Ci(args) => commands::ci::run(args),
    }
}
