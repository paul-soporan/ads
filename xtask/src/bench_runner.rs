use anyhow::{Context, Result, bail};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use owo_colors::OwoColorize;
use rayon::ThreadPoolBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{IsTerminal, Write, stdout};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, UNIX_EPOCH};
use walkdir::WalkDir;

use crate::cli::{BenchFamily, BenchKind as CliBenchKind, Suite};

const MEMORY_BENCHES: &[&str] = &[
    "micro_maps_dhat",
    "micro_sequences_dhat",
    "micro_heaps_dhat",
    "macro_read_heavy_dhat",
    "macro_write_heavy_dhat",
    "macro_thrashing_dhat",
    "sweep_btree_cache_dhat",
    "sweep_hash_collisions_dhat",
];
const CRITERION_BENCHES: &[&str] = &[
    "micro_maps_u64_criterion",
    "micro_maps_strings_criterion",
    "micro_maps_large_payload_criterion",
    "micro_sequences_criterion",
    "micro_heaps_criterion",
    "macro_read_heavy_criterion",
    "macro_write_heavy_criterion",
    "macro_thrashing_criterion",
    "sweep_btree_cache_criterion",
    "sweep_hash_collisions_criterion",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BenchKind {
    Criterion,
    Dhat,
    Callgrind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BenchFamilyGroup {
    Micro,
    Macro,
    Sweeps,
}

#[derive(Clone, Copy, Debug)]
struct BenchSpec {
    target: &'static str,
    kind: BenchKind,
    family: BenchFamilyGroup,
    subcategory: &'static str,
    criterion_groups: &'static [&'static str],
    workload_tags: &'static [&'static str],
}

