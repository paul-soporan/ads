use anyhow::{Context, Result, anyhow};
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub struct AggregateOptions {
    pub criterion_root: PathBuf,
    pub callgrind_root: PathBuf,
    pub dhat_root: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Serialize)]
struct AggregatedBenchmarks {
    generated_at_unix_secs: u64,
    operation_count: usize,
    operations: Vec<OperationGroup>,
}

#[derive(Debug, Serialize, Clone)]
struct JoinKeys {
    workload: String,
    payload: String,
    operation: String,
    implementation: String,
    size: usize,
    variant: String,
    join_key: String,
}

#[derive(Debug, Serialize)]
struct OperationGroup {
    join: JoinKeys,
    criterion: Vec<CriterionMeasurement>,
    callgrind: Vec<CallgrindMeasurement>,
    dhat: Vec<DhatMeasurement>,
}

#[derive(Debug, Serialize)]
struct CriterionMeasurement {
    path: String,
    group: String,
    function: String,
    sample: String,
    mean: Option<EstimateStats>,
    median: Option<EstimateStats>,
    slope: Option<EstimateStats>,
    std_dev: Option<EstimateStats>,
    throughput_elements: Option<u64>,
}

#[derive(Debug, Serialize)]
struct EstimateStats {
    point_estimate: f64,
    standard_error: Option<f64>,
    confidence_interval: Option<ConfidenceInterval>,
}

#[derive(Debug, Serialize)]
struct ConfidenceInterval {
    confidence_level: Option<f64>,
    lower_bound: f64,
    upper_bound: f64,
}

#[derive(Debug, Serialize)]
struct CallgrindMeasurement {
    path: String,
    events: Vec<String>,
    metrics: HashMap<String, u64>,
}

#[derive(Debug, Serialize)]
struct DhatMeasurement {
    path: String,
    total_bytes: Option<u64>,
    total_blocks: Option<u64>,
    max_bytes: Option<u64>,
    max_blocks: Option<u64>,
    extra_numeric_fields: HashMap<String, u64>,
}

#[derive(Debug)]
struct CriterionRecord {
    join: JoinKeys,
    measurement: CriterionMeasurement,
}

#[derive(Debug)]
struct CallgrindRecord {
    join: Option<JoinKeys>,
    measurement: CallgrindMeasurement,
}

#[derive(Debug)]
struct DhatRecord {
    join: Option<JoinKeys>,
    measurement: DhatMeasurement,
}

#[derive(Default)]
struct OperationBuilder {
    join: Option<JoinKeys>,
    criterion: Vec<CriterionMeasurement>,
    callgrind: Vec<CallgrindMeasurement>,
    dhat: Vec<DhatMeasurement>,
}

pub fn run_aggregate(opts: &AggregateOptions) -> Result<()> {
    let criterion = collect_criterion(&opts.criterion_root)?;
    let callgrind = collect_callgrind(&opts.callgrind_root)?;
    let dhat = collect_dhat(&opts.dhat_root)?;

    let mut by_join: BTreeMap<String, OperationBuilder> = BTreeMap::new();

    for row in criterion {
        let key = row.join.join_key.clone();
        let entry = by_join.entry(key).or_default();
        if entry.join.is_none() {
            entry.join = Some(row.join.clone());
        }
        entry.criterion.push(row.measurement);
    }

    for row in callgrind {
        if let Some(join) = row.join {
            let key = join.join_key.clone();
            let entry = by_join.entry(key).or_default();
            if entry.join.is_none() {
                entry.join = Some(join);
            }
            entry.callgrind.push(row.measurement);
        }
    }

    for row in dhat {
        if let Some(join) = row.join {
            let key = join.join_key.clone();
            let entry = by_join.entry(key).or_default();
            if entry.join.is_none() {
                entry.join = Some(join);
            }
            entry.dhat.push(row.measurement);
        }
    }

    let mut operations = Vec::new();
    for (_, builder) in by_join {
        if let Some(join) = builder.join {
            operations.push(OperationGroup {
                join,
                criterion: builder.criterion,
                callgrind: builder.callgrind,
                dhat: builder.dhat,
            });
        }
    }

    operations.sort_by(|a, b| a.join.join_key.cmp(&b.join.join_key));
    ensure_strict_one_to_one_to_one(&operations)?;

    let payload = AggregatedBenchmarks {
        generated_at_unix_secs: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock error")?
            .as_secs(),
        operation_count: operations.len(),
        operations,
    };

    if let Some(parent) = opts.output.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    fs::write(&opts.output, serde_json::to_vec_pretty(&payload)?)?;
    println!("wrote {}", opts.output.display());
    Ok(())
}

