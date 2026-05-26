use anyhow::{Context, Result, bail};

use crate::aggregate::{AggregateOptions, run_aggregate};
use crate::bench_runner::{
    BenchRunOptions, BenchSelectionRequest, default_output_path, resolve_bench_invocations,
    run_benches,
};
use crate::cli::CiArgs;

pub fn run(args: CiArgs) -> Result<()> {
    if args.parallel && args.pin_core.is_some() {
        bail!(
            "--pin-core is not supported with --parallel; pinning a single core conflicts with parallel scheduling"
        );
    }

    let workspace_root = std::env::current_dir().context("failed to determine cwd")?;
    let target_dir = workspace_root.join("target");
    let benches = resolve_bench_invocations(&BenchSelectionRequest {
        suites: &args.suites,
        kinds: &args.kinds,
        families: &args.families,
        subcategories: &args.subcategories,
        targets: &args.targets,
        benchmark: args.benchmark.as_deref(),
        workload: args.workload.as_deref(),
        payload: args.payload.as_deref(),
        operation: args.operation.as_deref(),
        implementation: args.implementation.as_deref(),
        size: args.size,
        variant: args.variant.as_deref(),
    })?;

    run_benches(
        &benches,
        &BenchRunOptions {
            workspace_root: &workspace_root,
            smoke: false,
            pin_core: args.pin_core.as_deref(),
            target_dir: &target_dir,
            parallel: args.parallel,
            jobs: args.jobs,
            incremental: args.incremental,
        },
    )?;

    let output = args.output.unwrap_or_else(default_output_path);
    run_aggregate(&AggregateOptions {
        criterion_root: target_dir.join("criterion"),
        callgrind_root: target_dir.clone(),
        dhat_root: target_dir.join("dhat"),
        output,
    })
}