const BENCH_SPECS: &[BenchSpec] = &[
    BenchSpec {
        target: "micro_maps_u64_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_maps",
        criterion_groups: &["micro_maps_u64"],
        workload_tags: &["micro_maps_u64"],
    },
    BenchSpec {
        target: "micro_maps_strings_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_maps",
        criterion_groups: &["micro_maps_strings"],
        workload_tags: &["micro_maps_strings"],
    },
    BenchSpec {
        target: "micro_maps_large_payload_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_maps",
        criterion_groups: &["micro_maps_large_payload"],
        workload_tags: &["micro_maps_large_payload"],
    },
    BenchSpec {
        target: "micro_sequences_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_sequences",
        criterion_groups: &["micro_sequences"],
        workload_tags: &["micro_sequences"],
    },
    BenchSpec {
        target: "micro_heaps_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_heaps",
        criterion_groups: &["micro_heaps_u64"],
        workload_tags: &["micro_heaps_u64"],
    },
    BenchSpec {
        target: "macro_read_heavy_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_read_heavy",
        criterion_groups: &["macro_read_heavy_u64"],
        workload_tags: &["macro_read_heavy_u64"],
    },
    BenchSpec {
        target: "macro_write_heavy_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_write_heavy",
        criterion_groups: &["macro_write_heavy_u64"],
        workload_tags: &["macro_write_heavy_u64"],
    },
    BenchSpec {
        target: "macro_thrashing_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_thrashing",
        criterion_groups: &["macro_thrashing_u64"],
        workload_tags: &["macro_thrashing_u64"],
    },
    BenchSpec {
        target: "sweep_btree_cache_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Sweeps,
        subcategory: "sweep_btree_cache",
        criterion_groups: &["sweep_btree_cache_u64"],
        workload_tags: &["sweep_btree_cache_u64"],
    },
    BenchSpec {
        target: "sweep_hash_collisions_criterion",
        kind: BenchKind::Criterion,
        family: BenchFamilyGroup::Sweeps,
        subcategory: "sweep_hash_collisions",
        criterion_groups: &["sweep_hash_collisions_u64"],
        workload_tags: &["sweep_hash_collisions_u64"],
    },
    BenchSpec {
        target: "micro_maps_dhat",
        kind: BenchKind::Dhat,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_maps",
        criterion_groups: &[],
        workload_tags: &[
            "micro_maps_u64",
            "micro_maps_strings",
            "micro_maps_large_payload",
        ],
    },
    BenchSpec {
        target: "micro_sequences_dhat",
        kind: BenchKind::Dhat,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_sequences",
        criterion_groups: &[],
        workload_tags: &["micro_sequences"],
    },
    BenchSpec {
        target: "micro_heaps_dhat",
        kind: BenchKind::Dhat,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_heaps",
        criterion_groups: &[],
        workload_tags: &["micro_heaps_u64"],
    },
    BenchSpec {
        target: "macro_read_heavy_dhat",
        kind: BenchKind::Dhat,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_read_heavy",
        criterion_groups: &[],
        workload_tags: &["macro_read_heavy_u64"],
    },
    BenchSpec {
        target: "macro_write_heavy_dhat",
        kind: BenchKind::Dhat,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_write_heavy",
        criterion_groups: &[],
        workload_tags: &["macro_write_heavy_u64"],
    },
    BenchSpec {
        target: "macro_thrashing_dhat",
        kind: BenchKind::Dhat,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_thrashing",
        criterion_groups: &[],
        workload_tags: &["macro_thrashing_u64"],
    },
    BenchSpec {
        target: "sweep_btree_cache_dhat",
        kind: BenchKind::Dhat,
        family: BenchFamilyGroup::Sweeps,
        subcategory: "sweep_btree_cache",
        criterion_groups: &[],
        workload_tags: &["sweep_btree_cache_u64"],
    },
    BenchSpec {
        target: "sweep_hash_collisions_dhat",
        kind: BenchKind::Dhat,
        family: BenchFamilyGroup::Sweeps,
        subcategory: "sweep_hash_collisions",
        criterion_groups: &[],
        workload_tags: &["sweep_hash_collisions_u64"],
    },
    BenchSpec {
        target: "micro_maps_callgrind",
        kind: BenchKind::Callgrind,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_maps",
        criterion_groups: &[],
        workload_tags: &[
            "micro_maps_u64",
            "micro_maps_strings",
            "micro_maps_large_payload",
        ],
    },
    BenchSpec {
        target: "micro_sequences_callgrind",
        kind: BenchKind::Callgrind,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_sequences",
        criterion_groups: &[],
        workload_tags: &["micro_sequences"],
    },
    BenchSpec {
        target: "micro_heaps_callgrind",
        kind: BenchKind::Callgrind,
        family: BenchFamilyGroup::Micro,
        subcategory: "micro_heaps",
        criterion_groups: &[],
        workload_tags: &["micro_heaps_u64"],
    },
    BenchSpec {
        target: "macro_read_heavy_callgrind",
        kind: BenchKind::Callgrind,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_read_heavy",
        criterion_groups: &[],
        workload_tags: &["macro_read_heavy_u64"],
    },
    BenchSpec {
        target: "macro_write_heavy_callgrind",
        kind: BenchKind::Callgrind,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_write_heavy",
        criterion_groups: &[],
        workload_tags: &["macro_write_heavy_u64"],
    },
    BenchSpec {
        target: "macro_thrashing_callgrind",
        kind: BenchKind::Callgrind,
        family: BenchFamilyGroup::Macro,
        subcategory: "macro_thrashing",
        criterion_groups: &[],
        workload_tags: &["macro_thrashing_u64"],
    },
    BenchSpec {
        target: "sweep_btree_cache_callgrind",
        kind: BenchKind::Callgrind,
        family: BenchFamilyGroup::Sweeps,
        subcategory: "sweep_btree_cache",
        criterion_groups: &[],
        workload_tags: &["sweep_btree_cache_u64"],
    },
    BenchSpec {
        target: "sweep_hash_collisions_callgrind",
        kind: BenchKind::Callgrind,
        family: BenchFamilyGroup::Sweeps,
        subcategory: "sweep_hash_collisions",
        criterion_groups: &[],
        workload_tags: &["sweep_hash_collisions_u64"],
    },
];

pub struct BenchSelectionRequest<'a> {
    pub suites: &'a [Suite],
    pub kinds: &'a [CliBenchKind],
    pub families: &'a [BenchFamily],
    pub subcategories: &'a [String],
    pub targets: &'a [String],
    pub benchmark: Option<&'a str>,
    pub workload: Option<&'a str>,
    pub payload: Option<&'a str>,
    pub operation: Option<&'a str>,
    pub implementation: Option<&'a str>,
    pub size: Option<usize>,
    pub variant: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct BenchInvocation {
    pub target: String,
    pub criterion_filter: Option<String>,
}

pub struct BenchRunOptions<'a> {
    pub workspace_root: &'a Path,
    pub smoke: bool,
    pub pin_core: Option<&'a str>,
    pub target_dir: &'a Path,
    pub parallel: bool,
    pub jobs: Option<usize>,
    pub incremental: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BenchStatus {
    Pending,
    Running,
    Done,
    Failed,
    Skipped,
}

#[derive(Clone, Debug)]
struct BenchProgress {
    bench: String,
    status: BenchStatus,
    started_at: Option<Instant>,
    finished_at: Option<Instant>,
}

#[derive(Debug)]
struct BenchFailure {
    bench: String,
    error: String,
    stderr: String,
}

#[derive(Debug)]
struct BenchOutcome {
    bench: String,
    success: bool,
    error: Option<String>,
    stderr: String,
}