fn ensure_strict_one_to_one_to_one(operations: &[OperationGroup]) -> Result<()> {
    let mut violations = Vec::new();

    for operation in operations {
        let criterion_count = operation.criterion.len();
        let callgrind_count = operation.callgrind.len();
        let dhat_count = operation.dhat.len();

        if criterion_count != 1 || callgrind_count != 1 || dhat_count != 1 {
            violations.push((
                operation.join.join_key.clone(),
                criterion_count,
                callgrind_count,
                dhat_count,
            ));
        }
    }

    if violations.is_empty() {
        return Ok(());
    }

    let mut preview = String::new();
    for (join_key, criterion_count, callgrind_count, dhat_count) in violations.iter().take(20) {
        preview.push_str(&format!(
            "\n- {join_key}: criterion={criterion_count}, callgrind={callgrind_count}, dhat={dhat_count}"
        ));
    }

    if violations.len() > 20 {
        preview.push_str(&format!(
            "\n- ... and {} more",
            violations.len() - 20
        ));
    }

    Err(anyhow!(
        "aggregation requires strict 1:1:1 mapping (criterion/callgrind/dhat). \
found {} invalid operation groups:{}",
        violations.len(),
        preview
    ))
}

fn collect_criterion(root: &Path) -> Result<Vec<CriterionRecord>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_name() == "estimates.json" {
            let path = entry.into_path();
            if let Some(sample_dir) = path.parent().and_then(|p| p.file_name())
                && sample_dir == "new"
            {
                files.push(path);
            }
        }
    }

    let path_re = Regex::new(
        r"^(?P<group>.+)/(?P<function>[^/]+)/(?P<size>[^/]+)/(?P<sample>new)/estimates\.json$",
    )
    .context("failed to compile criterion path regex")?;

    let mut rows: Vec<CriterionRecord> = files
        .par_iter()
        .filter_map(|path| parse_criterion_file(root, path, &path_re).ok())
        .collect();

    rows.sort_by(|a, b| a.join.join_key.cmp(&b.join.join_key));
    Ok(rows)
}

fn parse_criterion_file(root: &Path, path: &Path, path_re: &Regex) -> Result<CriterionRecord> {
    let rel = path
        .strip_prefix(root)?
        .to_string_lossy()
        .replace('\\', "/");
    let caps = path_re
        .captures(&rel)
        .ok_or_else(|| anyhow!("unexpected criterion path format: {rel}"))?;

    let group = caps
        .name("group")
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let function = caps
        .name("function")
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let size_raw = caps
        .name("size")
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let sample = caps
        .name("sample")
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();

    let size = size_raw
        .parse::<usize>()
        .with_context(|| format!("expected numeric size in criterion path, got: {size_raw}"))?;

    let (operation, implementation) = split_function_taxonomy(&function);
    let workload = group.to_string();
    let payload = infer_payload(&group, &function);
    let variant = infer_variant(&implementation);
    let join = build_join_keys(
        &workload,
        &payload,
        &operation,
        &implementation,
        size,
        &variant,
    );

    let estimates: Value = serde_json::from_slice(&fs::read(path)?)?;
    let benchmark_json = path
        .parent()
        .map(|p| p.join("benchmark.json"))
        .unwrap_or_else(|| PathBuf::from("benchmark.json"));

    let throughput_elements = if benchmark_json.exists() {
        parse_throughput_elements(&benchmark_json)
    } else {
        None
    };

    Ok(CriterionRecord {
        join,
        measurement: CriterionMeasurement {
            path: rel,
            group,
            function,
            sample,
            mean: extract_estimate(&estimates, "mean"),
            median: extract_estimate(&estimates, "median"),
            slope: extract_estimate(&estimates, "slope"),
            std_dev: extract_estimate(&estimates, "std_dev"),
            throughput_elements,
        },
    })
}

