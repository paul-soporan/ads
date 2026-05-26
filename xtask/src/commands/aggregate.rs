use anyhow::Result;

use crate::aggregate::{AggregateOptions, run_aggregate};
use crate::bench_runner::default_output_path;
use crate::cli::AggregateArgs;

pub fn run(args: AggregateArgs) -> Result<()> {
    let output = args.output.unwrap_or_else(default_output_path);
    run_aggregate(&AggregateOptions {
        criterion_root: args.criterion_root,
        callgrind_root: args.callgrind_root,
        dhat_root: args.dhat_root,
        output,
    })
}