#[derive(Serialize, Deserialize)]
struct BenchRunState {
    source_stamp_ns: u64,
    smoke: bool,
}

#[derive(Clone, Copy)]
enum OutputMode {
    Inherit,
    QuietCapture,
}

fn is_criterion_bench(bench: &str) -> bool {
    CRITERION_BENCHES.contains(&bench)
}

#[derive(Default, Clone, Debug)]
struct BenchmarkQuery {
    criterion_id: Option<String>,
    workload: Option<String>,
    payload: Option<String>,
    operation: Option<String>,
    implementation: Option<String>,
    size: Option<usize>,
    variant: Option<String>,
}

impl BenchmarkQuery {
    fn from_request(request: &BenchSelectionRequest<'_>) -> Result<Self> {
        let mut query = Self {
            criterion_id: request.benchmark.map(|s| s.to_string()),
            workload: request.workload.map(normalize_token),
            payload: request.payload.map(normalize_token),
            operation: request.operation.map(normalize_token),
            implementation: request.implementation.map(normalize_token),
            size: request.size,
            variant: request.variant.map(normalize_token),
        };

        if let Some(raw) = request.benchmark
            && raw.contains('=')
            && raw.contains('|')
        {
            query.criterion_id = None;
            for piece in raw.split('|') {
                let Some((key, value)) = piece.split_once('=') else {
                    continue;
                };
                let key = normalize_token(key);
                let value = normalize_token(value);
                match key.as_str() {
                    "workload" => query.workload = Some(value),
                    "payload" => query.payload = Some(value),
                    "op" | "operation" => query.operation = Some(value),
                    "impl" | "implementation" => query.implementation = Some(value),
                    "size" => query.size = value.parse::<usize>().ok(),
                    "variant" => query.variant = Some(value),
                    _ => {}
                }
            }
        }

        Ok(query)
    }

    fn has_granular_filter(&self) -> bool {
        self.criterion_id.is_some()
            || self.payload.is_some()
            || self.operation.is_some()
            || self.implementation.is_some()
            || self.size.is_some()
            || self.variant.is_some()
    }

    fn criterion_filter(&self) -> Option<String> {
        if let Some(id) = &self.criterion_id {
            return Some(id.clone());
        }

        let mut pieces = Vec::new();
        if let Some(workload) = &self.workload {
            pieces.push(workload.clone());
        }
        if let Some(op) = &self.operation {
            pieces.push(op.clone());
        }
        if let Some(implementation) = &self.implementation {
            pieces.push(implementation.clone());
        }
        if let Some(size) = self.size {
            pieces.push(size.to_string());
        }

        (!pieces.is_empty()).then_some(pieces.join("/"))
    }
}

fn normalize_token(value: &str) -> String {
    value.trim().to_lowercase()
}

fn infer_payload_from_workload(workload: &str) -> &'static str {
    if workload.contains("large_payload") {
        "large_payload"
    } else if workload.contains("string") {
        "string"
    } else {
        "u64"
    }
}

fn kind_matches(spec: &BenchSpec, kinds: &[CliBenchKind]) -> bool {
    if kinds.is_empty() {
        return true;
    }

    kinds.iter().any(|kind| {
        matches!(
            (kind, spec.kind),
            (CliBenchKind::Criterion, BenchKind::Criterion)
                | (CliBenchKind::Dhat, BenchKind::Dhat)
                | (CliBenchKind::Callgrind, BenchKind::Callgrind)
        )
    })
}

fn family_matches(spec: &BenchSpec, families: &[BenchFamily]) -> bool {
    if families.is_empty() || families.iter().any(|f| matches!(f, BenchFamily::All)) {
        return true;
    }

    families.iter().any(|family| {
        matches!(
            (family, spec.family),
            (BenchFamily::Micro, BenchFamilyGroup::Micro)
                | (BenchFamily::Macro, BenchFamilyGroup::Macro)
                | (BenchFamily::Sweeps, BenchFamilyGroup::Sweeps)
        )
    })
}

fn suite_matches(spec: &BenchSpec, suites: &[Suite]) -> bool {
    if suites.is_empty() || suites.iter().any(|s| matches!(s, Suite::All)) {
        return true;
    }

    suites.iter().any(|suite| {
        matches!(
            (suite, spec.family, spec.kind),
            (Suite::Micro, BenchFamilyGroup::Micro, _)
                | (Suite::Macro, BenchFamilyGroup::Macro, _)
                | (Suite::Sweeps, BenchFamilyGroup::Sweeps, _)
                | (Suite::Memory, _, BenchKind::Dhat)
                | (Suite::Callgrind, _, BenchKind::Callgrind)
        )
    })
}