fn parse_throughput_elements(path: &Path) -> Option<u64> {
    let json: Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    let throughput = json.get("throughput")?;

    if let Some(elements) = throughput.get("Elements") {
        return elements
            .as_u64()
            .or_else(|| elements.as_f64().map(|f| f as u64));
    }

    throughput.as_u64()
}

fn split_function_taxonomy(function: &str) -> (String, String) {
    const PREFIXES: &[&str] = &[
        "contains_temporal",
        "contains_zipf",
        "contains_mixed",
        "contains",
        "push_pop",
        "bulk_insert",
        "insert",
        "remove",
        "mix",
        "thrash",
    ];

    for prefix in PREFIXES {
        if let Some(rest) = function.strip_prefix(prefix)
            && let Some(implementation) = rest.strip_prefix('_')
        {
            return ((*prefix).to_string(), implementation.to_string());
        }
    }

    ("unknown".to_string(), function.to_string())
}

fn extract_estimate(estimates: &Value, key: &str) -> Option<EstimateStats> {
    let obj = estimates.get(key)?;
    let point_estimate = obj.get("point_estimate")?.as_f64()?;
    let standard_error = obj.get("standard_error").and_then(Value::as_f64);

    let confidence_interval = obj.get("confidence_interval").and_then(|ci| {
        let lower_bound = ci.get("lower_bound")?.as_f64()?;
        let upper_bound = ci.get("upper_bound")?.as_f64()?;
        let confidence_level = ci.get("confidence_level").and_then(Value::as_f64);
        Some(ConfidenceInterval {
            confidence_level,
            lower_bound,
            upper_bound,
        })
    });

    Some(EstimateStats {
        point_estimate,
        standard_error,
        confidence_interval,
    })
}

fn collect_callgrind(root: &Path) -> Result<Vec<CallgrindRecord>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".old") {
                continue;
            }
            let is_callgrind_file = name.starts_with("callgrind.out")
                || (name.starts_with("callgrind.")
                    && (name.ends_with(".out") || name.contains(".out.#")));
            if is_callgrind_file {
                files.push(entry.into_path());
            }
        }
    }

    let rows: Vec<CallgrindRecord> = files
        .par_iter()
        .filter_map(|path| parse_callgrind(root, path).ok())
        .collect();

    let mut deduped: HashMap<String, CallgrindRecord> = HashMap::new();
    let mut without_join: Vec<CallgrindRecord> = Vec::new();
    for row in rows {
        if let Some(join) = &row.join {
            deduped.entry(join.join_key.clone()).or_insert(row);
        } else {
            without_join.push(row);
        }
    }

    let mut rows: Vec<CallgrindRecord> = deduped.into_values().collect();
    rows.extend(without_join);

    rows.sort_by(|a, b| {
        a.join
            .as_ref()
            .map(|j| j.join_key.clone())
            .unwrap_or_default()
            .cmp(
                &b.join
                    .as_ref()
                    .map(|j| j.join_key.clone())
                    .unwrap_or_default(),
            )
    });
    Ok(rows)
}

