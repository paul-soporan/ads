use anyhow::{Context, Result, bail};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::aggregate::{AggregateOptions, run_aggregate};
use crate::bench_runner::{
    BenchRunOptions, BenchSelectionRequest, default_output_path, resolve_bench_invocations,
    run_benches,
};
use crate::cli::BenchArgs;

pub fn run(args: BenchArgs) -> Result<()> {
    if args.parallel && args.pin_core.is_some() {
        bail!(
            "--pin-core is not supported with --parallel; pinning a single core conflicts with parallel scheduling"
        );
    }

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

    let workspace_root = std::env::current_dir().context("failed to determine cwd")?;
    let target_dir = if args.no_artifacts {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("failed to read system clock")?
            .as_secs();
        workspace_root.join(format!("target/xtask-temp-{stamp}"))
    } else {
        workspace_root.join("target")
    };

    run_benches(
        &benches,
        &BenchRunOptions {
            workspace_root: &workspace_root,
            smoke: args.smoke,
            pin_core: args.pin_core.as_deref(),
            target_dir: &target_dir,
            parallel: args.parallel,
            jobs: args.jobs,
            incremental: args.incremental,
        },
    )?;

    if args.aggregate || args.output.is_some() {
        let output = args.output.unwrap_or_else(default_output_path);
        run_aggregate(&AggregateOptions {
            criterion_root: target_dir.join("criterion"),
            callgrind_root: target_dir.clone(),
            dhat_root: target_dir.join("dhat"),
            output,
        })?;
    }

    if args.no_artifacts {
        fs::remove_dir_all(&target_dir).with_context(|| {
            format!(
                "failed to clean temporary artifact directory {}",
                target_dir.display()
            )
        })?;
    }

    Ok(())
}