pub fn resolve_bench_invocations(
    request: &BenchSelectionRequest<'_>,
) -> Result<Vec<BenchInvocation>> {
    let mut specs: Vec<BenchSpec> = BENCH_SPECS
        .iter()
        .copied()
        .filter(|spec| suite_matches(spec, request.suites))
        .filter(|spec| kind_matches(spec, request.kinds))
        .filter(|spec| family_matches(spec, request.families))
        .collect();

    if !request.subcategories.is_empty() {
        let wanted: Vec<String> = request
            .subcategories
            .iter()
            .map(|s| normalize_token(s))
            .collect();
        specs.retain(|spec| wanted.iter().any(|s| s == spec.subcategory));
    }

    if !request.targets.is_empty() {
        let wanted: Vec<String> = request.targets.iter().map(|s| normalize_token(s)).collect();
        specs.retain(|spec| wanted.iter().any(|t| t == spec.target));
    }

    let query = BenchmarkQuery::from_request(request)?;

    if let Some(workload) = &query.workload {
        specs.retain(|spec| spec.workload_tags.iter().any(|tag| tag == workload));
    }

    if let Some(payload) = &query.payload {
        specs.retain(|spec| {
            spec.workload_tags
                .iter()
                .any(|workload| infer_payload_from_workload(workload) == payload)
        });
    }

    if query.has_granular_filter() {
        specs.retain(|spec| matches!(spec.kind, BenchKind::Criterion));
    }

    let criterion_filter = query.criterion_filter();

    let mut invocations = specs
        .into_iter()
        .map(|spec| BenchInvocation {
            target: spec.target.to_string(),
            criterion_filter: matches!(spec.kind, BenchKind::Criterion)
                .then(|| criterion_filter.clone())
                .flatten(),
        })
        .collect::<Vec<_>>();

    invocations.sort_by(|a, b| a.target.cmp(&b.target));
    invocations.dedup_by(|a, b| {
        a.target == b.target && a.criterion_filter.as_deref() == b.criterion_filter.as_deref()
    });

    if invocations.is_empty() {
        bail!(
            "no benchmark targets matched the provided filters (--suite/--kind/--family/--subcategory/--target/--benchmark)"
        );
    }

    Ok(invocations)
}

fn invocation_label(invocation: &BenchInvocation) -> String {
    if let Some(filter) = &invocation.criterion_filter {
        format!("{} [{}]", invocation.target, filter)
    } else {
        invocation.target.clone()
    }
}

fn truncate_with_ellipsis(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let len = value.chars().count();
    if len <= max_chars {
        return value.to_string();
    }

    if max_chars == 1 {
        return "…".to_string();
    }

    let mut out = String::with_capacity(max_chars);
    for ch in value.chars().take(max_chars - 1) {
        out.push(ch);
    }
    out.push('…');
    out
}