fn parse_callgrind(root: &Path, path: &Path) -> Result<CallgrindRecord> {
    let raw = fs::read_to_string(path)?;
    let mut command: Option<String> = None;
    let mut events: Vec<String> = Vec::new();
    let mut metrics: HashMap<String, u64> = HashMap::new();

    for line in raw.lines() {
        if let Some(value) = line.strip_prefix("cmd:") {
            command = Some(value.trim().to_string());
            continue;
        }
        if let Some(value) = line.strip_prefix("events:") {
            events = value
                .split_whitespace()
                .map(|token| token.trim().to_string())
                .collect();
            continue;
        }
        if let Some(value) = line.strip_prefix("summary:") {
            let nums: Vec<u64> = value
                .split_whitespace()
                .filter_map(|token| token.parse::<u64>().ok())
                .collect();
            for (event, n) in events.iter().zip(nums) {
                metrics.insert(event.clone(), n);
            }
        }
    }

    let rel = path
        .strip_prefix(root)?
        .to_string_lossy()
        .replace('\\', "/");

    let join = parse_callgrind_join(&rel, command.as_deref());
    Ok(CallgrindRecord {
        join,
        measurement: CallgrindMeasurement {
            path: rel,
            events,
            metrics,
        },
    })
}

fn collect_dhat(root: &Path) -> Result<Vec<DhatRecord>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if stem.starts_with("dhat__") {
            files.push(path.to_path_buf());
        }
    }

    let mut rows: Vec<DhatRecord> = files
        .par_iter()
        .filter_map(|path| parse_dhat(root, path).ok())
        .collect();

    rows.sort_by(|a, b| {
        a.join
            .as_ref()
            .map(|j| j.join_key.clone())
            .unwrap_or_default()
            .cmp(
                &b.join
                    .as_ref()
                    .map(|j| j.join_key.clone())
                    .unwrap_or_default(),
            )
    });
    Ok(rows)
}

fn parse_dhat(root: &Path, path: &Path) -> Result<DhatRecord> {
    let raw = fs::read(path)?;
    let json: Value = serde_json::from_slice(&raw)?;

    let sampled = extract_dhat_pps_stats(&json);
    let total_bytes = get_u64(&json, "total_bytes")
        .or_else(|| get_u64(&json, "tbytes"))
        .or(sampled.total_bytes);
    let total_blocks = get_u64(&json, "total_blocks")
        .or_else(|| get_u64(&json, "tblocks"))
        .or(sampled.total_blocks);
    let max_bytes = get_u64(&json, "max_bytes")
        .or_else(|| get_u64(&json, "tmax_bytes"))
        .or(sampled.max_bytes);
    let max_blocks = get_u64(&json, "max_blocks")
        .or_else(|| get_u64(&json, "tmax_blocks"))
        .or(sampled.max_blocks);

    let mut extra_numeric_fields = HashMap::new();
    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            if let Some(n) = value.as_u64().or_else(|| value.as_i64().map(|i| i as u64)) {
                extra_numeric_fields.insert(key.clone(), n);
            }
        }
    }

    let rel = path
        .strip_prefix(root)?
        .to_string_lossy()
        .replace('\\', "/");

    Ok(DhatRecord {
        join: parse_dhat_join(path),
        measurement: DhatMeasurement {
            path: rel,
            total_bytes,
            total_blocks,
            max_bytes,
            max_blocks,
            extra_numeric_fields,
        },
    })
}

fn get_u64(json: &Value, key: &str) -> Option<u64> {
    json.get(key)
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
}

#[derive(Default)]
struct DhatPpsStats {
    total_bytes: Option<u64>,
    total_blocks: Option<u64>,
    max_bytes: Option<u64>,
    max_blocks: Option<u64>,
}

