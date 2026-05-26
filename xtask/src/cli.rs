use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xtask", about = "ADS project task runner")]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Run one or more benchmark suites
    Bench(BenchArgs),
    /// Aggregate benchmark artifacts into a single JSON file
    Aggregate(AggregateArgs),
    /// Run the full pipeline: all suites -> aggregate -> JSON (one-shot CI command)
    Ci(CiArgs),
}

#[derive(Args)]
pub struct BenchArgs {
    /// Benchmark suites to run (repeat or use comma-separated values)
    #[arg(long = "suite", value_enum, value_delimiter = ',', default_values_t = vec![Suite::All])]
    pub suites: Vec<Suite>,
    /// Benchmark kinds to include (criterion, dhat, callgrind)
    #[arg(
        long = "kind",
        value_enum,
        value_delimiter = ',',
        default_values_t = vec![BenchKind::Criterion, BenchKind::Dhat, BenchKind::Callgrind]
    )]
    pub kinds: Vec<BenchKind>,
    /// Benchmark families to include (micro, macro, sweeps)
    #[arg(long = "family", value_enum, value_delimiter = ',', default_values_t = vec![BenchFamily::All])]
    pub families: Vec<BenchFamily>,
    /// Benchmark subcategory selector (e.g. micro_maps, macro_read_heavy)
    #[arg(long = "subcategory", value_delimiter = ',')]
    pub subcategories: Vec<String>,
    /// Run specific benchmark targets by target name (repeatable; accepts legacy --bench alias)
    #[arg(long = "target", alias = "bench", value_delimiter = ',')]
    pub targets: Vec<String>,
    /// Friendly benchmark selector (join-key form or criterion id)
    #[arg(long = "benchmark")]
    pub benchmark: Option<String>,
    /// Filter by workload token (e.g. micro_maps_u64)
    #[arg(long)]
    pub workload: Option<String>,
    /// Filter by payload token (e.g. u64, string, large_payload)
    #[arg(long)]
    pub payload: Option<String>,
    /// Filter by operation token (e.g. insert, contains_zipf, mix)
    #[arg(long = "op")]
    pub operation: Option<String>,
    /// Filter by implementation token (e.g. avl_arena)
    #[arg(long = "impl")]
    pub implementation: Option<String>,
    /// Filter by input size (e.g. 20000)
    #[arg(long)]
    pub size: Option<usize>,
    /// Filter by variant token (e.g. safe, raw, arena, std)
    #[arg(long)]
    pub variant: Option<String>,
    /// Use minimal Criterion settings for a fast smoke check
    #[arg(long)]
    pub smoke: bool,
    /// Pin execution to a CPU core via taskset (Linux only)
    #[arg(long)]
    pub pin_core: Option<String>,
    /// Run benchmark targets in parallel; sequential is the default
    #[arg(long, default_value_t = false)]
    pub parallel: bool,
    /// Maximum number of benchmarks to run concurrently (parallel mode only)
    #[arg(long)]
    pub jobs: Option<usize>,
    /// Only run benchmarks that are missing results or invalidated by source changes
    #[arg(long, default_value_t = false)]
    pub incremental: bool,
    /// Store artifacts in a temporary directory and delete them afterwards
    #[arg(long)]
    pub no_artifacts: bool,
    /// Run aggregator after all benchmarks complete
    #[arg(long)]
    pub aggregate: bool,
    /// Output path for the aggregated JSON (implies --aggregate)
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Args)]
pub struct AggregateArgs {
    /// Root of Criterion output
    #[arg(long, default_value = "target/criterion")]
    pub criterion_root: PathBuf,
    /// Root of Callgrind output files
    #[arg(long, default_value = "target")]
    pub callgrind_root: PathBuf,
    /// Root directory for dhat JSON files
    #[arg(long, default_value = "target/dhat")]
    pub dhat_root: PathBuf,
    /// Output path for the aggregated JSON
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Args)]
pub struct CiArgs {
    /// Benchmark suites to run (repeat or use comma-separated values)
    #[arg(long = "suite", value_enum, value_delimiter = ',', default_values_t = vec![Suite::All])]
    pub suites: Vec<Suite>,
    /// Benchmark kinds to include (criterion, dhat, callgrind)
    #[arg(
        long = "kind",
        value_enum,
        value_delimiter = ',',
        default_values_t = vec![BenchKind::Criterion, BenchKind::Dhat, BenchKind::Callgrind]
    )]
    pub kinds: Vec<BenchKind>,
    /// Benchmark families to include (micro, macro, sweeps)
    #[arg(long = "family", value_enum, value_delimiter = ',', default_values_t = vec![BenchFamily::All])]
    pub families: Vec<BenchFamily>,
    /// Benchmark subcategory selector (e.g. micro_maps, macro_read_heavy)
    #[arg(long = "subcategory", value_delimiter = ',')]
    pub subcategories: Vec<String>,
    /// Run specific benchmark targets by target name (repeatable; accepts legacy --bench alias)
    #[arg(long = "target", alias = "bench", value_delimiter = ',')]
    pub targets: Vec<String>,
    /// Friendly benchmark selector (join-key form or criterion id)
    #[arg(long = "benchmark")]
    pub benchmark: Option<String>,
    /// Filter by workload token (e.g. micro_maps_u64)
    #[arg(long)]
    pub workload: Option<String>,
    /// Filter by payload token (e.g. u64, string, large_payload)
    #[arg(long)]
    pub payload: Option<String>,
    /// Filter by operation token (e.g. insert, contains_zipf, mix)
    #[arg(long = "op")]
    pub operation: Option<String>,
    /// Filter by implementation token (e.g. avl_arena)
    #[arg(long = "impl")]
    pub implementation: Option<String>,
    /// Filter by input size (e.g. 20000)
    #[arg(long)]
    pub size: Option<usize>,
    /// Filter by variant token (e.g. safe, raw, arena, std)
    #[arg(long)]
    pub variant: Option<String>,
    /// Pin execution to a CPU core via taskset (Linux only)
    #[arg(long)]
    pub pin_core: Option<String>,
    /// Run benchmark targets in parallel; sequential is the default
    #[arg(long, default_value_t = false)]
    pub parallel: bool,
    /// Maximum number of benchmarks to run concurrently (parallel mode only)
    #[arg(long)]
    pub jobs: Option<usize>,
    /// Only run benchmarks that are missing results or invalidated by source changes
    #[arg(long, default_value_t = false)]
    pub incremental: bool,
    /// Output path for the aggregated JSON
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Clone, ValueEnum)]
pub enum Suite {
    All,
    Micro,
    Macro,
    Sweeps,
    Memory,
    Callgrind,
}

#[derive(Clone, ValueEnum, PartialEq, Eq)]
pub enum BenchKind {
    Criterion,
    Dhat,
    Callgrind,
}

#[derive(Clone, ValueEnum, PartialEq, Eq)]
pub enum BenchFamily {
    All,
    Micro,
    Macro,
    Sweeps,
}