pub fn run_benches(benches: &[BenchInvocation], options: &BenchRunOptions<'_>) -> Result<()> {
    if benches.is_empty() {
        bail!("no benchmark targets were selected");
    }

    if let Some(jobs) = options.jobs
        && jobs == 0
    {
        bail!("--jobs must be at least 1");
    }

    let source_stamp_ns = compute_ads_source_stamp_ns(options.workspace_root)?;
    let selected = select_benches_for_run(benches, options, source_stamp_ns)?;

    if selected.is_empty() {
        println!(
            "{}",
            "all selected benchmark results are up-to-date; nothing to run"
                .green()
                .bold()
        );
        return Ok(());
    }

    if !options.parallel {
        for bench in &selected {
            let outcome = run_bench(
                bench,
                options.smoke,
                options.pin_core,
                options.target_dir,
                OutputMode::Inherit,
            )
            .with_context(|| format!("failed running benchmark {}", invocation_label(bench)))?;

            if !outcome.success {
                let mut message = format!("benchmark failed: {}", invocation_label(bench));
                if !outcome.stderr.trim().is_empty() {
                    message.push_str("\n\nCaptured stderr:\n");
                    message.push_str(&outcome.stderr);
                }
                bail!(message);
            }

            persist_bench_state(options.target_dir, bench, source_stamp_ns, options.smoke)?;
        }

        return Ok(());
    }

    let progress = Arc::new(Mutex::new(initialize_progress(benches, &selected)));
    let shutdown = Arc::new(AtomicBool::new(false));
    let render_thread = spawn_progress_renderer(progress.clone(), shutdown.clone());

    let pool = if let Some(jobs) = options.jobs {
        ThreadPoolBuilder::new().num_threads(jobs).build()?
    } else {
        ThreadPoolBuilder::new().build()?
    };

    let outcomes = pool.install(|| {
        use rayon::prelude::*;

        selected
            .par_iter()
            .map(|bench| {
                set_status(&progress, &invocation_label(bench), BenchStatus::Running);

                let outcome = match run_bench(
                    bench,
                    options.smoke,
                    options.pin_core,
                    options.target_dir,
                    OutputMode::QuietCapture,
                ) {
                    Ok(ok) => ok,
                    Err(error) => BenchOutcome {
                        bench: invocation_label(bench),
                        success: false,
                        error: Some(format!("{error:#}")),
                        stderr: String::new(),
                    },
                };

                if outcome.success {
                    set_status(&progress, &invocation_label(bench), BenchStatus::Done);
                    let _ = persist_bench_state(
                        options.target_dir,
                        bench,
                        source_stamp_ns,
                        options.smoke,
                    );
                } else {
                    set_status(&progress, &invocation_label(bench), BenchStatus::Failed);
                }

                outcome
            })
            .collect::<Vec<_>>()
    });

    shutdown.store(true, Ordering::SeqCst);
    if let Some(handle) = render_thread {
        let _ = handle.join();
    }

    if let Ok(snapshot) = progress.lock() {
        print_parallel_final_statuses(&snapshot);
    }

    let failures: Vec<BenchFailure> = outcomes
        .into_iter()
        .filter(|o| !o.success)
        .map(|o| BenchFailure {
            bench: o.bench,
            error: o
                .error
                .unwrap_or_else(|| "benchmark process failed".to_string()),
            stderr: o.stderr,
        })
        .collect();

    print_parallel_fail_summary(&failures);

    if failures.is_empty() {
        return Ok(());
    }

    let mut summary = String::from("parallel benchmark run failed:\n");
    for failure in failures {
        summary.push_str(&format!("  - {}: {}\n", failure.bench, failure.error));
    }
    bail!("{}", summary.trim_end());
}

fn initialize_progress(
    all: &[BenchInvocation],
    selected: &[BenchInvocation],
) -> Vec<BenchProgress> {
    let selected_labels: Vec<String> = selected.iter().map(invocation_label).collect();
    all.iter()
        .map(|bench| {
            let label = invocation_label(bench);
            BenchProgress {
                bench: label.clone(),
                status: if selected_labels.contains(&label) {
                    BenchStatus::Pending
                } else {
                    BenchStatus::Skipped
                },
                started_at: None,
                finished_at: None,
            }
        })
        .collect()
}

fn set_status(progress: &Arc<Mutex<Vec<BenchProgress>>>, bench: &str, status: BenchStatus) {
    let now = Instant::now();
    if let Ok(mut rows) = progress.lock()
        && let Some(row) = rows.iter_mut().find(|row| row.bench == bench)
    {
        row.status = status;
        match status {
            BenchStatus::Running => {
                row.started_at = Some(now);
                row.finished_at = None;
            }
            BenchStatus::Done | BenchStatus::Failed | BenchStatus::Skipped => {
                if row.started_at.is_none() {
                    row.started_at = Some(now);
                }
                row.finished_at = Some(now);
            }
            BenchStatus::Pending => {}
        }
    }
}

fn spawn_progress_renderer(
    progress: Arc<Mutex<Vec<BenchProgress>>>,
    shutdown: Arc<AtomicBool>,
) -> Option<thread::JoinHandle<()>> {
    if !stdout().is_terminal() {
        return None;
    }

    Some(thread::spawn(move || {
        let mut out = stdout();
        // Alternate screen keeps redraws stable and avoids scrollback glitches.
        if execute!(out, crossterm::terminal::EnterAlternateScreen, Hide).is_err() {
            return;
        }

        struct ScreenGuard;
        impl Drop for ScreenGuard {
            fn drop(&mut self) {
                let mut out = stdout();
                let _ = execute!(out, Show, crossterm::terminal::LeaveAlternateScreen);
            }
        }
        let _guard = ScreenGuard;

        let mut page: usize = 0;
        let mut tick: usize = 0;

        loop {
            let snapshot = match progress.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => return,
            };

            if render_progress_table(&snapshot, page).is_err() {
                return;
            }

            let (_, height) = crossterm::terminal::size().unwrap_or((120, 40));
            let header_lines = 4usize;
            let footer_lines = 2usize;
            let rows_capacity = (height as usize)
                .saturating_sub(header_lines + footer_lines)
                .max(1);
            let page_count = snapshot.len().div_ceil(rows_capacity).max(1);

            // Rotate pages every ~4s so all benches become visible without resizing.
            if page_count > 1 && tick % 8 == 7 {
                page = (page + 1) % page_count;
            }
            tick = tick.wrapping_add(1);

            if shutdown.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(Duration::from_millis(500));
        }
    }))
}