fn extract_dhat_pps_stats(json: &Value) -> DhatPpsStats {
    let Some(points) = json.get("pps").and_then(|v| v.as_array()) else {
        return DhatPpsStats::default();
    };

    let mut max_mb = 0u64;
    let mut max_tb = 0u64;
    let mut max_mbk = 0u64;
    let mut max_tbk = 0u64;
    let mut saw_mb = false;
    let mut saw_tb = false;
    let mut saw_mbk = false;
    let mut saw_tbk = false;

    for point in points {
        if let Some(mb) = point.get("mb").and_then(Value::as_u64) {
            saw_mb = true;
            max_mb = max_mb.max(mb);
        }
        if let Some(tb) = point.get("tb").and_then(Value::as_u64) {
            saw_tb = true;
            max_tb = max_tb.max(tb);
        }
        if let Some(mbk) = point.get("mbk").and_then(Value::as_u64) {
            saw_mbk = true;
            max_mbk = max_mbk.max(mbk);
        }
        if let Some(tbk) = point.get("tbk").and_then(Value::as_u64) {
            saw_tbk = true;
            max_tbk = max_tbk.max(tbk);
        }
    }

    DhatPpsStats {
        total_bytes: saw_tb.then_some(max_tb),
        total_blocks: saw_tbk.then_some(max_tbk),
        max_bytes: saw_mb.then_some(max_mb),
        max_blocks: saw_mbk.then_some(max_mbk),
    }
}

fn infer_payload(workload: &str, function: &str) -> String {
    let blob = format!("{workload} {function}").to_lowercase();
    if blob.contains("large_payload") {
        return "large_payload".to_string();
    }
    if blob.contains("string") {
        return "string".to_string();
    }
    "u64".to_string()
}

fn infer_variant(implementation: &str) -> String {
    let normalized = implementation.to_lowercase();
    if normalized.contains("_safe") {
        return "safe".to_string();
    }
    if normalized.contains("_raw") {
        return "raw".to_string();
    }
    if normalized.contains("_arena") {
        return "arena".to_string();
    }
    if normalized.contains("std_") {
        return "std".to_string();
    }
    "other".to_string()
}

fn normalize_callgrind_workload(workload: &str, payload: &str) -> String {
    if payload == "u64"
        && matches!(
            workload,
            "micro_maps"
                | "micro_heaps"
                | "macro_read_heavy"
                | "macro_write_heavy"
                | "macro_thrashing"
                | "sweep_btree_cache"
                | "sweep_hash_collisions"
        )
    {
        return format!("{workload}_u64");
    }
    workload.to_string()
}

fn build_join_keys(
    workload: &str,
    payload: &str,
    operation: &str,
    implementation: &str,
    size: usize,
    variant: &str,
) -> JoinKeys {
    JoinKeys {
        workload: workload.to_string(),
        payload: payload.to_string(),
        operation: operation.to_string(),
        implementation: implementation.to_string(),
        size,
        variant: variant.to_string(),
        join_key: format!(
            "workload={workload}|payload={payload}|op={operation}|impl={implementation}|size={size}|variant={variant}"
        ),
    }
}

fn parse_size_token(token: &str) -> Option<usize> {
    if let Some(value) = token.strip_suffix('k') {
        let parsed = value.parse::<usize>().ok()?;
        return Some(parsed * 1_000);
    }
    token.parse::<usize>().ok()
}