fn render_progress_table(snapshot: &[BenchProgress], page: usize) -> Result<()> {
    let mut out = stdout();

    let (width, height) = crossterm::terminal::size().unwrap_or((120, 40));
    let header_lines = 4usize;
    let footer_lines = 2usize;
    let max_rows = height as usize;
    let rows_capacity = max_rows.saturating_sub(header_lines + footer_lines).max(1);

    // Keep row rendering inside the viewport width to prevent terminal scroll glitches.
    let bench_width = (width as usize).saturating_sub(24).clamp(12, 72);

    execute!(out, MoveTo(0, 0), Clear(ClearType::All))?;
    out.write_all(format!("{}\n", "Parallel Bench Progress".bold()).as_bytes())?;
    out.write_all(b"-----------------------------------------------------------\n")?;

    let now = Instant::now();
    let mut rows: Vec<&BenchProgress> = snapshot.iter().collect();
    rows.sort_by_key(|row| match row.status {
        BenchStatus::Running => 0,
        BenchStatus::Pending => 1,
        BenchStatus::Failed => 2,
        BenchStatus::Done => 3,
        BenchStatus::Skipped => 4,
    });

    let page_count = rows.len().div_ceil(rows_capacity).max(1);
    let current_page = page.min(page_count - 1);
    let start = current_page * rows_capacity;
    let shown = rows.iter().skip(start).take(rows_capacity);
    for row in shown {
        let status = match row.status {
            BenchStatus::Pending => "PENDING".yellow().to_string(),
            BenchStatus::Running => "RUNNING".cyan().bold().to_string(),
            BenchStatus::Done => "DONE".green().bold().to_string(),
            BenchStatus::Failed => "FAILED".red().bold().to_string(),
            BenchStatus::Skipped => "SKIPPED".magenta().to_string(),
        };

        let elapsed = match (row.started_at, row.finished_at) {
            (Some(start), Some(end)) => format_duration(end.saturating_duration_since(start)),
            (Some(start), None) => format_duration(now.saturating_duration_since(start)),
            _ => "0s".to_string(),
        };

        let bench = truncate_with_ellipsis(&row.bench, bench_width);
        out.write_all(format!("{status:>10}  {bench:<bench_width$} {elapsed:>8}\n").as_bytes())?;
    }

    if page_count > 1 {
        out.write_all(
            format!(
                "showing {}-{} of {} (page {}/{}) | auto-rotating pages\n",
                start + 1,
                (start + rows_capacity).min(rows.len()),
                rows.len(),
                current_page + 1,
                page_count
            )
            .as_bytes(),
        )?;
    }

    let done = snapshot
        .iter()
        .filter(|row| {
            matches!(
                row.status,
                BenchStatus::Done | BenchStatus::Failed | BenchStatus::Skipped
            )
        })
        .count();
    out.write_all(format!("\ncompleted: {done}/{}\n", snapshot.len()).as_bytes())?;

    out.flush()?;
    Ok(())
}

fn print_parallel_final_statuses(snapshot: &[BenchProgress]) {
    println!();
    println!("{}", "Final Parallel Bench Status".bold());
    println!("{}", "==========================".bold());

    let mut rows: Vec<&BenchProgress> = snapshot.iter().collect();
    rows.sort_by_key(|row| match row.status {
        BenchStatus::Failed => 0,
        BenchStatus::Running => 1,
        BenchStatus::Pending => 2,
        BenchStatus::Done => 3,
        BenchStatus::Skipped => 4,
    });

    for row in rows {
        let status = match row.status {
            BenchStatus::Pending => "PENDING".yellow().to_string(),
            BenchStatus::Running => "RUNNING".cyan().bold().to_string(),
            BenchStatus::Done => "DONE".green().bold().to_string(),
            BenchStatus::Failed => "FAILED".red().bold().to_string(),
            BenchStatus::Skipped => "SKIPPED".magenta().to_string(),
        };

        let elapsed = match (row.started_at, row.finished_at) {
            (Some(start), Some(end)) => format_duration(end.saturating_duration_since(start)),
            (Some(start), None) => format_duration(Instant::now().saturating_duration_since(start)),
            _ => "0s".to_string(),
        };

        // Intentionally untruncated for final reporting.
        println!("{status:>10}  {}  {elapsed:>8}", row.bench);
    }
}

fn print_parallel_fail_summary(failures: &[BenchFailure]) {
    if failures.is_empty() {
        return;
    }

    println!();
    println!("{}", "Failed Benchmarks (Captured stderr)".red().bold());
    println!("{}", "==================================".red());

    for failure in failures {
        println!(
            "{} {}",
            "bench:".yellow().bold(),
            failure.bench.cyan().bold()
        );
        println!("{} {}", "error:".yellow().bold(), failure.error);
        if failure.stderr.trim().is_empty() {
            println!("{}", "stderr: <empty>".dimmed());
        } else {
            println!("{}", "stderr:".yellow().bold());
            println!("{}", failure.stderr.trim_end());
        }
        println!();
    }
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();

    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}h {minutes:02}m {seconds:02}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

fn select_benches_for_run(
    benches: &[BenchInvocation],
    options: &BenchRunOptions<'_>,
    source_stamp_ns: u64,
) -> Result<Vec<BenchInvocation>> {
    if !options.incremental {
        return Ok(benches.to_vec());
    }

    let mut selected = Vec::with_capacity(benches.len());
    for bench in benches {
        if should_run_bench(bench, options, source_stamp_ns)? {
            selected.push(bench.clone());
        }
    }
    Ok(selected)
}

fn should_run_bench(
    bench: &BenchInvocation,
    options: &BenchRunOptions<'_>,
    source_stamp_ns: u64,
) -> Result<bool> {
    if bench.criterion_filter.is_some() {
        // Filtered criterion invocations are intentionally partial; do not skip them.
        return Ok(true);
    }

    if !bench_artifacts_exist(options.target_dir, bench)? {
        return Ok(true);
    }

    let state = load_bench_state(options.target_dir, bench)?;
    let Some(state) = state else {
        return Ok(true);
    };

    if state.source_stamp_ns != source_stamp_ns {
        return Ok(true);
    }

    if state.smoke != options.smoke {
        return Ok(true);
    }

    Ok(false)
}

fn bench_artifacts_exist(target_dir: &Path, bench: &BenchInvocation) -> Result<bool> {
    let Some(spec) = BENCH_SPECS.iter().find(|spec| spec.target == bench.target) else {
        return Ok(false);
    };

    match spec.kind {
        BenchKind::Criterion => criterion_groups_complete(target_dir, spec.criterion_groups),
        BenchKind::Dhat => dhat_artifact_exists(target_dir),
        BenchKind::Callgrind => callgrind_artifact_exists(target_dir),
    }
}

fn criterion_groups_complete(target_dir: &Path, groups: &[&str]) -> Result<bool> {
    let criterion_root = target_dir.join("criterion");
    if !criterion_root.exists() {
        return Ok(false);
    }

    for group in groups {
        let group_dir = criterion_root.join(group);
        if !group_dir.exists() {
            return Ok(false);
        }

        let mut found_estimates = false;
        for entry in WalkDir::new(&group_dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() && entry.file_name() == "estimates.json" {
                found_estimates = true;
                break;
            }
        }

        if !found_estimates {
            return Ok(false);
        }
    }

    Ok(true)
}

fn callgrind_artifact_exists(target_dir: &Path) -> Result<bool> {
    if !target_dir.exists() {
        return Ok(false);
    }

    for entry in WalkDir::new(target_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        if let Some(name) = entry.file_name().to_str() {
            let is_callgrind_file = name.starts_with("callgrind.out")
                || (name.starts_with("callgrind.") && name.ends_with(".out"));
            if is_callgrind_file {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn dhat_artifact_exists(target_dir: &Path) -> Result<bool> {
    let dhat_dir = target_dir.join("dhat");
    if !dhat_dir.exists() {
        return Ok(false);
    }

    for entry in WalkDir::new(dhat_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        if entry.path().extension().and_then(|x| x.to_str()) == Some("json") {
            return Ok(true);
        }
    }

    Ok(false)
}

fn compute_ads_source_stamp_ns(workspace_root: &Path) -> Result<u64> {
    let mut newest: u64 = 0;
    for root in [
        workspace_root.join("packages/ads/src"),
        workspace_root.join("packages/ads/benches"),
    ] {
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            let metadata = entry.metadata()?;
            let modified = metadata
                .modified()
                .with_context(|| format!("failed to read mtime for {}", entry.path().display()))?;
            let nanos = modified
                .duration_since(UNIX_EPOCH)
                .with_context(|| format!("invalid mtime for {}", entry.path().display()))?
                .as_nanos() as u64;
            newest = newest.max(nanos);
        }
    }
    Ok(newest)
}

fn state_dir(target_dir: &Path) -> PathBuf {
    target_dir.join("xtask-bench-state")
}

fn state_key(bench: &BenchInvocation) -> String {
    if let Some(filter) = &bench.criterion_filter {
        let normalized = filter
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                    ch
                } else {
                    '_'
                }
            })
            .collect::<String>();
        format!("{}__{normalized}", bench.target)
    } else {
        bench.target.clone()
    }
}