fn parse_callgrind_join(path: &str, command: Option<&str>) -> Option<JoinKeys> {
    // New format from split targets: callgrind_<workload>_<operation>_<implementation>.n<size>
    let name_re = Regex::new(
        r"callgrind\.(?P<name>callgrind_[a-z0-9_]+)\.n(?P<size>\d+k?|\d+)(?:\.\d+)?\.out(?:\.#\d+)?",
    )
    .ok()?;

    if let Some(caps) = name_re.captures(path) {
        let full_name = caps.name("name")?.as_str();
        let size = parse_size_token(caps.name("size")?.as_str()).unwrap_or(0);

        if full_name.starts_with("callgrind__") {
            // Legacy encoded format is handled by the fallback parser below.
        } else if let Some(rest) = full_name.strip_prefix("callgrind_") {
            let operations = [
                "contains_temporal",
                "contains_zipf",
                "contains_mixed",
                "push_pop",
                "insert",
                "remove",
                "mix",
                "thrash",
            ];

            for op in operations {
                let token = format!("_{op}_");
                if let Some(index) = rest.find(&token) {
                    let workload = rest[..index].to_string();
                    let implementation = rest[index + token.len()..].to_string();
                    let payload = infer_payload(&workload, "");
                    let workload = normalize_callgrind_workload(&workload, &payload);
                    let variant = infer_variant(&implementation);
                    return Some(build_join_keys(
                        &workload,
                        &payload,
                        op,
                        &implementation,
                        size,
                        &variant,
                    ));
                }
            }
        }
    }

    // Backstop for existing encoded format.
    let source = format!("{} {}", path, command.unwrap_or_default());
    let token_re = Regex::new(r"callgrind__[^\s/]+(?:__[^\s/]+)*").ok()?;
    let token = token_re
        .find_iter(&source)
        .last()
        .map(|m| m.as_str().to_string())?;

    let mut workload: Option<String> = None;
    let mut payload: Option<String> = None;
    let mut operation: Option<String> = None;
    let mut implementation: Option<String> = None;

    for segment in token.split("__") {
        if let Some(value) = segment.strip_prefix("workload_") {
            workload = Some(value.to_string());
        } else if let Some(value) = segment.strip_prefix("payload_") {
            payload = Some(value.to_string());
        } else if let Some(value) = segment.strip_prefix("op_") {
            operation = Some(value.to_string());
        } else if let Some(value) = segment.strip_prefix("impl_") {
            implementation = Some(value.to_string());
        }
    }

    let size_re = Regex::new(r"\.n(?P<size>\d+k?|\d+)(?:\.\d+)?\.out(?:\.#\d+)?").ok()?;
    let size = size_re
        .captures(path)
        .and_then(|size_caps| size_caps.name("size"))
        .and_then(|m| parse_size_token(m.as_str()))
        .unwrap_or(0);

    let workload = workload?;
    let payload = payload.unwrap_or_else(|| "u64".to_string());
    let workload = normalize_callgrind_workload(&workload, &payload);
    let operation = operation?;
    let implementation = implementation?;
    let variant = infer_variant(&implementation);

    Some(build_join_keys(
        &workload,
        &payload,
        &operation,
        &implementation,
        size,
        &variant,
    ))
}

fn parse_dhat_join(path: &Path) -> Option<JoinKeys> {
    let name = path.file_name()?.to_string_lossy();
    if !name.starts_with("dhat__") {
        return None;
    }

    let mut workload: Option<String> = None;
    let mut payload: Option<String> = None;
    let mut operation: Option<String> = None;
    let mut implementation: Option<String> = None;
    let mut size: Option<usize> = None;

    let stem = name.strip_suffix(".json").unwrap_or(&name);
    for segment in stem.split("__") {
        if let Some(value) = segment.strip_prefix("workload_") {
            workload = Some(value.to_string());
        } else if let Some(value) = segment.strip_prefix("payload_") {
            payload = Some(value.to_string());
        } else if let Some(value) = segment.strip_prefix("op_") {
            operation = Some(value.to_string());
        } else if let Some(value) = segment.strip_prefix("impl_") {
            implementation = Some(value.to_string());
        } else if let Some(value) = segment.strip_prefix("size_") {
            size = value.parse::<usize>().ok();
        }
    }

    let workload = workload?;
    let payload = payload.unwrap_or_else(|| "u64".to_string());
    let operation = operation?;
    let implementation = implementation?;
    let size = size.unwrap_or(0);
    let variant = infer_variant(&implementation);

    Some(build_join_keys(
        &workload,
        &payload,
        &operation,
        &implementation,
        size,
        &variant,
    ))
}