fn state_file(target_dir: &Path, bench: &BenchInvocation) -> PathBuf {
    state_dir(target_dir).join(format!("{}.json", state_key(bench)))
}

fn load_bench_state(target_dir: &Path, bench: &BenchInvocation) -> Result<Option<BenchRunState>> {
    let path = state_file(target_dir, bench);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path)?;
    let state = serde_json::from_slice::<BenchRunState>(&bytes)?;
    Ok(Some(state))
}

fn persist_bench_state(
    target_dir: &Path,
    bench: &BenchInvocation,
    source_stamp_ns: u64,
    smoke: bool,
) -> Result<()> {
    let dir = state_dir(target_dir);
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = state_file(target_dir, bench);
    let payload = BenchRunState {
        source_stamp_ns,
        smoke,
    };
    fs::write(path, serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}

fn run_bench(
    bench: &BenchInvocation,
    smoke: bool,
    pin_core: Option<&str>,
    target_dir: &Path,
    output_mode: OutputMode,
) -> Result<BenchOutcome> {
    let target = bench.target.as_str();
    let mut args: Vec<String> = vec![
        "bench".to_string(),
        "-p".to_string(),
        "ads".to_string(),
        "--bench".to_string(),
        target.to_string(),
    ];

    let mut has_criterion_args = false;
    if smoke && is_criterion_bench(target) {
        has_criterion_args = true;
        args.extend([
            "--".to_string(),
            "--warm-up-time".to_string(),
            "0.01".to_string(),
            "--measurement-time".to_string(),
            "0.01".to_string(),
            "--sample-size".to_string(),
            "10".to_string(),
        ]);
    }

    if let Some(filter) = &bench.criterion_filter {
        if !has_criterion_args {
            args.push("--".to_string());
        }
        args.push(filter.clone());
    }

    let mut envs: Vec<(String, String)> = vec![(
        "CARGO_TARGET_DIR".to_string(),
        target_dir.display().to_string(),
    )];

    if MEMORY_BENCHES.contains(&target) {
        let dhat_dir = target_dir.join("dhat");
        fs::create_dir_all(&dhat_dir).context("failed to create target/dhat directory")?;
        envs.push(("ADS_DHAT_DIR".to_string(), dhat_dir.display().to_string()));
    }

    if let Some(core) = pin_core {
        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new("taskset");
            cmd.arg("-c").arg(core).arg("cargo").args(&args);
            for (k, v) in &envs {
                cmd.env(k, v);
            }
            return execute_bench_command(cmd, bench, output_mode).with_context(|| {
                format!(
                    "failed running benchmark {} with taskset",
                    invocation_label(bench)
                )
            });
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = core;
            bail!("--pin-core is only supported on Linux");
        }
    } else {
        let mut cmd = Command::new("cargo");
        cmd.args(&args);
        for (k, v) in &envs {
            cmd.env(k, v);
        }
        return execute_bench_command(cmd, bench, output_mode)
            .with_context(|| format!("failed running benchmark {}", invocation_label(bench)));
    }

    #[allow(unreachable_code)]
    Ok(BenchOutcome {
        bench: invocation_label(bench),
        success: true,
        error: None,
        stderr: String::new(),
    })
}

fn execute_bench_command(
    mut cmd: Command,
    bench: &BenchInvocation,
    output_mode: OutputMode,
) -> Result<BenchOutcome> {
    match output_mode {
        OutputMode::Inherit => {
            cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
            let status = cmd.status()?;
            if status.success() {
                Ok(BenchOutcome {
                    bench: invocation_label(bench),
                    success: true,
                    error: None,
                    stderr: String::new(),
                })
            } else {
                Ok(BenchOutcome {
                    bench: invocation_label(bench),
                    success: false,
                    error: Some(format!("process exited with status {status}")),
                    stderr: String::new(),
                })
            }
        }
        OutputMode::QuietCapture => {
            cmd.stdout(Stdio::null()).stderr(Stdio::piped());
            let output = cmd.output()?;
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                Ok(BenchOutcome {
                    bench: invocation_label(bench),
                    success: true,
                    error: None,
                    stderr: String::new(),
                })
            } else {
                Ok(BenchOutcome {
                    bench: invocation_label(bench),
                    success: false,
                    error: Some(format!("process exited with status {}", output.status)),
                    stderr,
                })
            }
        }
    }
}

pub fn default_output_path() -> PathBuf {
    let frontend_public = Path::new("frontend/public");
    if frontend_public.exists() {
        frontend_public.join("aggregated_benchmarks.json")
    } else {
        PathBuf::from("aggregated_benchmarks.json")
    }
}
